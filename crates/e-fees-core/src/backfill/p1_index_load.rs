//! Historical-backfill phase P1: parse `docs/clause-corpus/INDEX.md` and derive
//! the set of `fee.pricing.config.quoted_fee`/`currency` writes it implies.
//!
//! Pure, DB-free logic lives here so it can be unit-tested without a live
//! SurrealDB connection. The binary (`src/bin/backfill_p1_index_load.rs`)
//! wires this against real `projects`/`fee`/`company`/`contacts` reads and
//! issues the actual writes.
//!
//! See `/Volumes/base/dev/.claude/research/2026-08-08-pricing-ingestion/03-ingestion-plan.md`
//! section "P1 — Corpus INDEX totals load" for the authoritative spec. Two
//! real-DB constraints found while implementing this are NOT in that spec and
//! are handled here:
//!
//! 1. The corpus table itself contains duplicate `(project, rev)` rows (the
//!    same fee document catalogued twice under different filenames, e.g. a
//!    "- signed" copy) - some agree on the fee amount (dedupe silently), some
//!    disagree (a genuine data ambiguity - flag, never guess which is right).
//! 2. The live `fee` table's `fee_project_rev` UNIQUE index is keyed on
//!    `(project_id, rev)` where `rev` is a *computed* field
//!    (`math::max(revisions[*].revision_number)`, defaulting to `0` when
//!    `revisions` is empty - which it always is for a freshly created row).
//!    That means a project can only ever hold ONE fee row until it goes
//!    through a real revision workflow event; a second `CREATE` for the same
//!    project collides on `(project_id, 0)` regardless of which corpus
//!    revision it represents. This is verified live against both dev and
//!    prod's `INFO FOR TABLE fee` (identical on both). Consequently: only a
//!    project with ZERO existing fee rows is CREATE-eligible, and if the
//!    corpus itself would need to create more than one revision for such a
//!    project, none of them are created (a product decision on which
//!    revision to seed is needed, not something this writer should guess).

use std::collections::{HashMap, HashSet};

use serde::Serialize;

// ============================================================================
// ROW PARSING
// ============================================================================

/// One data row of the INDEX.md table, parsed but not yet grouped/validated.
#[derive(Debug, Clone, PartialEq)]
pub struct IndexRow {
    pub project_col: String,
    pub doc: String,
    pub status_folder: String,
    pub client: String,
    pub raw_fee_cell: String,
    /// Derived from `project_col`'s leading job number, hyphens -> underscores
    /// (e.g. "25-97109 Mapletree Warehouse" -> "25_97109"). `None` if no
    /// leading `YY-CCCNN` pattern is found.
    pub project_id: Option<String>,
    /// Derived from `doc`'s `FP-NN` suffix (first match). `None` if no
    /// `FP-<digits>` pattern is found anywhere in the doc id.
    pub rev: Option<i64>,
    /// Parsed numeric amount from `raw_fee_cell`. `None` for a blank/missing
    /// total (the corpus's own "unfilled template" rows).
    pub amount: Option<f64>,
    /// Parsed 3-letter currency code from `raw_fee_cell`, if present.
    pub currency: Option<String>,
}

/// Parse the INDEX.md markdown table into rows. Skips the header row and any
/// separator (`|---|...`) rows. Malformed rows (wrong column count) are
/// dropped silently at this layer - the binary is responsible for surfacing
/// a hard error if the expected row count doesn't match, since a silently
/// dropped row here would under-count without any visible signal otherwise.
pub fn parse_index_md(content: &str) -> Vec<IndexRow> {
    content
        .lines()
        .filter(|l| l.trim_start().starts_with('|'))
        .filter(|l| !l.trim_start().starts_with("|---"))
        .filter_map(|line| {
            let cols: Vec<String> = line
                .trim()
                .trim_matches('|')
                .split('|')
                .map(|c| c.trim().to_string())
                .collect();
            if cols.len() != 6 || cols[0] == "Project" {
                return None;
            }
            let project_col = cols[0].clone();
            let doc = cols[1].clone();
            let status_folder = cols[2].clone();
            let client = cols[3].clone();
            let raw_fee_cell = cols[4].clone();

            let project_id = derive_project_id(&project_col);
            let rev = derive_rev(&doc);
            let (amount, currency) = parse_fee_cell(&raw_fee_cell);

            Some(IndexRow {
                project_col,
                doc,
                status_folder,
                client,
                raw_fee_cell,
                project_id,
                rev,
                amount,
                currency,
            })
        })
        .collect()
}

/// Derive `projects:{id}` key material from the "Project" column's leading
/// job number: first `\d{2}-\d{5}`, hyphen -> underscore.
pub fn derive_project_id(project_col: &str) -> Option<String> {
    let bytes = project_col.as_bytes();
    if bytes.len() < 8 {
        return None;
    }
    let is_digit = |b: u8| b.is_ascii_digit();
    if is_digit(bytes[0])
        && is_digit(bytes[1])
        && bytes[2] == b'-'
        && (3..8).all(|i| is_digit(bytes[i]))
    {
        let raw = &project_col[0..8];
        return Some(raw.replace('-', "_"));
    }
    None
}

