//! Historical pricing backfill, Phase P2 - "stage/outcome population from
//! the live DB".
//!
//! Populates the P0 fields `stage`, `outcome`, `outcome_changed_at`, and
//! `successor` on `projects` from the existing (workflow-owned, NEVER
//! written by this tool) `projects.status` field, per the owner-decided
//! mapping in
//! /Volumes/base/dev/.claude/research/2026-08-08-pricing-ingestion/03-ingestion-plan.md
//! section 2 "P2", plus the two follow-up owner rulings recorded on the
//! e-fees mission record's `ns-backfill-p2-awarded-mapping` next-step
//! (2026-08-19): Wynn Al Marjan Island (25-97102) and Lusail Plaza
//! (26-97401) map Awarded -> Design/Won (both are "reviews"), and the
//! Reserve Cut (24-97107) Superseded chain resolves to 24-97113 Level 63
//! from owner memory (no lexical relationship between the folder names, so
//! the folder-token-overlap algorithm below cannot find it on its own).
//!
//! Every write also appends one `activity_log` row when the `outcome`
//! value actually changes (the "outcome-change-log" requirement, P0 DDL
//! comment on the `outcome` field) - a stage-only backfill (RFP/Design,
//! outcome stays NONE/open) is not logged, since nothing about the
//! project's commercial outcome moved.
//!
//! Idempotent: a project whose stage/outcome/successor already match the
//! target is skipped entirely (no write, no duplicate activity_log row).
//!
//! Usage:
//!   cargo run -p e-fees-api --bin backfill_stage_outcome -- --target dev
//!   cargo run -p e-fees-api --bin backfill_stage_outcome -- --target dev --apply
//!   cargo run -p e-fees-api --bin backfill_stage_outcome -- --target prod                          (dry-run plan only)
//!   cargo run -p e-fees-api --bin backfill_stage_outcome -- --target prod --apply --confirm-prod    (orchestrator-run, authorized production apply)
//!
//! Default posture is dry-run, and `--apply --target prod` alone is
//! REFUSED before any DB connection is made. Production writes are
//! orchestrator-owned (see the e-fees CLAUDE.md Deploy Ownership
//! convention) - the explicit `--confirm-prod` flag is the deliberate,
//! non-default escape hatch for the orchestrator (or Martin directly) to
//! run the owner-authorized P1+P2 production load through this same
//! binary. This agent's own dispatch scope stays code-only regardless -
//! `--confirm-prod` is documented here for the orchestrator, not exercised
//! by this dispatch.
//!
//! Credentials: `SURREAL_USER` / `SURREAL_PASS`, falling back to
//! `EFEES_SURREALDB_USER` / `EFEES_SURREALDB_PASS` (the workspace
//! credential-directory names for this project) - never printed, never
//! logged. `SURREAL_URL` / `SURREAL_NS` / `SURREAL_DB` override the
//! `--target`-derived connection defaults, useful for pointing at a
//! throwaway namespace in tests.

use std::collections::{HashMap, HashSet};
use std::env;

use serde::Deserialize;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::types::RecordId;
use surrealdb::Surreal;
use surrealdb_types::SurrealValue;

use e_fees_core::models::record_key_string;

// ============================================================================
// STATUS -> STAGE/OUTCOME MAPPING (owner-decided, plan section 2 "P2")
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
struct StatusMapping {
    stage: &'static str,
    outcome: Option<&'static str>,
    /// "high" for an unambiguous 1:1 mapping, "medium" for an interpretive
    /// default (Lost -> No Response, Awarded -> Design) per the plan's
    /// data-provenance design (section 4).
    confidence: &'static str,
}

/// Map a live `projects.status` value to its target stage/outcome. Returns
/// `None` for any status with no P2 mapping entry (e.g. dev's `Lead` /
/// `No Response` / `Submitted` / `On Hold` rows, which exist in the schema's
/// ASSERT enum but are out of scope for this phase) - callers must report
/// these as unmapped/skipped, never guess a mapping for them.
fn map_status(status: &str) -> Option<StatusMapping> {
    Some(match status {
        "RFP" => StatusMapping {
            stage: "RFP",
            outcome: None,
            confidence: "high",
        },
        "Design" => StatusMapping {
            stage: "Design",
            outcome: None,
            confidence: "high",
        },
        "Completed" => StatusMapping {
            stage: "Completed",
            outcome: Some("Won"),
            confidence: "high",
        },
        "Cancelled" => StatusMapping {
            stage: "Cancelled",
            outcome: Some("Cancelled"),
            confidence: "high",
        },
        "Superseded" => StatusMapping {
            stage: "Superseded",
            outcome: Some("Won"),
            confidence: "high",
        },
        // Owner ruling 2026-08-19 (ns-backfill-p2-awarded-mapping): both
        // Awarded rows (Wynn Al Marjan Island, Lusail Plaza) are "reviews"
        // and stay in Design. No "Awarded" stage value exists - the enum
        // (RFP, Design, Construction, Completed, Superseded, Cancelled) is
        // final.
        "Awarded" => StatusMapping {
            stage: "Design",
            outcome: Some("Won"),
            confidence: "medium",
        },
        // D3 ruling: "Lost" was folder-housekeeping, not a genuine loss
        // record - default all 47 historical rows to No Response; a
        // handful may be genuine losses or reached Design, surfaced by the
        // lost-with-design-history cross-check below rather than
        // auto-classified.
        "Lost" => StatusMapping {
            stage: "RFP",
            outcome: Some("No Response"),
            confidence: "medium",
        },
        _ => return None,
    })
}

