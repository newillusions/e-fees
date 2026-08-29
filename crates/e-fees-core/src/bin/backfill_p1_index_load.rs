//! Historical-backfill P1: load `docs/clause-corpus/INDEX.md` totals into
//! `fee.pricing.config.quoted_fee`/`currency`. See
//! `crates/e-fees-core/src/backfill/p1_index_load.rs` for the pure parsing/
//! classification logic (unit-tested there) and its module doc comment for
//! the real-DB constraints the ingestion plan didn't anticipate (duplicate
//! corpus rows, the `fee_project_rev` unique-index collision, and the two
//! 2026-08-20 owner rulings this file implements: multi-revision seeding
//! carries the earlier amount forward in provenance instead of skipping
//! both, and CLIENT_ALIASES resolves owner-confirmed client identities the
//! fuzzy matcher correctly refuses to guess).
//!
//! Usage:
//!   cargo run -p e-fees-core --bin backfill_p1_index_load -- \
//!     --target dev [--apply] [--index-md PATH] [--report-file PATH]
//!
//! `--target prod --apply` is REFUSED unless `--confirm-prod` is also
//! passed, since prod writes are orchestrator-owned by default. `--target
//! prod` without `--apply` runs a read-only dry-run against prod, which is
//! the whole point of the prod pass: produce the exact plan for the
//! orchestrator to apply separately (or, when the orchestrator itself is
//! running this binary, `--apply --target prod --confirm-prod` applies it
//! directly).
//!
//! Default is `--dry-run` (i.e. omitting `--apply` never writes, regardless
//! of `--target`).
//!
//! Credentials: `EFEES_SURREALDB_USER` / `EFEES_SURREALDB_PASS` env vars
//! (same pair used by the P0 migration tests and the desktop app against
//! both dev and prod) - never read from a file directly, never printed.

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::process::ExitCode;

use serde::Serialize;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::types::{RecordId, SurrealValue};
use surrealdb::Surreal;

use e_fees_core::backfill::p1_index_load::{
    self, check_prod_apply_gate, CompanyRecord, ConflictGroup, ConsistentGroup, FeeGroupKey,
    RowPlan, SkippedRow, SupersededRevision,
};
use e_fees_core::models::{record_key_string, Revision};

struct TargetConfig {
    name: &'static str,
    host: &'static str,
    ns: &'static str,
    db: &'static str,
}

const DEV: TargetConfig = TargetConfig {
    name: "dev",
    host: "10.0.23.12:8000",
    ns: "emittiv_dev",
    db: "projects",
};
const PROD: TargetConfig = TargetConfig {
    name: "prod",
    host: "10.0.23.11:8000",
    ns: "emittiv",
    db: "projects",
};

struct Args {
    target: &'static TargetConfig,
    apply: bool,
    index_md_path: String,
    report_file: Option<String>,
}

fn parse_args() -> Result<Args, String> {
    let mut target: Option<&'static TargetConfig> = None;
    let mut apply = false;
    let mut confirm_prod = false;
    let mut index_md_path =
        "/Volumes/base/dev/claude/e-fees/docs/clause-corpus/INDEX.md".to_string();
    let mut report_file = None;

    let mut iter = env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--target" => {
                let v = iter.next().ok_or("--target requires a value (dev|prod)")?;
                target = Some(match v.as_str() {
                    "dev" => &DEV,
                    "prod" => &PROD,
                    other => return Err(format!("--target must be dev or prod, got {other:?}")),
                });
            }
            "--apply" => apply = true,
            "--dry-run" => apply = false,
            "--confirm-prod" => confirm_prod = true,
            "--index-md" => {
                index_md_path = iter.next().ok_or("--index-md requires a path")?;
            }
            "--report-file" => {
                report_file = Some(iter.next().ok_or("--report-file requires a path")?);
            }
            other => return Err(format!("unrecognized argument: {other}")),
        }
    }

    let target = target.ok_or("--target dev|prod is required")?;

    check_prod_apply_gate(target.name, apply, confirm_prod)?;

    Ok(Args {
        target,
        apply,
        index_md_path,
        report_file,
    })
}

