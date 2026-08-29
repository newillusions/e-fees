//! Historical-backfill: retroactively seed `fee.revisions[]` on every row
//! created before the fee-revisions fix (2026-08-28), so the DB-computed
//! `rev` field stops evaluating to `0`.
//!
//! WHY THIS EXISTS: `fee.rev` is `VALUE (IF array::len(revisions.*.revision_number)
//! greater than 0 THEN math::max(...) ELSE 0 END)` (see
//! `scripts/migration/v007_fee_revisions.surql`). Every fee-construction
//! call site wrote `revisions: []` before this fix (`clone_fee_as_revision`,
//! `import_wizard.rs`, `agent_server.rs`, and this repo's own
//! `backfill_p1_index_load.rs::apply_create`), so `rev` always computed to
//! `0` - confirmed live on dev, 2026-08-28: 100% of 30 sampled fee rows had
//! `rev: 0, revisions: []`. This backfill is the retroactive half of that
//! fix; `crate::models::Revision::new` (called from every now-fixed
//! call site) is the forward-going half.
//!
//! Pure, DB-free planning logic lives here (unit-tested); the binary
//! (`src/bin/backfill_fee_revisions_seed.rs`) wires it against a real
//! SurrealDB connection, following the same `--target`/`--apply`/
//! `--confirm-prod` shape as `p1_index_load.rs`/`backfill_p1_index_load.rs`.
//!
//! PROVENANCE HANDLING: a row whose `data_provenance.superseded_revisions`
//! is non-empty (written by `backfill_p1_index_load.rs::apply_create` for
//! projects where the historical corpus implied more than one revision -
//! only the latest could get its own fee row, per Martin's 2026-08-20
//! ruling, obs referenced in `p1_index_load.rs`'s header) gets its FULL
//! multi-entry history reconstructed: one entry per superseded revision
//! plus one for the row's own (latest) revision. Every other row - the
//! overwhelming majority, and as of 2026-08-28 the entirety of dev's 30
//! rows, since the P1 loader's real `--apply` run has not landed yet -
//! gets a single `revision_number: 1` entry.

use crate::models::Revision;
use serde::{Deserialize, Serialize};

/// One superseded-revision entry as written by the P1 historical-backfill
/// loader into `fee.data_provenance.superseded_revisions`. Duplicated here
/// deliberately rather than importing
/// `crate::backfill::p1_index_load::SupersededRevision` - this module only
/// needs `rev` (+ `amount`/`currency` for the note text), and importing the
/// whole P1 type would couple two otherwise-independent backfills for no
/// benefit. Field shape matches `p1_index_load.rs::apply_create`'s
/// `provenance` JSON exactly.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ProvenanceSupersededRevision {
    pub rev: i64,
    #[serde(default)]
    pub amount: Option<f64>,
    #[serde(default)]
    pub currency: Option<String>,
}