// ============================================================================
// SUCCESSOR LINEAGE RESOLUTION (folder match, never name - P0 DDL comment)
// ============================================================================

const VERSION_STOPWORDS: &[&str] = &["v2", "v3", "v4", "v5", "v6", "revision", "rev", "new"];
const SUCCESSOR_JACCARD_THRESHOLD: f64 = 0.5;

/// Owner-ruling overrides for lineage pairs the folder-token-overlap
/// algorithm cannot derive on its own (the successor's folder shares no
/// lexical relationship with the predecessor's). Keyed by the
/// PREDECESSOR's folder string (never by `name` - P0 DDL comment / plan
/// section 2 P2), each entry names the SUCCESSOR's folder string.
const MANUAL_SUCCESSOR_OVERRIDES: &[(&str, &str)] = &[
    // Owner ruling 2026-08-19 (mission-record next-step
    // ns-backfill-p2-awarded-mapping, verbatim): "Reserve cut became level
    // 63". "Reserve Cut" and "Level 63" share zero tokens, so the generic
    // matcher below scores this pair 0.0 - asserted from owner memory
    // instead. The owner also noted a further rename may follow; that is
    // simply the next successor pointer when it happens, not a schema
    // change.
    ("24-97107 Reserve Cut", "24-97113 Level 63"),
];

fn is_project_number_prefix(s: &str) -> bool {
    match s.split_once('-') {
        Some((a, b)) => {
            a.len() == 2
                && a.chars().all(|c| c.is_ascii_digit())
                && !b.is_empty()
                && b.chars().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

/// Parse a folder's leading "NN-NNNNN" project number into a comparable
/// `(year, seq)` tuple, or `None` if the folder has no such prefix.
fn parse_project_number_prefix(folder: &str) -> Option<(u32, u32)> {
    let prefix = folder.split_once(' ').map(|(p, _)| p).unwrap_or(folder);
    if !is_project_number_prefix(prefix) {
        return None;
    }
    let (year, seq) = prefix.split_once('-')?;
    Some((year.parse().ok()?, seq.parse().ok()?))
}

/// Tokenize a project folder into its comparable "base name" token set:
/// strip the leading "NN-NNNNN " project-number prefix (if present), then
/// lowercase and split on non-alphanumeric boundaries, dropping generic
/// version-suffix tokens (`v2`, `revision`, ...) so a version bump alone
/// does not depress the overlap score against its own predecessor.
fn folder_base_tokens(folder: &str) -> HashSet<String> {
    let body = match folder.split_once(' ') {
        Some((prefix, rest)) if is_project_number_prefix(prefix) => rest,
        _ => folder,
    };
    body.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty() && !VERSION_STOPWORDS.contains(s))
        .map(str::to_string)
        .collect()
}

fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(b).count() as f64;
    let union = a.union(b).count() as f64;
    inter / union
}

/// Resolve a Superseded project's successor by folder-token overlap,
/// falling back to the manual owner-ruling override table for pairs the
/// generic algorithm cannot derive. `candidates` must exclude the source
/// project itself. Returns the matching candidate folder string, or `None`
/// when no candidate clears the threshold and no override applies (an
/// unresolved/open lineage - reported, never guessed).
///
/// The generic algorithm additionally requires a resolvable candidate's
/// project number to sort AFTER the source's (chronologically later, per
/// this repo's YY-CCCNN numbering) - without this, a project's own
/// predecessor can token-match back onto it (e.g. 25-97104 "MAF MiCC v2"
/// scores 0.667 against its OWN predecessor 24-97105 "MAF MiCC Kids",
/// which would wrongly resolve 25-97104's successor to a project that came
/// BEFORE it). Folders that don't parse as a project number are left
/// unfiltered (permissive default) rather than silently excluded.
fn resolve_successor_folder<'a>(source_folder: &str, candidates: &[&'a str]) -> Option<&'a str> {
    if let Some((_, target)) = MANUAL_SUCCESSOR_OVERRIDES
        .iter()
        .find(|(pred, _)| *pred == source_folder)
    {
        return candidates.iter().find(|c| *c == target).copied();
    }

    let source_number = parse_project_number_prefix(source_folder);
    let forward_candidates = candidates.iter().copied().filter(|c| {
        match (source_number, parse_project_number_prefix(c)) {
            (Some(src), Some(cand)) => cand > src,
            _ => true,
        }
    });

    let source_tokens = folder_base_tokens(source_folder);
    forward_candidates
        .map(|c| (c, jaccard(&source_tokens, &folder_base_tokens(c))))
        .filter(|(_, score)| *score >= SUCCESSOR_JACCARD_THRESHOLD)
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).expect("jaccard scores are finite"))
        .map(|(c, _)| c)
}