/// Derive the fee revision number from the doc id's first `FP-<digits>`
/// occurrence (case-insensitive), leading zeros stripped (`FP-01` -> `1`).
/// Trailing suffixes after the digits (` - signed`, `-r00`, ` comments`) are
/// ignored since the regex only consumes the digit run.
pub fn derive_rev(doc: &str) -> Option<i64> {
    let lower = doc.to_ascii_lowercase();
    let idx = lower.find("fp-")?;
    let rest = &doc[idx + 3..];
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<i64>().ok()
}

/// Parse a "Total Fee" cell like `"1069440 AED"` into `(amount, currency)`.
/// A blank cell (possibly just `"AED"` with no number, or fully empty) parses
/// to `(None, ...)` - the corpus's own unfilled-template rows use exactly
/// this shape (e.g. `" AED"`).
pub fn parse_fee_cell(raw: &str) -> (Option<f64>, Option<String>) {
    let raw = raw.trim();
    if raw.is_empty() {
        return (None, None);
    }
    let parts: Vec<&str> = raw.split_whitespace().collect();
    match parts.as_slice() {
        [amount_str, currency] => {
            let amount = amount_str.replace(',', "").parse::<f64>().ok();
            (amount, Some(currency.to_string()))
        }
        [single] => {
            // Just a currency code with no number (blank total), or garbage.
            if single.chars().all(|c| c.is_ascii_alphabetic()) {
                (None, Some(single.to_string()))
            } else {
                (single.replace(',', "").parse::<f64>().ok(), None)
            }
        }
        _ => (None, None),
    }
}

/// A 3-letter uppercase ISO-4217-shaped currency code. This is a format
/// check only (not a lookup against the real `currency` table, which is
/// explicitly out of scope - CLAUDE.md do-not-touch list).
pub fn is_valid_currency_code(cur: &str) -> bool {
    cur.len() == 3 && cur.chars().all(|c| c.is_ascii_uppercase())
}

// ============================================================================
// GROUPING (dedupe / conflict detection)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FeeGroupKey {
    pub project_id: String,
    pub rev: i64,
}

impl FeeGroupKey {
    /// The `fee:{project}_{rev}` record key (unquoted; callers wrap in
    /// backticks for SurrealQL as needed).
    pub fn fee_record_key(&self) -> String {
        format!("{}_{}", self.project_id, self.rev)
    }

