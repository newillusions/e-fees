//! One-time (then periodic) clause usage-frequency mining job for
//! clause-selection Stage 3, Option B - see
//! docs/plans/2026-07-10-clause-selection-stage3-design.md.
//!
//! For each active library clause x each ingested corpus document, asks
//! Ollama (qwen3.6:27b, "think": false - obs:5cpj0lskql7rmiaq9jbq) whether the
//! document contains a section functionally equivalent to the clause
//! (meaning/purpose match, NOT verbatim text match - curated clause wording
//! diverges from historical proposal wording per docs/clause-corpus/
//! LIBRARY-AUDIT.md). Positive matches upsert `clause_corpus_stat`
//! (usage_count, sample_project_numbers, classified_at), one row per clause
//! keyed by the clause's own record key.
//!
//! Resumable: already-processed (clause, doc) pairs are appended to a local
//! JSONL checkpoint file and skipped on re-run. Rate-limited: a short sleep
//! between Ollama calls.
//!
//! Usage:
//!   cargo run -p e-fees-scope --bin mine_clause_usage
//!
//! Env overrides (optional; useful for a smoke slice before the full run):
//!   MINE_CLAUSE_LIMIT=2      only mine the first N active clauses
//!   MINE_DOC_LIMIT=5         only compare against the first N corpus docs
//!   MINE_CHECKPOINT_PATH     override checkpoint file location
//!                            (default: e-fees-scope/.mine-clause-usage-checkpoint.jsonl)
//!   MINE_RATE_LIMIT_MS=300   sleep between Ollama calls (default 300ms)

use std::collections::HashSet;
use std::env;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::json;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::types::RecordId;
use surrealdb::Surreal;
use surrealdb_types::SurrealValue;

use e_fees_core::models::record_key_string;

#[derive(Debug, Clone, Deserialize, SurrealValue)]
struct ClauseRow {
    id: RecordId,
    category: String,
    title: String,
    body: String,
    status: String,
    sort_order: i64,
}

#[derive(Debug, Clone, Deserialize, SurrealValue)]
struct CorpusRow {
    id: RecordId,
    filename: String,
    project_number: Option<String>,
    extracted_text: String,
    created_at: surrealdb_types::Datetime,
}

#[derive(Debug, Serialize, Deserialize)]
struct CheckpointLine {
    clause_key: String,
    doc_key: String,
    matched: bool,
}

/// Truncate at a char boundary (never a byte boundary) so multi-byte UTF-8
/// text in corpus documents can't panic the slice.
fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let truncated: String = text.chars().take(max_chars).collect();
    format!("{truncated}...")
}

fn checkpoint_key(clause_key: &str, doc_key: &str) -> String {
    format!("{clause_key}|{doc_key}")
}

fn build_classification_prompt(
    category: &str,
    title: &str,
    clause_body: &str,
    filename: &str,
    doc_text: &str,
) -> String {
    format!(
        "You are comparing a STANDARD LIBRARY CLAUSE (a template, generalised wording) against \
         ONE HISTORICAL PROPOSAL DOCUMENT. Decide whether the document contains a section that is \
         functionally EQUIVALENT to the library clause - i.e. covers the same purpose/topic - even \
         if the wording differs substantially. Curated library clauses are often paraphrased, \
         trimmed, or updated versions of the original proposal wording. Do NOT require verbatim or \
         near-verbatim text match - judge by MEANING and PURPOSE of the section, not exact phrasing.\n\n\
         LIBRARY CLAUSE\nCategory: {category}\nTitle: {title}\nBody:\n{clause_body}\n\n\
         PROPOSAL DOCUMENT ({filename})\n{doc_text}\n\n\
         Output ONLY valid JSON, no markdown, no explanation: {{\"match\": true or false, \"confidence\": 0.0 to 1.0}}"
    )
}

/// Parse the LLM's classification response - strips markdown code fences
/// (same defensive pattern as extract_clauses in routes/corpus.rs) then
/// parses `{"match": bool, "confidence": number}`.
fn parse_classification(raw: &str) -> Result<(bool, f64), String> {
    let cleaned = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let parsed: serde_json::Value =
        serde_json::from_str(cleaned).map_err(|e| format!("invalid JSON: {e}. raw: {cleaned}"))?;

    let matched = parsed
        .get("match")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| format!("missing/invalid 'match' field. raw: {cleaned}"))?;
    let confidence = parsed
        .get("confidence")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);

    Ok((matched, confidence))
}

fn load_checkpoint(path: &str) -> HashSet<String> {
    let mut seen = HashSet::new();
    let Ok(file) = std::fs::File::open(path) else {
        return seen;
    };
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        if let Ok(entry) = serde_json::from_str::<CheckpointLine>(&line) {
            seen.insert(checkpoint_key(&entry.clause_key, &entry.doc_key));
        }
    }
    seen
}