// ============================================================================
// DB ROW SHAPES (bespoke - deliberately not the shared `Project` model,
// which does not carry the new P0 fields yet)
// ============================================================================

#[derive(Debug, Clone, Deserialize, SurrealValue)]
struct ProjectRow {
    id: RecordId,
    folder: String,
    name_short: String,
    status: String,
    stage: Option<String>,
    outcome: Option<String>,
    successor: Option<RecordId>,
}

#[derive(Debug, Clone, Deserialize, SurrealValue)]
struct FeeRow {
    project_id: RecordId,
    status: String,
}

// ============================================================================
// PLAN COMPUTATION (pure - testable without a DB connection)
// ============================================================================

#[derive(Debug, Clone)]
struct ProjectPlan {
    key: String,
    folder: String,
    status: String,
    target_stage: String,
    target_outcome: Option<String>,
    target_successor_key: Option<String>,
    confidence: &'static str,
    already_correct: bool,
    flags: Vec<String>,
}

#[allow(clippy::too_many_arguments)]
fn build_plan(
    project: &ProjectRow,
    mapping: &StatusMapping,
    target_successor_key: Option<String>,
    successor_unresolved: bool,
    fee_statuses: &[String],
) -> ProjectPlan {
    let key = record_key_string(&project.id.key);
    let target_outcome = mapping.outcome.map(str::to_string);

    let current_successor_key = project
        .successor
        .as_ref()
        .map(|r| record_key_string(&r.key));
    let already_correct = project.stage.as_deref() == Some(mapping.stage)
        && project.outcome == target_outcome
        && current_successor_key == target_successor_key;

    let mut flags = Vec::new();

    if project.status == "Lost"
        && fee_statuses
            .iter()
            .any(|s| s == "Accepted" || s == "Negotiation")
    {
        flags.push(format!(
            "lost-with-design-shaped-fee-history: fee status(es) found = {}",
            fee_statuses.join(", ")
        ));
    }

    if target_outcome.as_deref() == Some("Won") && !fee_statuses.iter().any(|s| s == "Accepted") {
        flags.push("won-zero-accepted-fee: outcome target Won, no Accepted fee row found".into());
    }

    if project.status == "Superseded" && successor_unresolved {
        flags.push("successor-unresolved: open lineage chain, needs owner input".into());
    }

    ProjectPlan {
        key,
        folder: project.folder.clone(),
        status: project.status.clone(),
        target_stage: mapping.stage.to_string(),
        target_outcome,
        target_successor_key,
        confidence: mapping.confidence,
        already_correct,
        flags,
    }
}

// ============================================================================
// APPLY GATE (pure - testable without a DB connection)
// ============================================================================

/// Gate decision for `--apply --target prod`: refused unless the operator
/// also passes the explicit `--confirm-prod` flag. Production writes are
/// orchestrator-owned (e-fees CLAUDE.md Deploy Ownership convention) - this
/// tool's default posture stays refusal, but the orchestrator (or Martin
/// directly) needs a real path to run the authorized production apply
/// through this same binary rather than a second hand-written tool.
/// `--target dev` is never gated, regardless of `--confirm-prod`.
fn check_prod_apply_gate(target: &str, apply: bool, confirm_prod: bool) -> Result<(), String> {
    if target == "prod" && apply && !confirm_prod {
        return Err(
            "REFUSED. --apply --target prod also requires --confirm-prod (explicit, deliberate \
             production-write confirmation - not a default-on flag). Run --target prod without \
             --apply for a dry-run plan first."
                .to_string(),
        );
    }
    Ok(())
}