#[derive(Serialize)]
struct RowReport {
    project_id: String,
    rev: i64,
    action: String,
    amount: Option<f64>,
    currency: Option<String>,
    reason: Option<String>,
    /// Non-empty only for a `create` row that won a multi-revision seeding
    /// choice (ruling: Martin, 2026-08-20) - the earlier revision(s) whose
    /// amount is carried forward in this row's `data_provenance` instead of
    /// being written as their own fee row.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    superseded_revisions: Vec<SupersededRevision>,
}

#[derive(Serialize)]
struct Report {
    target: String,
    apply: bool,
    counts: ReportCounts,
    created: Vec<RowReport>,
    updated: Vec<RowReport>,
    skipped_already_populated: Vec<RowReport>,
    skipped: Vec<RowReport>,
    conflicts: Vec<ConflictReport>,
    parse_skipped: Vec<SkippedRow>,
}

#[derive(Serialize)]
struct ConflictReport {
    project_id: String,
    rev: i64,
    variants: Vec<(String, f64, String)>,
}

#[derive(Serialize, Default)]
struct ReportCounts {
    total_index_rows: usize,
    consistent_groups: usize,
    conflict_groups: usize,
    parse_skipped_rows: usize,
    created: usize,
    updated: usize,
    skipped_already_populated: usize,
    skipped_other: usize,
}

#[derive(Debug, SurrealValue)]
struct ProjectIdRow {
    id: RecordId,
}

#[derive(Debug, SurrealValue)]
struct FeeRow {
    id: RecordId,
    qf: Option<f64>,
}

#[derive(Debug, SurrealValue)]
struct CompanyRow {
    id: RecordId,
    name: String,
    name_short: String,
}