fn append_checkpoint(path: &str, entry: &CheckpointLine) -> std::io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", serde_json::to_string(entry).unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    let surreal_url =
        env::var("SURREAL_URL").unwrap_or_else(|_| "ws://10.0.23.11:8000".to_string());
    let surreal_ns = env::var("SURREAL_NS").unwrap_or_else(|_| "emittiv".to_string());
    let surreal_db = env::var("SURREAL_DB").unwrap_or_else(|_| "projects".to_string());
    let surreal_user = env::var("SURREAL_USER").expect("SURREAL_USER required");
    let surreal_pass = env::var("SURREAL_PASS").expect("SURREAL_PASS required");
    let ollama_url =
        env::var("OLLAMA_URL").unwrap_or_else(|_| "http://10.0.21.20:11434".to_string());
    let ollama_model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen3.6:27b".to_string());

    let clause_limit: Option<usize> = env::var("MINE_CLAUSE_LIMIT").ok().and_then(|v| v.parse().ok());
    let doc_limit: Option<usize> = env::var("MINE_DOC_LIMIT").ok().and_then(|v| v.parse().ok());
    let checkpoint_path = env::var("MINE_CHECKPOINT_PATH")
        .unwrap_or_else(|_| ".mine-clause-usage-checkpoint.jsonl".to_string());
    let rate_limit_ms: u64 = env::var("MINE_RATE_LIMIT_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);

    println!("mine_clause_usage: connecting to {surreal_ns}/{surreal_db} at {surreal_url}");

    let connection_address = surreal_url
        .strip_prefix("ws://")
        .or_else(|| surreal_url.strip_prefix("wss://"))
        .unwrap_or(&surreal_url);
    let db: Surreal<Client> = Surreal::new::<Ws>(connection_address).await?;
    db.signin(Root {
        username: surreal_user.clone(),
        password: surreal_pass.clone(),
    })
    .await?;
    db.use_ns(&surreal_ns).use_db(&surreal_db).await?;

    let mut clause_res = db
        .query(
            "SELECT id, category, title, body, status, sort_order FROM clause \
             WHERE status = 'active' ORDER BY sort_order ASC",
        )
        .await?;
    let mut clauses: Vec<ClauseRow> = clause_res.take(0)?;
    if let Some(limit) = clause_limit {
        clauses.truncate(limit);
    }

    let mut doc_res = db
        .query(
            "SELECT id, filename, project_number, extracted_text, created_at FROM proposal_corpus \
             ORDER BY created_at DESC",
        )
        .await?;
    let mut docs: Vec<CorpusRow> = doc_res.take(0)?;
    if let Some(limit) = doc_limit {
        docs.truncate(limit);
    }

    println!(
        "mine_clause_usage: {} clauses x {} corpus docs = {} pairs (checkpoint: {})",
        clauses.len(),
        docs.len(),
        clauses.len() * docs.len(),
        checkpoint_path
    );

    let already_done = load_checkpoint(&checkpoint_path);
    let http = reqwest::Client::new();

    let mut positives = 0usize;
    let mut negatives = 0usize;
    let mut skipped = 0usize;
    let mut errors = 0usize;

    for clause in &clauses {
        let clause_raw = record_key_string(&clause.id.key);
        let clause_key = clause_raw
            .strip_prefix("clause:")
            .unwrap_or(&clause_raw)
            .to_string();

        for doc in &docs {
            let doc_raw = record_key_string(&doc.id.key);
            let doc_key = doc_raw
                .strip_prefix("proposal_corpus:")
                .unwrap_or(&doc_raw)
                .to_string();

            if already_done.contains(&checkpoint_key(&clause_key, &doc_key)) {
                skipped += 1;
                continue;
            }

            let prompt = build_classification_prompt(
                &clause.category,
                &clause.title,
                &clause.body,
                &doc.filename,
                &truncate_text(&doc.extracted_text, 6000),
            );

            let llm_body = json!({
                "model": &ollama_model,
                "prompt": prompt,
                "system": "You are a precise document classifier. Output ONLY valid JSON. No markdown, no explanation.",
                "stream": false,
                "think": false,
                "options": { "temperature": 0.1 }
            });

            let res = match http
                .post(format!("{ollama_url}/api/generate"))
                .json(&llm_body)
                .timeout(Duration::from_secs(120))
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("mine_clause_usage: Ollama request failed for clause={clause_key} doc={doc_key}: {e}");
                    errors += 1;
                    continue;
                }
            };

            if !res.status().is_success() {
                eprintln!(
                    "mine_clause_usage: Ollama returned {} for clause={clause_key} doc={doc_key}",
                    res.status()
                );
                errors += 1;
                continue;
            }

            let result: serde_json::Value = match res.json().await {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("mine_clause_usage: failed to parse Ollama response JSON: {e}");
                    errors += 1;
                    continue;
                }
            };

            let raw_response = result["response"].as_str().unwrap_or("");
            let (matched, confidence) = match parse_classification(raw_response) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("mine_clause_usage: classification parse failed for clause={clause_key} doc={doc_key}: {e}");
                    errors += 1;
                    continue;
                }
            };

            if matched {
                positives += 1;
                let project_number = doc.project_number.clone().unwrap_or_else(|| doc.filename.clone());
                let upsert_query = format!(
                    "UPSERT clause_corpus_stat:{clause_key} SET \
                     clause_id = type::record('clause', $clause_key), \
                     category = $category, \
                     usage_count += 1, \
                     sample_project_numbers += $project_number, \
                     classified_at = time::now();"
                );
                if let Err(e) = db
                    .query(&upsert_query)
                    .bind(("clause_key", clause_key.clone()))
                    .bind(("category", clause.category.clone()))
                    .bind(("project_number", project_number))
                    .await
                {
                    eprintln!("mine_clause_usage: upsert failed for clause={clause_key}: {e}");
                    errors += 1;
                    continue;
                }
                println!(
                    "  MATCH  clause={clause_key} doc={doc_key} confidence={confidence:.2} ({})",
                    clause.title
                );
            } else {
                negatives += 1;
            }

            if let Err(e) = append_checkpoint(
                &checkpoint_path,
                &CheckpointLine {
                    clause_key: clause_key.clone(),
                    doc_key: doc_key.clone(),
                    matched,
                },
            ) {
                eprintln!("mine_clause_usage: checkpoint write failed: {e}");
            }

            tokio::time::sleep(Duration::from_millis(rate_limit_ms)).await;
        }
    }

    println!(
        "mine_clause_usage: done. positives={positives} negatives={negatives} skipped(resumed)={skipped} errors={errors}"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_text_leaves_short_text_unchanged() {
        assert_eq!(truncate_text("short", 100), "short");
    }

    #[test]
    fn truncate_text_truncates_at_char_boundary_with_ellipsis() {
        let text = "abcdefghij";
        assert_eq!(truncate_text(text, 4), "abcd...");
    }

    #[test]
    fn truncate_text_never_panics_on_multibyte_utf8() {
        // Each of these chars is multi-byte in UTF-8; a naive byte-slice
        // truncation at an odd offset would panic. Char-based truncation
        // must not.
        let text = "\u{00e9}\u{00e9}\u{00e9}\u{00e9}\u{00e9}\u{00e9}\u{00e9}\u{00e9}\u{00e9}\u{00e9}";
        let out = truncate_text(text, 3);
        assert_eq!(out.chars().filter(|c| *c == '\u{00e9}').count(), 3);
    }

    #[test]
    fn checkpoint_key_is_stable_and_distinguishes_pairs() {
        assert_eq!(checkpoint_key("c1", "d1"), "c1|d1");
        assert_ne!(checkpoint_key("c1", "d1"), checkpoint_key("c1", "d2"));
    }

    #[test]
    fn parse_classification_handles_plain_json() {
        let (matched, confidence) = parse_classification(r#"{"match": true, "confidence": 0.87}"#)
            .expect("should parse");
        assert!(matched);
        assert!((confidence - 0.87).abs() < 1e-9);
    }

    #[test]
    fn parse_classification_strips_markdown_fences() {
        let raw = "```json\n{\"match\": false, \"confidence\": 0.2}\n```";
        let (matched, confidence) = parse_classification(raw).expect("should parse");
        assert!(!matched);
        assert!((confidence - 0.2).abs() < 1e-9);
    }

    #[test]
    fn parse_classification_defaults_confidence_when_missing() {
        let (matched, confidence) =
            parse_classification(r#"{"match": true}"#).expect("should parse");
        assert!(matched);
        assert_eq!(confidence, 0.0);
    }

    #[test]
    fn parse_classification_rejects_missing_match_field() {
        let result = parse_classification(r#"{"confidence": 0.5}"#);
        assert!(result.is_err());
    }

    #[test]
    fn parse_classification_rejects_invalid_json() {
        let result = parse_classification("not json at all");
        assert!(result.is_err());
    }

    #[test]
    fn build_classification_prompt_includes_all_inputs_and_instructs_against_verbatim_bias() {
        let prompt = build_classification_prompt(
            "Services",
            "Scope & Services - Areas",
            "clause body text",
            "26-97104-FP.pdf",
            "document text here",
        );
        assert!(prompt.contains("Services"));
        assert!(prompt.contains("Scope & Services - Areas"));
        assert!(prompt.contains("clause body text"));
        assert!(prompt.contains("26-97104-FP.pdf"));
        assert!(prompt.contains("document text here"));
        assert!(prompt.to_lowercase().contains("not require verbatim"));
    }
}