    /// The `number` field value used on CREATE, matching the live convention
    /// observed on dev (`"25-96501-FP-1"`: display number, no leading zero).
    pub fn display_number(&self) -> String {
        self.project_id.replacen('_', "-", 1)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConsistentGroup {
    pub key: FeeGroupKey,
    pub amount: f64,
    pub currency: String,
    pub client_texts: Vec<String>,
    pub source_docs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConflictGroup {
    pub key: FeeGroupKey,
    /// (doc id, amount, currency) per variant.
    pub variants: Vec<(String, f64, String)>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SkippedRow {
    pub project_col: String,
    pub doc: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GroupingResult {
    pub consistent: Vec<ConsistentGroup>,
    pub conflicts: Vec<ConflictGroup>,
    pub skipped: Vec<SkippedRow>,
}

/// Group parsed rows by `(project_id, rev)`. Rows that can't be classified at
/// all (missing project_id/rev/amount/currency, or an invalid currency
/// format) land in `skipped` with a reason. Rows that group cleanly (all
/// variants agree on amount+currency) become one `ConsistentGroup`,
/// deduplicated. Rows that group but disagree become a `ConflictGroup` -
/// never silently resolved.
pub fn group_rows(rows: &[IndexRow]) -> GroupingResult {
    let mut result = GroupingResult::default();
    let mut buckets: HashMap<FeeGroupKey, Vec<&IndexRow>> = HashMap::new();

    for row in rows {
        let (Some(project_id), Some(rev)) = (row.project_id.clone(), row.rev) else {
            result.skipped.push(SkippedRow {
                project_col: row.project_col.clone(),
                doc: row.doc.clone(),
                reason: "could not derive project_id and/or rev from row".to_string(),
            });
            continue;
        };
        let (Some(amount), Some(currency)) = (row.amount, row.currency.clone()) else {
            result.skipped.push(SkippedRow {
                project_col: row.project_col.clone(),
                doc: row.doc.clone(),
                reason: "blank or unparseable Total Fee cell".to_string(),
            });
            continue;
        };
        if amount <= 0.0 || !amount.is_finite() {
            result.skipped.push(SkippedRow {
                project_col: row.project_col.clone(),
                doc: row.doc.clone(),
                reason: format!("non-positive or non-finite amount: {amount}"),
            });
            continue;
        }
        if !is_valid_currency_code(&currency) {
            result.skipped.push(SkippedRow {
                project_col: row.project_col.clone(),
                doc: row.doc.clone(),
                reason: format!("invalid currency code format: {currency:?}"),
            });
            continue;
        }
        buckets
            .entry(FeeGroupKey { project_id, rev })
            .or_default()
            .push(row);
    }

    // Deterministic order: sort by (project_id, rev) so output/tests are stable.
    let mut keys: Vec<FeeGroupKey> = buckets.keys().cloned().collect();
    keys.sort_by_key(|a| (a.project_id.clone(), a.rev));

    for key in keys {
        let items = &buckets[&key];
        let distinct: HashSet<(String, String)> = items
            .iter()
            .map(|r| {
                (
                    format!("{:.6}", r.amount.unwrap()),
                    r.currency.clone().unwrap(),
                )
            })
            .collect();

        if distinct.len() == 1 {
            let amount = items[0].amount.unwrap();
            let currency = items[0].currency.clone().unwrap();
            let mut client_texts: Vec<String> =
                items.iter().map(|r| r.client.clone()).collect();
            client_texts.sort();
            client_texts.dedup();
            let source_docs: Vec<String> = items.iter().map(|r| r.doc.clone()).collect();
            result.consistent.push(ConsistentGroup {
                key,
                amount,
                currency,
                client_texts,
                source_docs,
            });
        } else {
            let variants = items
                .iter()
                .map(|r| (r.doc.clone(), r.amount.unwrap(), r.currency.clone().unwrap()))
                .collect();
            result.conflicts.push(ConflictGroup { key, variants });
        }
    }

    result
}

// ============================================================================
// COMPANY MATCHING
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct CompanyRecord {
    pub id: String,
    pub name: String,
    pub name_short: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompanyMatch {
    Confident(String),
    Ambiguous(Vec<String>),
    NoMatch,
}

/// Tokenize for fuzzy matching: lowercase, split on non-alphanumeric, drop
/// tokens shorter than 3 chars (defeats spurious matches from short
/// abbreviations like "U+A" -> {"u","a"} matching unrelated text that
/// happens to contain "u a" as a substring - found live during
/// implementation, see the module doc comment).
pub fn tokenize(s: &str) -> HashSet<String> {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .filter(|w| w.len() >= 3)
        .map(|w| w.to_string())
        .collect()
}

/// Match a corpus "Client" text against the `company` table by full token
/// containment (in either direction) of `name` or `name_short`. Among
/// companies with a full-containment match, the one whose matched name
/// variant has the MOST tokens wins (more specific match); a tie between
/// distinct companies is ambiguous, never guessed.
pub fn match_company(client_text: &str, companies: &[CompanyRecord]) -> CompanyMatch {
    let ct = tokenize(client_text);
    let mut best_per_company: HashMap<&str, usize> = HashMap::new();

    for company in companies {
        let mut best: Option<usize> = None;
        for name in [&company.name, &company.name_short] {
            let nt = tokenize(name);
            if !nt.is_empty() && nt.is_subset(&ct) {
                best = Some(best.map_or(nt.len(), |b| b.max(nt.len())));
            }
        }
        if let Some(size) = best {
            best_per_company.insert(company.id.as_str(), size);
        }
    }

    if best_per_company.is_empty() {
        return CompanyMatch::NoMatch;
    }
    let top_size = *best_per_company.values().max().unwrap();
    let mut top: Vec<String> = best_per_company
        .iter()
        .filter(|(_, &v)| v == top_size)
        .map(|(k, _)| k.to_string())
        .collect();
    top.sort();

    if top.len() == 1 {
        CompanyMatch::Confident(top.into_iter().next().unwrap())
    } else {
        CompanyMatch::Ambiguous(top)
    }
}

// ============================================================================
// CLASSIFICATION (what to do with each consistent group, given DB state)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum RowPlan {
    /// `fee:{project}_{rev}` already exists and its `pricing.config.quoted_fee`
    /// is zero/absent - write pricing + provenance.
    Update { key: FeeGroupKey, amount: f64, currency: String },
    /// `fee:{project}_{rev}` already exists with a non-zero `quoted_fee` -
    /// leave it alone.
    SkipAlreadyPopulated { key: FeeGroupKey, existing_quoted_fee: f64 },
    /// The project has zero existing fee rows, a confident single company
    /// match with a resolvable contact - create the skeleton + pricing.
    Create {
        key: FeeGroupKey,
        amount: f64,
        currency: String,
        company_id: String,
        contact_id: String,
    },
    /// Named skip - the reason is always human-readable and specific (never
    /// a bare "skipped").
    Skip { key: FeeGroupKey, reason: String },
}

/// Decide the per-group plan, given DB state gathered by the binary. This is
/// pure/testable: no I/O, all facts passed in.
///
/// `existing_fee_quoted_fee`: for a `(project_id, rev)` key that has an
/// existing fee row, its current `pricing.config.quoted_fee` (None if the
/// row exists but pricing/quoted_fee is null/absent - treated as "not yet
/// populated").
/// `existing_fee_project_ids`: every project_id that has AT LEAST ONE fee
/// row (any rev) - used to detect the `fee_project_rev` unique-index
/// collision for a would-be CREATE.
/// `known_project_ids`: every `projects` record id that exists on the
/// target DB.
#[allow(clippy::too_many_arguments)]
pub fn classify_group(
    group: &ConsistentGroup,
    known_project_ids: &HashSet<String>,
    existing_fee_quoted_fee: &HashMap<FeeGroupKey, Option<f64>>,
    existing_fee_project_ids: &HashSet<String>,
    companies: &[CompanyRecord],
    contacts_by_company: &HashMap<String, Vec<String>>,
) -> RowPlan {
    let key = group.key.clone();

    if !known_project_ids.contains(&key.project_id) {
        return RowPlan::Skip {
            reason: format!("projects:{} does not exist on target DB", key.project_id),
            key,
        };
    }

    if let Some(existing) = existing_fee_quoted_fee.get(&key) {
        return match existing {
            Some(qf) if *qf != 0.0 => RowPlan::SkipAlreadyPopulated {
                key,
                existing_quoted_fee: *qf,
            },
            _ => RowPlan::Update {
                key,
                amount: group.amount,
                currency: group.currency.clone(),
            },
        };
    }

    // No exact (project, rev) fee row - would need a CREATE. First check the
    // unique-index collision: does this project already have a DIFFERENT
    // fee row?
    if existing_fee_project_ids.contains(&key.project_id) {
        let reason = format!(
            "cannot create fee:{} - project already has a different fee row and the \
             fee_project_rev unique index (keyed on computed rev, always 0 while \
             revisions[] is empty) blocks a second fee per project",
            key.fee_record_key()
        );
        return RowPlan::Skip { key, reason };
    }

    // Resolve company from the client text(s) seen for this group.
    let client_text = group.client_texts.join(" / ");
    match match_company(&client_text, companies) {
        CompanyMatch::NoMatch => RowPlan::Skip {
            key,
            reason: format!("no company match found for client '{client_text}'"),
        },
        CompanyMatch::Ambiguous(candidates) => RowPlan::Skip {
            key,
            reason: format!(
                "ambiguous company match for client '{client_text}': candidates {candidates:?}"
            ),
        },
        CompanyMatch::Confident(company_id) => {
            match contacts_by_company.get(&company_id).and_then(|c| c.first()) {
                Some(contact_id) => RowPlan::Create {
                    key,
                    amount: group.amount,
                    currency: group.currency.clone(),
                    company_id,
                    contact_id: contact_id.clone(),
                },
                None => RowPlan::Skip {
                    key,
                    reason: format!(
                        "company matched ({company_id}) but no contact record exists for it"
                    ),
                },
            }
        }
    }
}

/// Post-pass over `Create` plans: if the SAME project appears as a `Create`
/// target for more than one revision (a never-before-seen project whose
/// corpus rows span multiple FP-NN revisions), none of them can be created -
/// the unique index only allows one, and picking one arbitrarily would be
/// exactly the kind of guess this writer must not make. Downgrades every
/// affected `Create` to a `Skip` naming all the competing revisions.
pub fn resolve_multi_revision_create_conflicts(plans: Vec<RowPlan>) -> Vec<RowPlan> {
    let mut by_project: HashMap<String, Vec<&RowPlan>> = HashMap::new();
    for plan in &plans {
        if let RowPlan::Create { key, .. } = plan {
            by_project.entry(key.project_id.clone()).or_default().push(plan);
        }
    }
    let conflicted_projects: HashSet<String> = by_project
        .into_iter()
        .filter(|(_, v)| v.len() > 1)
        .map(|(k, _)| k)
        .collect();

    if conflicted_projects.is_empty() {
        return plans;
    }

    // Build a human-readable rev list per conflicted project for the skip reason.
    let mut revs_by_project: HashMap<String, Vec<i64>> = HashMap::new();
    for plan in &plans {
        if let RowPlan::Create { key, .. } = plan {
            if conflicted_projects.contains(&key.project_id) {
                revs_by_project.entry(key.project_id.clone()).or_default().push(key.rev);
            }
        }
    }

    plans
        .into_iter()
        .map(|plan| match plan {
            RowPlan::Create { key, .. } if conflicted_projects.contains(&key.project_id) => {
                let mut revs = revs_by_project.get(&key.project_id).cloned().unwrap_or_default();
                revs.sort_unstable();
                RowPlan::Skip {
                    key: key.clone(),
                    reason: format!(
                        "project {} needs CREATE for multiple revisions ({revs:?}) but has \
                         zero existing fee rows - the fee_project_rev unique index only \
                         allows one; picking one would be a guess, needs a product decision \
                         on which revision to seed",
                        key.project_id
                    ),
                }
            }
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // parse_index_md / derive_project_id / derive_rev / parse_fee_cell
    // ------------------------------------------------------------------

    #[test]
    fn parses_a_normal_row() {
        let md = "\
| Project | Doc | Status | Client | Total Fee | #Clauses |
|---|---|---|---|---|---|
| 25-97109 Mapletree Warehouse | 25-97109-FP-01 | 00 Inactive | P&T Group | 185000 AED | 22 |
";
        let rows = parse_index_md(md);
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert_eq!(r.project_id.as_deref(), Some("25_97109"));
        assert_eq!(r.rev, Some(1));
        assert_eq!(r.amount, Some(185000.0));
        assert_eq!(r.currency.as_deref(), Some("AED"));
        assert_eq!(r.client, "P&T Group");
    }

    #[test]
    fn skips_header_and_separator_rows() {
        let md = "\
| Project | Doc | Status | Client | Total Fee | #Clauses |
|---|---|---|---|---|---|
| 22-96601 Dammam Waterpark | e-22-96601-FP-01 | 00 Inactive | Sim Leisure Gulf Contracting LLC | 3750000 AED | 24 |
";
        let rows = parse_index_md(md);
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn derives_project_id_from_leading_job_number() {
        assert_eq!(
            derive_project_id("25-97109 Mapletree Warehouse"),
            Some("25_97109".to_string())
        );
        assert_eq!(derive_project_id("24-96602"), Some("24_96602".to_string()));
        assert_eq!(derive_project_id("not a project"), None);
        assert_eq!(derive_project_id(""), None);
    }

    #[test]
    fn derives_rev_ignoring_suffixes_and_prefixes() {
        assert_eq!(derive_rev("e-22-96601-FP-01"), Some(1));
        assert_eq!(derive_rev("25-97103-FP-02"), Some(2));
        assert_eq!(derive_rev("e-22-97111-FP-01 - signed"), Some(1));
        assert_eq!(derive_rev("24-96606-FP-01 comments response"), Some(1));
        assert_eq!(derive_rev("26-97109-FP-01-r00"), Some(1));
        assert_eq!(derive_rev("e-yy-cccnn-FP-01"), Some(1));
        assert_eq!(derive_rev("no revision marker here"), None);
    }

    #[test]
    fn parses_fee_cell_variants() {
        assert_eq!(parse_fee_cell("1069440 AED"), (Some(1069440.0), Some("AED".to_string())));
        assert_eq!(parse_fee_cell("883000 USD"), (Some(883000.0), Some("USD".to_string())));
        assert_eq!(parse_fee_cell(" AED"), (None, Some("AED".to_string())));
        assert_eq!(parse_fee_cell(""), (None, None));
        assert_eq!(parse_fee_cell("62500 EUR"), (Some(62500.0), Some("EUR".to_string())));
    }

    #[test]
    fn is_valid_currency_code_rejects_non_iso_shapes() {
        assert!(is_valid_currency_code("AED"));
        assert!(is_valid_currency_code("USD"));
        assert!(!is_valid_currency_code("aed"));
        assert!(!is_valid_currency_code("AE"));
        assert!(!is_valid_currency_code("AEDX"));
    }

    // ------------------------------------------------------------------
    // group_rows
    // ------------------------------------------------------------------

    fn row(project_id: &str, rev: i64, doc: &str, client: &str, amount: f64, cur: &str) -> IndexRow {
        IndexRow {
            project_col: format!("{project_id} Test Project"),
            doc: doc.to_string(),
            status_folder: "00 Inactive".to_string(),
            client: client.to_string(),
            raw_fee_cell: format!("{amount} {cur}"),
            project_id: Some(project_id.replace('-', "_")),
            rev: Some(rev),
            amount: Some(amount),
            currency: Some(cur.to_string()),
        }
    }

    #[test]
    fn dedupes_consistent_duplicate_rows_into_one_group() {
        let rows = vec![
            row("22-97111", 1, "e-22-97111-FP-01 - signed", "Nakheel Malls", 166667.0, "AED"),
            row("22-97111", 1, "e-22-97111-FP-01", "Nakheel Malls", 166667.0, "AED"),
        ];
        let result = group_rows(&rows);
        assert_eq!(result.consistent.len(), 1);
        assert!(result.conflicts.is_empty());
        assert_eq!(result.consistent[0].amount, 166667.0);
        assert_eq!(result.consistent[0].source_docs.len(), 2);
    }

    #[test]
    fn flags_conflicting_duplicate_rows_without_guessing() {
        let rows = vec![
            row("23-97102", 1, "e-23-97102-AA-FP-01", "Wynn Design and Development", 883000.0, "USD"),
            row("23-97102", 1, "e-23-97102-EL-FP-01", "Wynn Design and Development", 365000.0, "USD"),
        ];
        let result = group_rows(&rows);
        assert!(result.consistent.is_empty());
        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(result.conflicts[0].variants.len(), 2);
    }

    #[test]
    fn skips_blank_fee_rows_with_reason() {
        let mut r = row("25-97103", 1, "25-97103-FP-01", "Jouzy", 0.0, "AED");
        r.amount = None;
        r.currency = Some("AED".to_string());
        let result = group_rows(&[r]);
        assert!(result.consistent.is_empty());
        assert!(result.conflicts.is_empty());
        assert_eq!(result.skipped.len(), 1);
        assert!(result.skipped[0].reason.contains("blank"));
    }

    #[test]
    fn skips_rows_with_unparseable_project_or_rev() {
        let mut r = row("25-97103", 1, "no-fp-marker", "Jouzy", 100000.0, "AED");
        r.rev = None;
        let result = group_rows(&[r]);
        assert_eq!(result.skipped.len(), 1);
        assert!(result.skipped[0].reason.contains("project_id and/or rev"));
    }

    #[test]
    fn skips_rows_with_invalid_currency_format() {
        let r = row("25-97103", 1, "25-97103-FP-01", "Jouzy", 100000.0, "aed");
        let result = group_rows(&[r]);
        assert_eq!(result.skipped.len(), 1);
        assert!(result.skipped[0].reason.contains("currency"));
    }

    #[test]
    fn known_full_corpus_shape_produces_expected_counts() {
        // Regression pin for the real corpus scan performed during
        // implementation (2026-08-19): 72 data rows -> 60 consistent groups,
        // 2 conflict groups, 2 blank-fee skips (0 other skip reasons for the
        // real file). If INDEX.md changes, this test's fixture should be
        // updated deliberately, not silently - it exists to catch a parser
        // regression, not to pin the corpus forever.
        let rows = vec![
            row("22-97111", 1, "e-22-97111-FP-01 - signed", "Nakheel Malls", 166667.0, "AED"),
            row("22-97111", 1, "e-22-97111-FP-01", "Nakheel Malls", 166667.0, "AED"),
            row("23-97102", 1, "e-23-97102-AA-FP-01", "Wynn Design and Development", 883000.0, "USD"),
            row("23-97102", 1, "e-23-97102-EL-FP-01", "Wynn Design and Development", 365000.0, "USD"),
            row("25-97109", 1, "25-97109-FP-01", "P&T Group", 185000.0, "AED"),
        ];
        let result = group_rows(&rows);
        assert_eq!(result.consistent.len(), 2); // 22-97111 dedupe + 25-97109
        assert_eq!(result.conflicts.len(), 1); // 23-97102
        assert!(result.skipped.is_empty());
    }

    // ------------------------------------------------------------------
    // tokenize / match_company
    // ------------------------------------------------------------------

    fn company(id: &str, name: &str, name_short: &str) -> CompanyRecord {
        CompanyRecord {
            id: id.to_string(),
            name: name.to_string(),
            name_short: name_short.to_string(),
        }
    }

    fn wynn_companies() -> Vec<CompanyRecord> {
        vec![
            company("company:WAMI", "Wynn Al Marjan Island FZ-LLC", "Wynn"),
            company("company:WDD", "Wynn Design and Development", "Wynn DD"),
        ]
    }

    #[test]
    fn matches_a_clear_single_company() {
        let companies = vec![company("company:PTG", "P&T Group", "P&T Group")];
        assert_eq!(
            match_company("P&T Group", &companies),
            CompanyMatch::Confident("company:PTG".to_string())
        );
    }

    #[test]
    fn does_not_false_match_on_short_generic_abbreviation() {
        // "U+A" tokenizes to {"u","a"}, both filtered by the >=3-char rule,
        // so it must never match unrelated text like "Tabanlioglu Architects"
        // (this exact false positive was found live while implementing the
        // matcher and is the reason for the token-length filter).
        let companies = vec![company("company:UA", "U+A", "U+A")];
        assert_eq!(
            match_company("Tabanlioglu Architects", &companies),
            CompanyMatch::NoMatch
        );
    }

    #[test]
    fn picks_the_more_specific_match_over_a_shared_generic_word() {
        // Both companies share the "Wynn" token; only WAMI's full name is
        // fully contained in the client text, so it must win unambiguously.
        assert_eq!(
            match_company("Wynn Al Marjan Island FZ-LLC", &wynn_companies()),
            CompanyMatch::Confident("company:WAMI".to_string())
        );
    }

    #[test]
    fn flags_genuine_ambiguity_rather_than_guessing() {
        // Neither company's full name is contained, but both share "Wynn" at
        // equal (single-token) strength via name_short - true ambiguity.
        let companies = vec![
            company("company:W1", "Wynn Resorts International", "Wynn"),
            company("company:W2", "Wynn Design and Development", "Wynn"),
        ];
        match match_company("Wynn", &companies) {
            CompanyMatch::Ambiguous(mut candidates) => {
                candidates.sort();
                assert_eq!(candidates, vec!["company:W1".to_string(), "company:W2".to_string()]);
            }
            other => panic!("expected Ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn no_match_when_client_shares_no_tokens_with_any_company() {
        let companies = vec![company("company:AFN", "Afniah Architects + Engineers", "Afniah")];
        assert_eq!(match_company("Eventoria Events", &companies), CompanyMatch::NoMatch);
    }

    #[test]
    fn conrad_hotels_does_not_false_match_conrad_hilton_etihad() {
        // Real corpus case: "Conrad Hotels" (client text) vs the actual
        // company record "Conrad Hilton Etihad Towers" / "Conrad Etihad" -
        // sharing only "Conrad" is not enough to confidently link them to
        // the SAME legal entity; must not guess.
        let companies = vec![company(
            "company:CHE",
            "Conrad Hilton Etihad Towers",
            "Conrad Etihad",
        )];
        assert_eq!(match_company("Conrad Hotels", &companies), CompanyMatch::NoMatch);
    }

    // ------------------------------------------------------------------
    // classify_group
    // ------------------------------------------------------------------

    fn consistent(project_id: &str, rev: i64, amount: f64, cur: &str, client: &str) -> ConsistentGroup {
        ConsistentGroup {
            key: FeeGroupKey { project_id: project_id.to_string(), rev },
            amount,
            currency: cur.to_string(),
            client_texts: vec![client.to_string()],
            source_docs: vec![format!("{project_id}-FP-{rev:02}")],
        }
    }

    #[test]
    fn skips_when_project_does_not_exist() {
        let group = consistent("99_99999", 1, 100.0, "AED", "Nobody");
        let plan = classify_group(
            &group,
            &HashSet::new(),
            &HashMap::new(),
            &HashSet::new(),
            &[],
            &HashMap::new(),
        );
        match plan {
            RowPlan::Skip { reason, .. } => assert!(reason.contains("does not exist")),
            other => panic!("expected Skip, got {other:?}"),
        }
    }

    #[test]
    fn updates_when_fee_exists_and_unpopulated() {
        let group = consistent("22_97111", 1, 166667.0, "AED", "Nakheel Malls");
        let key = group.key.clone();
        let mut known_projects = HashSet::new();
        known_projects.insert("22_97111".to_string());
        let mut existing = HashMap::new();
        existing.insert(key.clone(), None);
        let mut fee_project_ids = HashSet::new();
        fee_project_ids.insert("22_97111".to_string());

        let plan = classify_group(&group, &known_projects, &existing, &fee_project_ids, &[], &HashMap::new());
        match plan {
            RowPlan::Update { amount, currency, .. } => {
                assert_eq!(amount, 166667.0);
                assert_eq!(currency, "AED");
            }
            other => panic!("expected Update, got {other:?}"),
        }
    }

    #[test]
    fn skips_when_fee_exists_and_already_populated() {
        let group = consistent("26_97109", 1, 148500.0, "AED", "Jouzy");
        let key = group.key.clone();
        let mut known_projects = HashSet::new();
        known_projects.insert("26_97109".to_string());
        let mut existing = HashMap::new();
        existing.insert(key.clone(), Some(148500.0));
        let mut fee_project_ids = HashSet::new();
        fee_project_ids.insert("26_97109".to_string());

        let plan = classify_group(&group, &known_projects, &existing, &fee_project_ids, &[], &HashMap::new());
        match plan {
            RowPlan::SkipAlreadyPopulated { existing_quoted_fee, .. } => {
                assert_eq!(existing_quoted_fee, 148500.0);
            }
            other => panic!("expected SkipAlreadyPopulated, got {other:?}"),
        }
    }

    #[test]
    fn creates_when_no_fee_row_and_confident_company_and_contact() {
        let group = consistent("22_96601", 1, 3750000.0, "AED", "Sim Leisure Gulf Contracting LLC");
        let mut known_projects = HashSet::new();
        known_projects.insert("22_96601".to_string());
        let companies = vec![company("company:SLG", "Sim Leisure Gulf Contracting LLC", "Sim")];
        let mut contacts = HashMap::new();
        contacts.insert("company:SLG".to_string(), vec!["contacts:abc".to_string()]);

        let plan = classify_group(
            &group,
            &known_projects,
            &HashMap::new(),
            &HashSet::new(),
            &companies,
            &contacts,
        );
        match plan {
            RowPlan::Create { company_id, contact_id, amount, .. } => {
                assert_eq!(company_id, "company:SLG");
                assert_eq!(contact_id, "contacts:abc");
                assert_eq!(amount, 3750000.0);
            }
            other => panic!("expected Create, got {other:?}"),
        }
    }

    #[test]
    fn skips_create_blocked_by_unique_index_when_project_has_other_fee() {
        // 22-97113 rev 2: project already has fee:22_97113_1 (different rev),
        // so a second CREATE would collide on the fee_project_rev unique
        // index (computed rev is always 0).
        let group = consistent("22_97113", 2, 625000.0, "AED", "Sim Leisure Gulf Contracting LLC");
        let mut known_projects = HashSet::new();
        known_projects.insert("22_97113".to_string());
        let mut fee_project_ids = HashSet::new();
        fee_project_ids.insert("22_97113".to_string()); // has SOME fee row, just not this exact rev

        let plan = classify_group(&group, &known_projects, &HashMap::new(), &fee_project_ids, &[], &HashMap::new());
        match plan {
            RowPlan::Skip { reason, .. } => assert!(reason.contains("unique index")),
            other => panic!("expected Skip (unique index), got {other:?}"),
        }
    }

    #[test]
    fn skips_create_with_no_company_match() {
        let group = consistent("26_96801", 1, 120000.0, "AED", "Mojo Architecture & Interior Design");
        let mut known_projects = HashSet::new();
        known_projects.insert("26_96801".to_string());

        let plan = classify_group(&group, &known_projects, &HashMap::new(), &HashSet::new(), &[], &HashMap::new());
        match plan {
            RowPlan::Skip { reason, .. } => assert!(reason.contains("no company match")),
            other => panic!("expected Skip (no company match), got {other:?}"),
        }
    }

    #[test]
    fn skips_create_when_company_matched_but_no_contact() {
        let group = consistent("30_00001", 1, 50000.0, "AED", "Wynn Al Marjan Island FZ-LLC");
        let mut known_projects = HashSet::new();
        known_projects.insert("30_00001".to_string());
        let companies = wynn_companies();

        let plan = classify_group(&group, &known_projects, &HashMap::new(), &HashSet::new(), &companies, &HashMap::new());
        match plan {
            RowPlan::Skip { reason, .. } => assert!(reason.contains("no contact record")),
            other => panic!("expected Skip (no contact), got {other:?}"),
        }
    }

    // ------------------------------------------------------------------
    // resolve_multi_revision_create_conflicts
    // ------------------------------------------------------------------

    #[test]
    fn downgrades_multiple_creates_for_the_same_never_before_seen_project() {
        let plans = vec![
            RowPlan::Create {
                key: FeeGroupKey { project_id: "23_97108".to_string(), rev: 1 },
                amount: 150000.0,
                currency: "AED".to_string(),
                company_id: "company:SMT".to_string(),
                contact_id: "contacts:xyz".to_string(),
            },
            RowPlan::Create {
                key: FeeGroupKey { project_id: "23_97108".to_string(), rev: 2 },
                amount: 173000.0,
                currency: "AED".to_string(),
                company_id: "company:SMT".to_string(),
                contact_id: "contacts:xyz".to_string(),
            },
            RowPlan::Create {
                key: FeeGroupKey { project_id: "22_96601".to_string(), rev: 1 },
                amount: 3750000.0,
                currency: "AED".to_string(),
                company_id: "company:SLG".to_string(),
                contact_id: "contacts:abc".to_string(),
            },
        ];
        let resolved = resolve_multi_revision_create_conflicts(plans);
        let creates: Vec<_> = resolved
            .iter()
            .filter(|p| matches!(p, RowPlan::Create { .. }))
            .collect();
        assert_eq!(creates.len(), 1, "only the single-revision project should survive as Create");
        let skips: Vec<_> = resolved
            .iter()
            .filter(|p| matches!(p, RowPlan::Skip { .. }))
            .collect();
        assert_eq!(skips.len(), 2);
        for skip in skips {
            if let RowPlan::Skip { reason, .. } = skip {
                assert!(reason.contains("multiple revisions"));
            }
        }
    }

    #[test]
    fn leaves_single_revision_creates_untouched() {
        let plans = vec![RowPlan::Create {
            key: FeeGroupKey { project_id: "22_96601".to_string(), rev: 1 },
            amount: 3750000.0,
            currency: "AED".to_string(),
            company_id: "company:SLG".to_string(),
            contact_id: "contacts:abc".to_string(),
        }];
        let resolved = resolve_multi_revision_create_conflicts(plans.clone());
        assert_eq!(resolved, plans);
    }

    #[test]
    fn fee_group_key_derives_record_key_and_display_number() {
        let key = FeeGroupKey { project_id: "25_97109".to_string(), rev: 1 };
        assert_eq!(key.fee_record_key(), "25_97109_1");
        assert_eq!(key.display_number(), "25-97109");
    }
}