// ============================================================================
// MAIN
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    let args: Vec<String> = env::args().collect();
    let mut target: Option<String> = None;
    let mut apply = false;
    let mut confirm_prod = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--target" => {
                i += 1;
                target = args.get(i).cloned();
            }
            "--apply" => apply = true,
            "--dry-run" => apply = false,
            "--confirm-prod" => confirm_prod = true,
            other => {
                eprintln!("backfill_stage_outcome: unknown argument '{other}'");
                std::process::exit(2);
            }
        }
        i += 1;
    }

    let target = target.unwrap_or_else(|| {
        eprintln!("backfill_stage_outcome: --target dev|prod is required");
        std::process::exit(2);
    });

    let (default_url, default_ns, default_db) = match target.as_str() {
        "dev" => ("ws://10.0.23.12:8000", "emittiv_dev", "projects"),
        "prod" => ("ws://10.0.23.11:8000", "emittiv", "projects"),
        other => {
            eprintln!("backfill_stage_outcome: --target must be 'dev' or 'prod', got '{other}'");
            std::process::exit(2);
        }
    };

    // Gate check, before any DB connection is made: --apply --target prod
    // needs the explicit --confirm-prod flag too (see check_prod_apply_gate).
    if let Err(msg) = check_prod_apply_gate(&target, apply, confirm_prod) {
        eprintln!("backfill_stage_outcome: {msg}");
        std::process::exit(1);
    }

    let surreal_url = env::var("SURREAL_URL").unwrap_or_else(|_| default_url.to_string());
    let surreal_ns = env::var("SURREAL_NS").unwrap_or_else(|_| default_ns.to_string());
    let surreal_db = env::var("SURREAL_DB").unwrap_or_else(|_| default_db.to_string());
    let surreal_user = env::var("SURREAL_USER")
        .or_else(|_| env::var("EFEES_SURREALDB_USER"))
        .expect("SURREAL_USER or EFEES_SURREALDB_USER required");
    let surreal_pass = env::var("SURREAL_PASS")
        .or_else(|_| env::var("EFEES_SURREALDB_PASS"))
        .expect("SURREAL_PASS or EFEES_SURREALDB_PASS required");

    println!(
        "backfill_stage_outcome: target={target} apply={apply} connecting to {surreal_ns}/{surreal_db} at {surreal_url}"
    );

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

    let mut project_res = db
        .query("SELECT id, folder, name_short, status, stage, outcome, successor FROM projects;")
        .await?;
    let projects: Vec<ProjectRow> = project_res.take(0)?;

    let mut fee_res = db.query("SELECT project_id, status FROM fee;").await?;
    let fees: Vec<FeeRow> = fee_res.take(0)?;

    let mut fee_statuses_by_project: HashMap<String, Vec<String>> = HashMap::new();
    for fee in &fees {
        fee_statuses_by_project
            .entry(record_key_string(&fee.project_id.key))
            .or_default()
            .push(fee.status.clone());
    }

    // Real status breakdown, re-read at execution time (never trust a
    // figure quoted in a plan doc - live rows change continuously).
    let mut status_counts: HashMap<String, usize> = HashMap::new();
    for p in &projects {
        *status_counts.entry(p.status.clone()).or_insert(0) += 1;
    }
    println!("\nLive status breakdown (n={}):", projects.len());
    let mut status_keys: Vec<&String> = status_counts.keys().collect();
    status_keys.sort();
    for status in status_keys {
        println!("  {status}: {}", status_counts[status]);
    }

    let folder_to_key: HashMap<String, String> = projects
        .iter()
        .map(|p| (p.folder.clone(), record_key_string(&p.id.key)))
        .collect();
    let all_folders: Vec<&str> = projects.iter().map(|p| p.folder.as_str()).collect();

    let mut plans: Vec<ProjectPlan> = Vec::new();
    let mut unmapped: Vec<(String, String)> = Vec::new();

    for project in &projects {
        let Some(mapping) = map_status(&project.status) else {
            unmapped.push((record_key_string(&project.id.key), project.status.clone()));
            continue;
        };

        let (target_successor_key, successor_unresolved) = if project.status == "Superseded" {
            let candidates: Vec<&str> = all_folders
                .iter()
                .copied()
                .filter(|f| *f != project.folder)
                .collect();
            match resolve_successor_folder(&project.folder, &candidates) {
                Some(successor_folder) => (folder_to_key.get(successor_folder).cloned(), false),
                None => (None, true),
            }
        } else {
            (None, false)
        };

        let fee_statuses = fee_statuses_by_project
            .get(&record_key_string(&project.id.key))
            .cloned()
            .unwrap_or_default();

        plans.push(build_plan(
            project,
            &mapping,
            target_successor_key,
            successor_unresolved,
            &fee_statuses,
        ));
    }

    // Per-bucket plan report.
    let mut by_status: HashMap<&str, Vec<&ProjectPlan>> = HashMap::new();
    for plan in &plans {
        by_status.entry(plan.status.as_str()).or_default().push(plan);
    }
    println!("\nPer-bucket plan:");
    let mut bucket_keys: Vec<&&str> = by_status.keys().collect();
    bucket_keys.sort();
    for status in bucket_keys {
        let bucket = &by_status[*status];
        let already = bucket.iter().filter(|p| p.already_correct).count();
        let to_write = bucket.len() - already;
        let sample = &bucket[0];
        println!(
            "  {status} -> stage={} outcome={} (confidence={}): {} projects, {already} already correct (skip), {to_write} to write",
            sample.target_stage,
            sample.target_outcome.as_deref().unwrap_or("NONE"),
            sample.confidence,
            bucket.len()
        );
    }

    if !unmapped.is_empty() {
        println!("\nUnmapped statuses (no P2 mapping entry - skipped, not guessed):");
        for (key, status) in &unmapped {
            println!("  {key}: status={status}");
        }
    }

    println!("\nSuccessor pointers:");
    for plan in plans.iter().filter(|p| p.status == "Superseded") {
        match &plan.target_successor_key {
            Some(succ_key) => println!("  {} ({}) -> {succ_key}", plan.key, plan.folder),
            None => println!("  {} ({}) -> UNRESOLVED (open chain)", plan.key, plan.folder),
        }
    }

    let all_flags: Vec<(&str, &str, &str)> = plans
        .iter()
        .flat_map(|p| p.flags.iter().map(move |f| (p.key.as_str(), p.folder.as_str(), f.as_str())))
        .collect();
    if !all_flags.is_empty() {
        println!("\nFlags (report only, never auto-written):");
        for (key, folder, flag) in &all_flags {
            println!("  [{key}] {folder}: {flag}");
        }
    }

    let to_apply: Vec<&ProjectPlan> = plans.iter().filter(|p| !p.already_correct).collect();
    println!(
        "\n{} of {} mapped projects already correct; {} would be written.",
        plans.len() - to_apply.len(),
        plans.len(),
        to_apply.len()
    );

    if !apply {
        println!("\nDRY RUN - no writes performed. Re-run with --apply to write.");
        return Ok(());
    }

    let mut written = 0usize;
    let mut activity_logged = 0usize;
    for project in &projects {
        let Some(plan) = plans.iter().find(|p| p.key == record_key_string(&project.id.key)) else {
            continue;
        };
        if plan.already_correct {
            continue;
        }

        let successor_bind: Option<RecordId> = plan
            .target_successor_key
            .as_ref()
            .map(|k| RecordId::new("projects", k.clone()));

        let provenance = serde_json::json!({
            "source": "backfill",
            "confidence": plan.confidence,
            "backfilled_at": chrono::Utc::now().to_rfc3339(),
            "backfilled_by": "ingestion-P2-stage-outcome",
        });

        // .check() is mandatory, not decoration: `db.query(...).await` only
        // surfaces TRANSPORT-level errors - a per-statement failure (e.g. a
        // SCHEMAFULL rejection) leaves the outer Result Ok and is silently
        // swallowed unless .check() is called to convert it into an Err.
        // Found live during dev-proving: the activity_log write below
        // originally set `metadata = $metadata` with a structured object,
        // which SurrealDB rejected ("Found field 'metadata.stage', but no
        // such field exists for table 'activity_log'" - activity_log's
        // `metadata` field is `TYPE none | object`, NOT FLEXIBLE, with no
        // sub-fields ever defined) - every activity_log CREATE failed while
        // this tool printed "APPLIED: ... created N activity_log row(s)"
        // and the UPDATE-only counters kept incrementing regardless. Same
        // failure CLASS as obs:kss52yw38ev0vpcrlm1n (query succeeds, no
        // rows actually change) - only .check() closes it structurally.
        db.query(
            "UPDATE type::record('projects', $key) SET \
             stage = $stage, \
             outcome = $outcome, \
             outcome_changed_at = time::now(), \
             successor = $successor, \
             data_provenance = $provenance;",
        )
        .bind(("key", plan.key.clone()))
        .bind(("stage", plan.target_stage.clone()))
        .bind(("outcome", plan.target_outcome.clone()))
        .bind(("successor", successor_bind))
        .bind(("provenance", provenance))
        .await?
        .check()?;
        written += 1;

        let outcome_changed = project.outcome != plan.target_outcome;
        if outcome_changed && plan.target_outcome.is_some() {
            // `metadata` is deliberately NOT set (see the .check() note
            // above) - the same context (stage, source status) already
            // lives in `description` as free text, which activity_log's
            // schema does support.
            db.query(
                "CREATE activity_log SET \
                 entity_type = 'project', \
                 entity_id = $entity_id, \
                 entity_name = $entity_name, \
                 action = 'status_change', \
                 description = $description, \
                 old_value = $old_value, \
                 new_value = $new_value, \
                 user = 'backfill:ingestion-P2-stage-outcome';",
            )
            .bind(("entity_id", plan.key.clone()))
            .bind(("entity_name", plan.folder.clone()))
            .bind((
                "description",
                format!(
                    "Backfill P2: outcome set to {} (stage {}) from live status '{}'",
                    plan.target_outcome.as_deref().unwrap_or("NONE"),
                    plan.target_stage,
                    plan.status
                ),
            ))
            .bind((
                "old_value",
                project.outcome.clone().unwrap_or_else(|| "NONE".to_string()),
            ))
            .bind((
                "new_value",
                plan.target_outcome.clone().unwrap_or_else(|| "NONE".to_string()),
            ))
            .await?
            .check()?;
            activity_logged += 1;
        }
    }

    println!("\nAPPLIED: wrote {written} project(s), created {activity_logged} activity_log row(s).");

    Ok(())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -- map_status -----------------------------------------------------

    #[test]
    fn map_status_obvious_1to1_buckets_are_high_confidence() {
        assert_eq!(
            map_status("RFP"),
            Some(StatusMapping {
                stage: "RFP",
                outcome: None,
                confidence: "high"
            })
        );
        assert_eq!(
            map_status("Design"),
            Some(StatusMapping {
                stage: "Design",
                outcome: None,
                confidence: "high"
            })
        );
        assert_eq!(
            map_status("Completed"),
            Some(StatusMapping {
                stage: "Completed",
                outcome: Some("Won"),
                confidence: "high"
            })
        );
        assert_eq!(
            map_status("Cancelled"),
            Some(StatusMapping {
                stage: "Cancelled",
                outcome: Some("Cancelled"),
                confidence: "high"
            })
        );
        assert_eq!(
            map_status("Superseded"),
            Some(StatusMapping {
                stage: "Superseded",
                outcome: Some("Won"),
                confidence: "high"
            })
        );
    }

    #[test]
    fn map_status_awarded_maps_to_design_won_medium_confidence() {
        // Owner ruling 2026-08-19: both Awarded rows are "reviews", stay in
        // Design. No "Awarded" stage value exists.
        assert_eq!(
            map_status("Awarded"),
            Some(StatusMapping {
                stage: "Design",
                outcome: Some("Won"),
                confidence: "medium"
            })
        );
    }

    #[test]
    fn map_status_lost_defaults_to_no_response_medium_confidence() {
        // D3 ruling: Lost was folder-housekeeping, default to No Response.
        assert_eq!(
            map_status("Lost"),
            Some(StatusMapping {
                stage: "RFP",
                outcome: Some("No Response"),
                confidence: "medium"
            })
        );
    }

    #[test]
    fn map_status_never_writes_open_and_never_guesses_unmapped_statuses() {
        // "Open" is never a value (D3) - absence of outcome already means
        // open, so it must never appear as a literal mapping target.
        for status in ["RFP", "Design", "Completed", "Cancelled", "Superseded", "Awarded", "Lost"] {
            let mapping = map_status(status).unwrap();
            assert_ne!(mapping.stage, "Open");
            assert_ne!(mapping.outcome, Some("Open"));
        }
        // Statuses that exist in the schema's ASSERT enum but have no P2
        // mapping (seen live on dev: Lead, No Response) must not be
        // guessed - they come back None and the caller reports them
        // unmapped.
        assert_eq!(map_status("Lead"), None);
        assert_eq!(map_status("No Response"), None);
        assert_eq!(map_status("Submitted"), None);
        assert_eq!(map_status("On Hold"), None);
        assert_eq!(map_status("totally-unknown"), None);
    }

    // -- folder_base_tokens / jaccard ------------------------------------

    #[test]
    fn folder_base_tokens_strips_project_number_prefix_and_lowercases() {
        let tokens = folder_base_tokens("25-97101 Shanghai Tang");
        assert_eq!(
            tokens,
            HashSet::from(["shanghai".to_string(), "tang".to_string()])
        );
    }

    #[test]
    fn folder_base_tokens_strips_version_suffix_tokens() {
        let tokens = folder_base_tokens("25-97105 Shanghai Tang v2");
        assert_eq!(
            tokens,
            HashSet::from(["shanghai".to_string(), "tang".to_string()])
        );
    }

    #[test]
    fn folder_base_tokens_does_not_strip_non_version_words() {
        // "Kids" is real content, not a version marker - it must survive
        // tokenization even though it reduces the overlap score against
        // "MAF MiCC v2".
        let tokens = folder_base_tokens("24-97105 MAF MiCC Kids");
        assert_eq!(
            tokens,
            HashSet::from(["maf".to_string(), "micc".to_string(), "kids".to_string()])
        );
    }

    #[test]
    fn jaccard_identical_sets_scores_one() {
        let a = HashSet::from(["shanghai".to_string(), "tang".to_string()]);
        assert_eq!(jaccard(&a, &a), 1.0);
    }

    #[test]
    fn jaccard_disjoint_sets_scores_zero() {
        let a = HashSet::from(["reserve".to_string(), "cut".to_string()]);
        let b = HashSet::from(["level".to_string(), "63".to_string()]);
        assert_eq!(jaccard(&a, &b), 0.0);
    }

    // -- resolve_successor_folder / the four owner-decided lineage pairs --

    #[test]
    fn resolve_successor_shanghai_tang_matches_via_folder_token_overlap() {
        let candidates = [
            "25-97105 Shanghai Tang v2",
            "24-97107 Reserve Cut",
            "24-97113 Level 63",
            "26-97401 Lusail Plaza",
        ];
        assert_eq!(
            resolve_successor_folder("25-97101 Shanghai Tang", &candidates),
            Some("25-97105 Shanghai Tang v2")
        );
    }

    #[test]
    fn resolve_successor_maf_micc_kids_matches_via_folder_token_overlap() {
        let candidates = [
            "25-97104 MAF MiCC v2",
            "25-97101 Shanghai Tang",
            "24-97107 Reserve Cut",
        ];
        assert_eq!(
            resolve_successor_folder("24-97105 MAF MiCC Kids", &candidates),
            Some("25-97104 MAF MiCC v2")
        );
    }

    #[test]
    fn resolve_successor_reserve_cut_uses_manual_owner_ruling_override() {
        // "Reserve Cut" and "Level 63" share zero tokens - the generic
        // algorithm alone cannot find this pair.
        let candidates = [
            "24-97113 Level 63",
            "25-97101 Shanghai Tang",
            "25-97104 MAF MiCC v2",
        ];
        assert_eq!(
            resolve_successor_folder("24-97107 Reserve Cut", &candidates),
            Some("24-97113 Level 63")
        );
    }

    #[test]
    fn resolve_successor_maf_micc_v2_itself_is_unresolved_open_chain() {
        // 25-97104 MAF MiCC v2 is itself Superseded with no further
        // successor in this data (per D3 evidence) - must resolve to None,
        // not guess a wrong candidate.
        let candidates = [
            "24-97105 MAF MiCC Kids", // its own predecessor, not a valid successor
            "25-97101 Shanghai Tang",
            "24-97107 Reserve Cut",
            "24-97113 Level 63",
            "26-97401 Lusail Plaza",
        ];
        assert_eq!(
            resolve_successor_folder("25-97104 MAF MiCC v2", &candidates),
            None
        );
    }

    #[test]
    fn resolve_successor_never_matches_backward_to_its_own_predecessor() {
        // 25-97104 "MAF MiCC v2" token-overlaps its OWN predecessor
        // 24-97105 "MAF MiCC Kids" at 0.667 (above threshold) - without the
        // chronological-forward filter this would wrongly resolve
        // backward. It must resolve to None here (no later candidate
        // shares tokens).
        let candidates = ["24-97105 MAF MiCC Kids"];
        assert_eq!(
            resolve_successor_folder("25-97104 MAF MiCC v2", &candidates),
            None
        );
    }

    #[test]
    fn resolve_successor_no_match_below_threshold_returns_none() {
        let candidates = ["26-97401 Lusail Plaza", "25-97102 WAMI"];
        assert_eq!(
            resolve_successor_folder("24-97107 Some Unrelated Project", &candidates),
            None
        );
    }

    // -- check_prod_apply_gate (--confirm-prod escape hatch) ---------------

    #[test]
    fn apply_gate_allows_dev_apply_without_confirm_prod() {
        assert!(check_prod_apply_gate("dev", true, false).is_ok());
    }

    #[test]
    fn apply_gate_allows_dev_dry_run() {
        assert!(check_prod_apply_gate("dev", false, false).is_ok());
    }

    #[test]
    fn apply_gate_allows_prod_dry_run_without_confirm_prod() {
        assert!(check_prod_apply_gate("prod", false, false).is_ok());
    }

    #[test]
    fn apply_gate_refuses_prod_apply_without_confirm_prod() {
        assert!(check_prod_apply_gate("prod", true, false).is_err());
    }

    #[test]
    fn apply_gate_allows_prod_apply_with_confirm_prod() {
        assert!(check_prod_apply_gate("prod", true, true).is_ok());
    }

    #[test]
    fn apply_gate_confirm_prod_alone_is_a_no_op_without_apply() {
        // --confirm-prod with no --apply changes nothing (still a dry run).
        assert!(check_prod_apply_gate("prod", false, true).is_ok());
    }

    // -- build_plan idempotency + cross-checks ---------------------------

    fn make_project(status: &str, stage: Option<&str>, outcome: Option<&str>) -> ProjectRow {
        ProjectRow {
            id: RecordId::new("projects", "test_key"),
            folder: "26-97999 Test Project".to_string(),
            name_short: "Test".to_string(),
            status: status.to_string(),
            stage: stage.map(str::to_string),
            outcome: outcome.map(str::to_string),
            successor: None,
        }
    }

    #[test]
    fn build_plan_flags_new_row_as_not_already_correct() {
        let project = make_project("RFP", None, None);
        let mapping = map_status("RFP").unwrap();
        let plan = build_plan(&project, &mapping, None, false, &[]);
        assert!(!plan.already_correct);
        assert_eq!(plan.target_stage, "RFP");
        assert_eq!(plan.target_outcome, None);
    }

    #[test]
    fn build_plan_idempotent_when_stage_and_outcome_already_match() {
        let project = make_project("Completed", Some("Completed"), Some("Won"));
        let mapping = map_status("Completed").unwrap();
        let plan = build_plan(&project, &mapping, None, false, &[]);
        assert!(plan.already_correct);
    }

    #[test]
    fn build_plan_not_idempotent_when_only_outcome_differs() {
        let project = make_project("Completed", Some("Completed"), Some("Lost"));
        let mapping = map_status("Completed").unwrap();
        let plan = build_plan(&project, &mapping, None, false, &[]);
        assert!(!plan.already_correct);
    }

    #[test]
    fn build_plan_flags_lost_with_accepted_fee_history() {
        let project = make_project("Lost", None, None);
        let mapping = map_status("Lost").unwrap();
        let plan = build_plan(
            &project,
            &mapping,
            None,
            false,
            &["Accepted".to_string()],
        );
        assert!(plan
            .flags
            .iter()
            .any(|f| f.starts_with("lost-with-design-shaped-fee-history")));
    }

    #[test]
    fn build_plan_does_not_flag_lost_with_only_rejected_fee() {
        let project = make_project("Lost", None, None);
        let mapping = map_status("Lost").unwrap();
        let plan = build_plan(
            &project,
            &mapping,
            None,
            false,
            &["Rejected".to_string()],
        );
        assert!(!plan
            .flags
            .iter()
            .any(|f| f.starts_with("lost-with-design-shaped-fee-history")));
    }

    #[test]
    fn build_plan_flags_won_outcome_with_zero_accepted_fee_rows() {
        let project = make_project("Completed", None, None);
        let mapping = map_status("Completed").unwrap();
        let plan = build_plan(&project, &mapping, None, false, &[]);
        assert!(plan
            .flags
            .iter()
            .any(|f| f.starts_with("won-zero-accepted-fee")));
    }

    #[test]
    fn build_plan_does_not_flag_won_outcome_when_accepted_fee_exists() {
        let project = make_project("Completed", None, None);
        let mapping = map_status("Completed").unwrap();
        let plan = build_plan(
            &project,
            &mapping,
            None,
            false,
            &["Accepted".to_string()],
        );
        assert!(!plan
            .flags
            .iter()
            .any(|f| f.starts_with("won-zero-accepted-fee")));
    }

    #[test]
    fn build_plan_flags_unresolved_superseded_successor_as_open_chain() {
        let project = make_project("Superseded", None, None);
        let mapping = map_status("Superseded").unwrap();
        let plan = build_plan(&project, &mapping, None, true, &[]);
        assert!(plan
            .flags
            .iter()
            .any(|f| f.starts_with("successor-unresolved")));
    }

    #[test]
    fn build_plan_does_not_flag_resolved_superseded_successor() {
        let project = make_project("Superseded", None, None);
        let mapping = map_status("Superseded").unwrap();
        let plan = build_plan(
            &project,
            &mapping,
            Some("97113".to_string()),
            false,
            &[],
        );
        assert!(!plan
            .flags
            .iter()
            .any(|f| f.starts_with("successor-unresolved")));
    }
}