/// The minimal facts this module needs about one fee row to plan its
/// backfill - deliberately narrow, gathered by the binary via a live query.
#[derive(Debug, Clone)]
pub struct FeeRowFacts {
    /// The fee row's own record key, e.g. "24_97101_2" - parsed to recover
    /// the revision number the row was INTENDED to represent at create
    /// time. The DB-computed `rev` field can't be trusted for this - it is
    /// exactly what this backfill exists to fix. Record keys are
    /// `{project_id}_{rev}` (project_id itself is `YY_CCCNN`, no further
    /// underscores) - same convention `backfill_p1_index_load.rs` and
    /// `db/client.rs::create_fee` both rely on.
    pub record_key: String,
    /// `array::len(revisions)` at read time - only rows where this is 0
    /// are backfill candidates; already-populated rows are left untouched
    /// (this backfill is idempotent by construction, not just by re-run
    /// convention).
    pub existing_revisions_len: i64,
    pub superseded_revisions: Vec<ProvenanceSupersededRevision>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RowBackfillPlan {
    /// Already has real `revisions[]` - nothing to do. Distinct from `Skip`
    /// so callers can report "already correct" separately from "could not
    /// determine a safe plan".
    AlreadyPopulated,
    /// Write this exact `revisions[]` array (already sorted ascending by
    /// `revision_number`, guaranteed non-empty).
    Seed(Vec<Revision>),
    /// Named skip - never a bare "skipped". The record key didn't parse
    /// into `{project}_{rev}` as every fee record is documented to
    /// (CLAUDE.md "Record keys use underscore form"), or the row's
    /// provenance was internally inconsistent, so no revision_number could
    /// be safely inferred.
    Skip { reason: String },
}

const BACKFILL_NOTE_SEED: &str =
    "Backfilled: retroactive revision-history seed - this fee predates the \
     fee.revisions[] population fix (see docs/development/DATABASE_SCHEMA.md \
     'Revision Management').";
const BACKFILL_NOTE_SUPERSEDED: &str = "Backfilled: earlier revision reconstructed from \
     data_provenance.superseded_revisions (P1 historical-backfill loader).";
const BACKFILL_NOTE_LATEST_FROM_PROVENANCE: &str =
    "Backfilled: seeded as the latest known revision for this fee \
     (data_provenance.superseded_revisions records the earlier amount(s)).";

/// Parse the trailing `_<rev>` off a fee record key ("24_97101_2" -> 2).
pub fn parse_rev_from_record_key(record_key: &str) -> Option<i64> {
    let (_, rev_str) = record_key.rsplit_once('_')?;
    rev_str.parse::<i64>().ok()
}

/// Plan the backfill for one fee row. `backfilled_at` is an RFC3339
/// timestamp supplied by the caller (one fixed value per binary run) rather
/// than read from the clock in here, so a `--dry-run` report and the later
/// real `--apply` of the SAME invocation produce byte-identical revision
/// entries, and so this function stays a pure, deterministically-testable
/// planner.
pub fn plan_row_backfill(row: &FeeRowFacts, backfilled_at: &str) -> RowBackfillPlan {
    if row.existing_revisions_len > 0 {
        return RowBackfillPlan::AlreadyPopulated;
    }

    let own_rev = match parse_rev_from_record_key(&row.record_key) {
        Some(r) if r > 0 => r,
        _ => {
            return RowBackfillPlan::Skip {
                reason: format!(
                    "fee record key '{}' does not end in a positive integer revision - \
                     cannot infer revision_number",
                    row.record_key
                ),
            };
        }
    };

    let mut superseded: Vec<&ProvenanceSupersededRevision> =
        row.superseded_revisions.iter().collect();
    superseded.sort_by_key(|s| s.rev);

    // Sanity check: every superseded rev must be strictly less than the
    // row's own rev (the row always represents the LATEST known revision,
    // per the P1 loader's 2026-08-20 owner ruling) and there must be no
    // duplicate/overlapping rev numbers. A violation means the provenance
    // data doesn't match this module's assumptions - refuse to guess an
    // ordering rather than writing a plausible-looking but wrong history.
    let mut seen = std::collections::HashSet::new();
    for s in &superseded {
        if s.rev >= own_rev || !seen.insert(s.rev) {
            return RowBackfillPlan::Skip {
                reason: format!(
                    "fee record key '{}': data_provenance.superseded_revisions contains \
                     rev {} which is >= the row's own rev {} or duplicated - refusing to \
                     guess an ordering",
                    row.record_key, s.rev, own_rev
                ),
            };
        }
    }

    let mut revisions: Vec<Revision> = superseded
        .iter()
        .map(|s| {
            let amount_note = match (s.amount, &s.currency) {
                (Some(a), Some(c)) => format!(" (amount {a} {c})"),
                (Some(a), None) => format!(" (amount {a})"),
                _ => String::new(),
            };
            Revision::at(
                s.rev,
                "",
                "",
                format!("{BACKFILL_NOTE_SUPERSEDED}{amount_note}"),
                backfilled_at,
            )
        })
        .collect();

    let latest_note = if revisions.is_empty() {
        BACKFILL_NOTE_SEED
    } else {
        BACKFILL_NOTE_LATEST_FROM_PROVENANCE
    };
    revisions.push(Revision::at(own_rev, "", "", latest_note, backfilled_at));

    RowBackfillPlan::Seed(revisions)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(record_key: &str, existing_len: i64) -> FeeRowFacts {
        FeeRowFacts {
            record_key: record_key.to_string(),
            existing_revisions_len: existing_len,
            superseded_revisions: Vec::new(),
        }
    }

    #[test]
    fn already_populated_rows_are_left_alone() {
        let row = facts("24_97101_1", 1);
        assert_eq!(
            plan_row_backfill(&row, "2026-08-28T00:00:00Z"),
            RowBackfillPlan::AlreadyPopulated
        );
    }

    #[test]
    fn simple_row_gets_a_single_revision_1_entry() {
        let row = facts("25_97102_1", 0);
        match plan_row_backfill(&row, "2026-08-28T00:00:00Z") {
            RowBackfillPlan::Seed(revisions) => {
                assert_eq!(revisions.len(), 1);
                assert_eq!(revisions[0].revision_number, 1);
                assert_eq!(revisions[0].revision_date, "2026-08-28T00:00:00Z");
                assert!(revisions[0]
                    .notes
                    .contains("retroactive revision-history seed"));
            }
            other => panic!("expected Seed, got {other:?}"),
        }
    }

    #[test]
    fn multi_digit_rev_parses_correctly() {
        let row = facts("25_97102_12", 0);
        match plan_row_backfill(&row, "2026-08-28T00:00:00Z") {
            RowBackfillPlan::Seed(revisions) => {
                assert_eq!(revisions.len(), 1);
                assert_eq!(revisions[0].revision_number, 12);
            }
            other => panic!("expected Seed, got {other:?}"),
        }
    }

    #[test]
    fn row_with_superseded_provenance_gets_full_history_in_order() {
        let mut row = facts("24_97101_2", 0);
        row.superseded_revisions = vec![ProvenanceSupersededRevision {
            rev: 1,
            amount: Some(628500.0),
            currency: Some("AED".to_string()),
        }];
        match plan_row_backfill(&row, "2026-08-28T00:00:00Z") {
            RowBackfillPlan::Seed(revisions) => {
                assert_eq!(revisions.len(), 2);
                assert_eq!(revisions[0].revision_number, 1);
                assert!(revisions[0].notes.contains("628500"));
                assert!(revisions[0].notes.contains("AED"));
                assert_eq!(revisions[1].revision_number, 2);
                assert!(revisions[1].notes.contains("latest known revision"));
            }
            other => panic!("expected Seed, got {other:?}"),
        }
    }

    #[test]
    fn multiple_superseded_entries_sort_ascending_regardless_of_input_order() {
        let mut row = facts("24_97101_4", 0);
        row.superseded_revisions = vec![
            ProvenanceSupersededRevision {
                rev: 2,
                amount: None,
                currency: None,
            },
            ProvenanceSupersededRevision {
                rev: 1,
                amount: None,
                currency: None,
            },
            ProvenanceSupersededRevision {
                rev: 3,
                amount: None,
                currency: None,
            },
        ];
        match plan_row_backfill(&row, "2026-08-28T00:00:00Z") {
            RowBackfillPlan::Seed(revisions) => {
                let numbers: Vec<i64> = revisions.iter().map(|r| r.revision_number).collect();
                assert_eq!(numbers, vec![1, 2, 3, 4]);
            }
            other => panic!("expected Seed, got {other:?}"),
        }
    }

    #[test]
    fn unparseable_record_key_is_a_named_skip_not_a_default() {
        let row = facts("not-a-valid-key", 0);
        match plan_row_backfill(&row, "2026-08-28T00:00:00Z") {
            RowBackfillPlan::Skip { reason } => {
                assert!(reason.contains("does not end in a positive integer"));
            }
            other => panic!("expected Skip, got {other:?}"),
        }
    }

    #[test]
    fn zero_or_negative_trailing_number_is_a_named_skip() {
        let row = facts("25_97102_0", 0);
        match plan_row_backfill(&row, "2026-08-28T00:00:00Z") {
            RowBackfillPlan::Skip { reason } => {
                assert!(reason.contains("does not end in a positive integer"));
            }
            other => panic!("expected Skip, got {other:?}"),
        }
    }

    #[test]
    fn superseded_rev_at_or_above_own_rev_is_a_named_skip() {
        let mut row = facts("24_97101_2", 0);
        row.superseded_revisions = vec![ProvenanceSupersededRevision {
            rev: 2,
            amount: None,
            currency: None,
        }];
        match plan_row_backfill(&row, "2026-08-28T00:00:00Z") {
            RowBackfillPlan::Skip { reason } => {
                assert!(reason.contains(">= the row's own rev"));
            }
            other => panic!("expected Skip, got {other:?}"),
        }
    }

    #[test]
    fn duplicate_superseded_rev_is_a_named_skip() {
        let mut row = facts("24_97101_3", 0);
        row.superseded_revisions = vec![
            ProvenanceSupersededRevision {
                rev: 1,
                amount: None,
                currency: None,
            },
            ProvenanceSupersededRevision {
                rev: 1,
                amount: None,
                currency: None,
            },
        ];
        match plan_row_backfill(&row, "2026-08-28T00:00:00Z") {
            RowBackfillPlan::Skip { reason } => {
                assert!(reason.contains("duplicated"));
            }
            other => panic!("expected Skip, got {other:?}"),
        }
    }

    #[test]
    fn backfill_is_deterministic_across_repeated_calls() {
        let row = facts("25_97102_1", 0);
        let a = plan_row_backfill(&row, "2026-08-28T00:00:00Z");
        let b = plan_row_backfill(&row, "2026-08-28T00:00:00Z");
        assert_eq!(a, b);
    }
}