#[derive(Debug, SurrealValue)]
struct ContactRow {
    id: RecordId,
    company: RecordId,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };

    let content = match fs::read_to_string(&args.index_md_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error reading {}: {e}", args.index_md_path);
            return ExitCode::FAILURE;
        }
    };

    let rows = p1_index_load::parse_index_md(&content);
    let total_index_rows = rows.len();
    let grouping = p1_index_load::group_rows(&rows);

    let user = match env::var("EFEES_SURREALDB_USER") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("error: EFEES_SURREALDB_USER must be set");
            return ExitCode::FAILURE;
        }
    };
    let pass = match env::var("EFEES_SURREALDB_PASS") {
        Ok(v) => v,
        Err(_) => {
            eprintln!("error: EFEES_SURREALDB_PASS must be set");
            return ExitCode::FAILURE;
        }
    };

    eprintln!(
        "backfill_p1_index_load: connecting to {} ({}/{}), mode={}",
        args.target.host,
        args.target.ns,
        args.target.db,
        if args.apply { "APPLY" } else { "DRY-RUN" }
    );

    let db: Surreal<Client> = match Surreal::new::<Ws>(args.target.host).await {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error connecting to {}: {e}", args.target.host);
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = db
        .signin(Root {
            username: user,
            password: pass,
        })
        .await
    {
        eprintln!("error signing in: {e}");
        return ExitCode::FAILURE;
    }
    if let Err(e) = db.use_ns(args.target.ns).use_db(args.target.db).await {
        eprintln!("error selecting ns/db: {e}");
        return ExitCode::FAILURE;
    }

    // --- Gather DB state -----------------------------------------------
    let known_project_ids: HashSet<String> = match db
        .query("SELECT id FROM projects")
        .await
        .and_then(|mut r| r.take::<Vec<ProjectIdRow>>(0))
    {
        Ok(rows) => rows
            .into_iter()
            .map(|r| record_key_string(&r.id.key))
            .collect(),
        Err(e) => {
            eprintln!("error reading projects: {e}");
            return ExitCode::FAILURE;
        }
    };

    let fee_rows: Vec<FeeRow> = match db
        // <float> cast is required: SurrealDB v3's SurrealValue derive for f64
        // fields rejects a raw Number::Int (some rows store an integer
        // quoted_fee) - obs:a055mx2sbu80jbpp9khs. Found live against prod
        // during this dispatch's dry-run (dev's populated row happened to be
        // stored as a float already, masking the bug there).
        .query("SELECT id, <float|none>pricing.config.quoted_fee AS qf FROM fee")
        .await
        .and_then(|mut r| r.take::<Vec<FeeRow>>(0))
    {
        Ok(rows) => rows,
        Err(e) => {
            eprintln!("error reading fee: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut existing_fee_quoted_fee: HashMap<FeeGroupKey, Option<f64>> = HashMap::new();
    let mut existing_fee_project_ids: HashSet<String> = HashSet::new();
    for row in &fee_rows {
        let full_key = record_key_string(&row.id.key);
        // fee record keys are "{project_id}_{rev}" and project_id itself has
        // no further underscores (YY_CCCNN) - split off the trailing rev.
        if let Some(idx) = full_key.rfind('_') {
            let (proj, rev_str) = full_key.split_at(idx);
            let rev_str = &rev_str[1..];
            if let Ok(rev) = rev_str.parse::<i64>() {
                existing_fee_project_ids.insert(proj.to_string());
                existing_fee_quoted_fee.insert(
                    FeeGroupKey {
                        project_id: proj.to_string(),
                        rev,
                    },
                    row.qf,
                );
            }
        }
    }

    let companies: Vec<CompanyRecord> = match db
        .query("SELECT id, name, name_short FROM company")
        .await
        .and_then(|mut r| r.take::<Vec<CompanyRow>>(0))
    {
        Ok(rows) => rows
            .into_iter()
            .map(|r| CompanyRecord {
                id: format!("company:{}", record_key_string(&r.id.key)),
                name: r.name,
                name_short: r.name_short,
            })
            .collect(),
        Err(e) => {
            eprintln!("error reading company: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut contacts_by_company: HashMap<String, Vec<String>> = HashMap::new();
    match db
        .query("SELECT id, company FROM contacts")
        .await
        .and_then(|mut r| r.take::<Vec<ContactRow>>(0))
    {
        Ok(rows) => {
            for r in rows {
                let company_id = format!("company:{}", record_key_string(&r.company.key));
                let contact_id = format!("contacts:{}", record_key_string(&r.id.key));
                contacts_by_company
                    .entry(company_id)
                    .or_default()
                    .push(contact_id);
            }
        }
        Err(e) => {
            eprintln!("error reading contacts: {e}");
            return ExitCode::FAILURE;
        }
    }

    // --- Classify ---------------------------------------------------------
    let mut plans: Vec<RowPlan> = grouping
        .consistent
        .iter()
        .map(|g| {
            p1_index_load::classify_group(
                g,
                &known_project_ids,
                &existing_fee_quoted_fee,
                &existing_fee_project_ids,
                &companies,
                &contacts_by_company,
            )
        })
        .collect();
    plans = p1_index_load::resolve_multi_revision_create_conflicts(plans);

    // --- Apply (dev only) ---------------------------------------------
    let mut created = Vec::new();
    let mut updated = Vec::new();
    let mut skipped_already_populated = Vec::new();
    let mut skipped = Vec::new();

    for plan in &plans {
        match plan {
            RowPlan::Update {
                key,
                amount,
                currency,
            } => {
                let doc = group_source_doc(&grouping.consistent, key);
                if args.apply {
                    if let Err(e) = apply_update(&db, key, *amount, currency, &doc).await {
                        eprintln!("error updating fee:{}: {e}", key.fee_record_key());
                        skipped.push(row_report(key, "error", None, None, Some(e)));
                        continue;
                    }
                }
                updated.push(row_report(
                    key,
                    "update",
                    Some(*amount),
                    Some(currency.clone()),
                    None,
                ));
            }
            RowPlan::Create {
                key,
                amount,
                currency,
                company_id,
                contact_id,
                superseded_revisions,
            } => {
                let doc = group_source_doc(&grouping.consistent, key);
                if args.apply {
                    if let Err(e) = apply_create(
                        &db,
                        key,
                        *amount,
                        currency,
                        company_id,
                        contact_id,
                        &doc,
                        superseded_revisions,
                    )
                    .await
                    {
                        eprintln!("error creating fee:{}: {e}", key.fee_record_key());
                        skipped.push(row_report(key, "error", None, None, Some(e)));
                        continue;
                    }
                }
                created.push(row_report_with_superseded(
                    key,
                    "create",
                    Some(*amount),
                    Some(currency.clone()),
                    None,
                    superseded_revisions.clone(),
                ));
            }
            RowPlan::SkipAlreadyPopulated {
                key,
                existing_quoted_fee,
            } => {
                skipped_already_populated.push(row_report(
                    key,
                    "skip_already_populated",
                    Some(*existing_quoted_fee),
                    None,
                    None,
                ));
            }
            RowPlan::Skip { key, reason } => {
                skipped.push(row_report(key, "skip", None, None, Some(reason.clone())));
            }
        }
    }

    let conflicts: Vec<ConflictReport> = grouping
        .conflicts
        .iter()
        .map(|c: &ConflictGroup| ConflictReport {
            project_id: c.key.project_id.clone(),
            rev: c.key.rev,
            variants: c.variants.clone(),
        })
        .collect();

    let report = Report {
        target: args.target.name.to_string(),
        apply: args.apply,
        counts: ReportCounts {
            total_index_rows,
            consistent_groups: grouping.consistent.len(),
            conflict_groups: grouping.conflicts.len(),
            parse_skipped_rows: grouping.skipped.len(),
            created: created.len(),
            updated: updated.len(),
            skipped_already_populated: skipped_already_populated.len(),
            skipped_other: skipped.len(),
        },
        created,
        updated,
        skipped_already_populated,
        skipped,
        conflicts,
        parse_skipped: grouping.skipped,
    };

    let json_report = serde_json::to_string_pretty(&report).unwrap();
    println!("{json_report}");

    if let Some(path) = &args.report_file {
        if let Err(e) = fs::write(path, &json_report) {
            eprintln!("warning: failed to write report file {path}: {e}");
        }
    }

    ExitCode::SUCCESS
}

fn group_source_doc(groups: &[ConsistentGroup], key: &FeeGroupKey) -> String {
    groups
        .iter()
        .find(|g| &g.key == key)
        .and_then(|g| g.source_docs.first().cloned())
        .unwrap_or_else(|| format!("{}-FP-{:02}", key.display_number(), key.rev))
}

fn row_report(
    key: &FeeGroupKey,
    action: &str,
    amount: Option<f64>,
    currency: Option<String>,
    reason: Option<String>,
) -> RowReport {
    row_report_with_superseded(key, action, amount, currency, reason, Vec::new())
}

#[allow(clippy::too_many_arguments)]
fn row_report_with_superseded(
    key: &FeeGroupKey,
    action: &str,
    amount: Option<f64>,
    currency: Option<String>,
    reason: Option<String>,
    superseded_revisions: Vec<SupersededRevision>,
) -> RowReport {
    RowReport {
        project_id: key.project_id.clone(),
        rev: key.rev,
        action: action.to_string(),
        amount,
        currency,
        reason,
        superseded_revisions,
    }
}

async fn apply_update(
    db: &Surreal<Client>,
    key: &FeeGroupKey,
    amount: f64,
    currency: &str,
    doc: &str,
) -> Result<(), String> {
    let fee_id = RecordId::new("fee", key.fee_record_key());
    db.query(
        "UPDATE $fee_id SET \
            pricing.config.quoted_fee = $amount, \
            pricing.config.currency = $currency, \
            data_provenance = { \
                source: 'backfill', \
                confidence: 'high', \
                backfilled_at: time::now(), \
                backfilled_by: 'ingestion-P1-index-load' \
            }, \
            source_document = $doc;",
    )
    .bind(("fee_id", fee_id))
    .bind(("amount", amount))
    .bind(("currency", currency.to_string()))
    .bind(("doc", doc.to_string()))
    .await
    .map_err(|e| e.to_string())?
    .check()
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn apply_create(
    db: &Surreal<Client>,
    key: &FeeGroupKey,
    amount: f64,
    currency: &str,
    company_id: &str,
    contact_id: &str,
    doc: &str,
    superseded_revisions: &[SupersededRevision],
) -> Result<(), String> {
    let fee_id = RecordId::new("fee", key.fee_record_key());
    let project_id = RecordId::new("projects", key.project_id.clone());
    let company_rid = parse_record_id(company_id)?;
    let contact_rid = parse_record_id(contact_id)?;
    let number = format!("{}-FP-{}", key.display_number(), key.rev);

    // `fee` is SCHEMALESS (CLAUDE.md), so no migration is needed to write
    // `superseded_revisions` here - it just needs to exist for the multi-
    // revision seeding ruling (Martin, 2026-08-20): the earlier revision(s)'
    // amount is carried forward in provenance instead of getting its own
    // fee row, since `fee_project_rev`'s unique index allows only one.
    let backfilled_at = chrono::Utc::now().to_rfc3339();
    let provenance = serde_json::json!({
        "source": "backfill",
        "confidence": "high",
        "backfilled_at": backfilled_at,
        "backfilled_by": "ingestion-P1-index-load",
        "superseded_revisions": superseded_revisions
            .iter()
            .map(|s| serde_json::json!({ "rev": s.rev, "amount": s.amount, "currency": s.currency }))
            .collect::<Vec<_>>(),
    });

    // `revisions[]` MUST be populated - `fee.rev` is DB-computed from
    // `revisions[*].revision_number` (see
    // `crates/e-fees-core/src/backfill/fee_revisions_seed.rs`'s module doc
    // for the full story: an empty array here would silently compute
    // `rev = 0` for this newly-created row too, reintroducing the exact
    // bug this backfill's own sibling (`backfill_fee_revisions_seed`) fixes
    // retroactively on every OTHER row). One entry per superseded revision
    // (ascending) plus one for this row's own rev = key.rev, matching the
    // shape `plan_row_backfill` produces for a row it finds after the fact.
    let mut revisions: Vec<Revision> = superseded_revisions
        .iter()
        .map(|s| {
            Revision::at(
                s.rev,
                "",
                "",
                format!(
                    "Backfilled: earlier revision reconstructed from \
                     data_provenance.superseded_revisions (P1 historical-backfill \
                     loader) (amount {} {})",
                    s.amount, s.currency
                ),
                &backfilled_at,
            )
        })
        .collect();
    let latest_note = if revisions.is_empty() {
        "Backfilled: P1 corpus INDEX.md load - initial revision for this project."
    } else {
        "Backfilled: P1 corpus INDEX.md load - seeded as the latest known revision \
         (data_provenance.superseded_revisions records the earlier amount(s))."
    };
    revisions.push(Revision::at(key.rev, "", "", latest_note, &backfilled_at));

    db.query(
        "CREATE $fee_id CONTENT { \
            name: 'Fee Proposal', \
            number: $number, \
            status: 'Draft', \
            issue_date: '000000', \
            activity: '', \
            package: '', \
            project_id: $project_id, \
            company_id: $company_id, \
            contact_id: $contact_id, \
            staff_name: '', \
            staff_email: '', \
            staff_phone: '', \
            staff_position: '', \
            strap_line: 'sensory design studio', \
            revisions: $revisions, \
            time: {}, \
            pricing: { config: { quoted_fee: $amount, currency: $currency } }, \
            data_provenance: $provenance, \
            source_document: $doc \
        };",
    )
    .bind(("fee_id", fee_id))
    .bind(("number", number))
    .bind(("project_id", project_id))
    .bind(("company_id", company_rid))
    .bind(("contact_id", contact_rid))
    .bind(("amount", amount))
    .bind(("currency", currency.to_string()))
    .bind(("doc", doc.to_string()))
    .bind(("provenance", provenance))
    // Bind via serde_json::Value, NOT the native Vec<Revision>
    // (SurrealValue-derived) - confirmed live on dev this session
    // (backfill_fee_revisions_seed's own apply run): binding a
    // SurrealValue-derived Vec<struct> directly against `revisions.*` (a
    // schemaful nested object field) fails with "Couldn't coerce value for
    // field `revisions.*`... Expected `object` but found `NONE`", while the
    // identical data serialized to JSON first succeeds - matches
    // `db/client.rs`'s working `create_fee`/`update_fee` strategy for this
    // exact field.
    .bind((
        "revisions",
        serde_json::to_value(&revisions).unwrap_or(serde_json::json!([])),
    ))
    .await
    .map_err(|e| e.to_string())?
    .check()
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn parse_record_id(full: &str) -> Result<RecordId, String> {
    let (table, key) = full
        .split_once(':')
        .ok_or_else(|| format!("expected table:key, got {full:?}"))?;
    Ok(RecordId::new(table, key))
}
