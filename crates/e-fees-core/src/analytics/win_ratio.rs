//! Per-client win-ratio aggregation.
//!
//! # Data model this relies on
//!
//! `projects.outcome` (added by the 2026-08 historical backfill,
//! `scripts/migration/v006_backfill_metadata_fields.surql`) is one of
//! Won / Lost / No Response / Cancelled, or `NONE` while the opportunity is
//! still pending a decision. `company` is NOT a field on `projects` - it is
//! only reachable via a linked `fee` row's `company_id` (required by that
//! table's schema). So attributing a project's outcome to a client always
//! goes through its linked fee(s), and a decided project with zero fee rows
//! cannot be attributed at all (see `WinRatioReport::unattributed_decided_count`).
//!
//! # Lineage dedup
//!
//! A project that got re-scoped mid-flight is recorded as a chain:
//! `stage=Superseded` on the earlier record, with `successor` pointing
//! forward to the record that continues it (never backward, never by
//! `name` - see the migration file's comment on why `folder` is the match
//! key upstream). A won lineage must count once, not once per record in the
//! chain, or a project re-scoped twice (the historical Shanghai Tang case)
//! triples its own win. The rule: only count a project whose `successor` is
//! `None` - i.e. the terminal record of its chain (or a standalone project
//! that was never superseded). An intermediate record's own `outcome` is
//! real (the predecessor genuinely was won) but is superseded BY that later
//! record's outcome, which is what should represent the lineage.
//!
//! # Currency
//!
//! Fee amounts are grouped by currency, never summed across currencies -
//! Emittiv quotes in AED, USD and others depending on the client's market,
//! and blending them into one number would misstate every total.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

use crate::models::{record_id_string, ClientWinRatio, CurrencyAmount, Fee, WinRatioReport};

/// Raw shape of the fields on `projects` this report needs, beyond the
/// app-facing `Project` model (which deliberately does not carry the
/// backfill-only fields - see the migration file's own comment on why).
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ProjectOutcomeRow {
    pub id: RecordId,
    pub outcome: Option<String>,
    pub successor: Option<RecordId>,
}

/// Outcome bucket totals plus fee-value accumulators for one company,
/// built up incrementally then converted to a `ClientWinRatio`.
#[derive(Debug, Default)]
struct Accumulator {
    won_count: i64,
    lost_count: i64,
    no_response_count: i64,
    cancelled_count: i64,
    pending_count: i64,
    won_value: BTreeMap<String, f64>,
    lost_value: BTreeMap<String, f64>,
}

impl Accumulator {
    fn record(&mut self, outcome: Option<&str>, amount: Option<(String, f64)>) {
        match outcome {
            Some("Won") => {
                self.won_count += 1;
                if let Some((currency, quoted)) = amount {
                    *self.won_value.entry(currency).or_insert(0.0) += quoted;
                }
            }
            Some("Lost") => {
                self.lost_count += 1;
                if let Some((currency, quoted)) = amount {
                    *self.lost_value.entry(currency).or_insert(0.0) += quoted;
                }
            }
            Some("No Response") => {
                self.no_response_count += 1;
                if let Some((currency, quoted)) = amount {
                    *self.lost_value.entry(currency).or_insert(0.0) += quoted;
                }
            }
            Some("Cancelled") => self.cancelled_count += 1,
            None => self.pending_count += 1,
            // The DB ASSERT closes `outcome` to the four values above (or
            // NONE) - an unrecognised string can only reach here via a
            // future schema change this report hasn't been updated for.
            // Drop it rather than panic (a safer default than crashing the
            // report), but warn loudly - a silently-dropped bucket is
            // exactly the "vanishes without a trace" failure mode this
            // module exists to avoid.
            Some(other) => {
                log::warn!(
                    "win_ratio: unrecognised project outcome {other:?} - dropped from every \
                     bucket; check for schema/ASSERT drift on projects.outcome"
                );
            }
        }
    }

    fn into_client_win_ratio(self, company_id: String, company_name: String) -> ClientWinRatio {
        let decided_count = self.won_count + self.lost_count + self.no_response_count;
        let win_ratio = if decided_count > 0 {
            Some(self.won_count as f64 / decided_count as f64)
        } else {
            None
        };
        ClientWinRatio {
            company_id,
            company_name,
            won_count: self.won_count,
            lost_count: self.lost_count,
            no_response_count: self.no_response_count,
            cancelled_count: self.cancelled_count,
            pending_count: self.pending_count,
            decided_count,
            win_ratio,
            won_value: to_currency_amounts(self.won_value),
            lost_value: to_currency_amounts(self.lost_value),
        }
    }
}

fn to_currency_amounts(map: BTreeMap<String, f64>) -> Vec<CurrencyAmount> {
    map.into_iter()
        .map(|(currency, amount)| CurrencyAmount { currency, amount })
        .collect()
}

/// Walk a project's `successor` chain (with cycle detection) to determine
/// whether it eventually reaches a real terminal record - one that exists
/// in `id_index` and has `successor: None`. A project whose own successor
/// is `None` is trivially terminal and is never passed here (the caller
/// only calls this for records with `successor: Some`); this function
/// exists to distinguish a genuine "superseded, chain resolves onward"
/// record from a broken one: a dangling pointer (successor points at a
/// project not present in this dataset), a self-reference, or a cycle
/// between two or more records. Any of those three shapes returns `false`.
fn chain_terminates(start: &str, id_index: &BTreeMap<String, &ProjectOutcomeRow>) -> bool {
    let mut current = start.to_string();
    let mut visited: BTreeSet<String> = BTreeSet::new();
    loop {
        if !visited.insert(current.clone()) {
            return false; // revisited a node - cycle (a self-reference is a 1-node cycle)
        }
        let Some(project) = id_index.get(current.as_str()) else {
            return false; // shouldn't happen on the first iteration (start is always in-index)
        };
        match &project.successor {
            None => return true,
            Some(successor) => {
                let successor_key = record_id_string(successor);
                if !id_index.contains_key(successor_key.as_str()) {
                    return false; // dangling successor pointer
                }
                current = successor_key;
            }
        }
    }
}

/// Century-pivot-aware parse of a `YYMMDD` issue-date string, matching the
/// convention already used by `export::format_issue_date` elsewhere in this
/// crate: two-digit year < 50 -> 20xx, >= 50 -> 19xx. Returns `None` for
/// anything that doesn't parse as a real calendar date rather than guessing.
fn parse_issue_date(date_str: &str) -> Option<chrono::NaiveDate> {
    if date_str.len() != 6 {
        return None;
    }
    let year = date_str.get(0..2)?.parse::<i32>().ok()?;
    let month = date_str.get(2..4)?.parse::<u32>().ok()?;
    let day = date_str.get(4..6)?.parse::<u32>().ok()?;
    let full_year = if year >= 50 { 1900 + year } else { 2000 + year };
    chrono::NaiveDate::from_ymd_opt(full_year, month, day)
}

/// True if `candidate` should replace `existing` as the fee representing a
/// project's current quoted state. Preference order: an Accepted fee beats
/// any non-Accepted fee regardless of revision (an abandoned higher-rev
/// draft renegotiation must never outrank the fee that was actually agreed
/// to); then the higher revision; then, on an equal-rev tie, the later
/// issue date.
fn is_preferred_fee(candidate: &Fee, existing: &Fee) -> bool {
    let candidate_accepted = candidate.status == "Accepted";
    let existing_accepted = existing.status == "Accepted";
    if candidate_accepted != existing_accepted {
        return candidate_accepted;
    }
    if candidate.rev != existing.rev {
        return candidate.rev > existing.rev;
    }
    parse_issue_date(&candidate.issue_date) > parse_issue_date(&existing.issue_date)
}

/// Aggregate raw project/fee/company rows into a per-client win-ratio
/// report. Pure function - no DB access - so it's unit-testable directly
/// against fixture rows.
pub fn aggregate_win_ratios(
    projects: &[ProjectOutcomeRow],
    fees: &[Fee],
    company_names: &BTreeMap<String, String>,
) -> WinRatioReport {
    // Lineage dedup: only terminal (non-superseded-further) records.
    let outcome_by_project: BTreeMap<String, Option<String>> = projects
        .iter()
        .filter(|p| p.successor.is_none())
        .map(|p| (record_id_string(&p.id), p.outcome.clone()))
        .collect();

    // Preferred fee per project: an Accepted fee always wins over a
    // non-Accepted one regardless of revision; otherwise the highest
    // revision; on an equal-rev tie, the later issue date.
    let mut latest_fee_by_project: BTreeMap<String, &Fee> = BTreeMap::new();
    for fee in fees {
        let key = record_id_string(&fee.project_id);
        latest_fee_by_project
            .entry(key)
            .and_modify(|existing| {
                if is_preferred_fee(fee, existing) {
                    *existing = fee;
                }
            })
            .or_insert(fee);
    }

    let mut by_company: BTreeMap<String, Accumulator> = BTreeMap::new();
    let mut unattributed_decided_count = 0i64;

    // Broken-lineage records: a project with a successor pointer whose
    // chain never reaches a real terminal record - dangling, self-
    // referencing, or cyclic - must not silently vanish from every
    // counter. A decided outcome on one of these folds into
    // unattributed_decided_count, exactly like a decided project with no
    // linked fee does; a pending one is simply skipped, same as today.
    let id_index: BTreeMap<String, &ProjectOutcomeRow> =
        projects.iter().map(|p| (record_id_string(&p.id), p)).collect();
    for project in projects.iter().filter(|p| p.successor.is_some()) {
        let key = record_id_string(&project.id);
        if project.outcome.is_some() && !chain_terminates(&key, &id_index) {
            unattributed_decided_count += 1;
        }
    }

    for (project_key, outcome) in &outcome_by_project {
        let Some(fee) = latest_fee_by_project.get(project_key) else {
            // No linked fee at all: a pending project with no proposal
            // issued yet is normal and not counted anywhere. A *decided*
            // project with no fee is a real data gap - surfaced as a
            // top-level count rather than silently dropped.
            if outcome.is_some() {
                unattributed_decided_count += 1;
            }
            continue;
        };

        let company_key = record_id_string(&fee.company_id);
        let amount = fee.pricing_typed().and_then(|breakdown| {
            let quoted = breakdown.config.quoted_fee;
            (quoted > 0.0).then(|| (breakdown.config.currency, quoted))
        });

        by_company
            .entry(company_key)
            .or_default()
            .record(outcome.as_deref(), amount);
    }

    let mut clients: Vec<ClientWinRatio> = by_company
        .into_iter()
        .map(|(company_id, accum)| {
            let company_name = company_names
                .get(&company_id)
                .cloned()
                .unwrap_or_else(|| company_id.clone());
            accum.into_client_win_ratio(company_id, company_name)
        })
        .collect();

    // Busiest (most-decided) clients first; alphabetical as a stable
    // tiebreak so the order doesn't jitter between runs.
    clients.sort_by(|a, b| {
        b.decided_count
            .cmp(&a.decided_count)
            .then_with(|| a.company_name.cmp(&b.company_name))
    });

    WinRatioReport {
        clients,
        unattributed_decided_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{json_to_dbvalue, PricingBreakdown, PricingConfig, TimeStamps};
    use surrealdb_types::Datetime;

    fn project_row(key: &str, outcome: Option<&str>, successor: Option<&str>) -> ProjectOutcomeRow {
        ProjectOutcomeRow {
            id: RecordId::new("projects", key),
            outcome: outcome.map(str::to_string),
            successor: successor.map(|s| RecordId::new("projects", s)),
        }
    }

    fn fee_row(project_key: &str, company_key: &str, rev: i64, quoted_fee: f64, currency: &str) -> Fee {
        let mut fee = Fee {
            id: Some(RecordId::new("fee", format!("{project_key}_{rev}"))),
            name: "Fee Proposal".into(),
            number: format!("{project_key}-{rev}"),
            rev,
            status: "Sent".into(),
            issue_date: "260101".into(),
            activity: String::new(),
            package: String::new(),
            project_id: RecordId::new("projects", project_key),
            company_id: RecordId::new("company", company_key),
            contact_id: RecordId::new("contacts", "test"),
            staff_name: String::new(),
            staff_email: String::new(),
            staff_phone: String::new(),
            staff_position: String::new(),
            strap_line: String::new(),
            revisions: vec![],
            time: TimeStamps {
                created_at: Datetime::default(),
                updated_at: Datetime::default(),
            },
            pricing: None,
            post_contract_items: None,
            reimbursable_costs: None,
            payment_schedule: None,
            pricing_revisions: None,
            current_revision_number: None,
            current_release_number: None,
            import_source: None,
        };
        let breakdown = PricingBreakdown {
            config: PricingConfig {
                target_fee: 0.0,
                buffer_percent: 0.0,
                quoted_fee,
                currency: currency.to_string(),
                vat_percent: 0.0,
                vat_included: false,
                mobilisation_percent: 0.0,
            },
            ..Default::default()
        };
        fee.pricing = Some(json_to_dbvalue(&serde_json::to_value(&breakdown).unwrap()));
        fee
    }

    /// Like `fee_row` but with an explicit status and issue_date, for the
    /// Accepted-preference and equal-rev tiebreak tests.
    fn fee_row_full(
        project_key: &str,
        company_key: &str,
        rev: i64,
        quoted_fee: f64,
        currency: &str,
        status: &str,
        issue_date: &str,
    ) -> Fee {
        let mut fee = fee_row(project_key, company_key, rev, quoted_fee, currency);
        fee.status = status.to_string();
        fee.issue_date = issue_date.to_string();
        fee
    }
    fn company_names(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(id, name)| (format!("company:{id}"), name.to_string()))
            .collect()
    }

    #[test]
    fn wins_and_no_responses_compute_a_ratio_per_client() {
        let projects = vec![
            project_row("p1", Some("Won"), None),
            project_row("p2", Some("No Response"), None),
            project_row("p3", Some("No Response"), None),
        ];
        let fees = vec![
            fee_row("p1", "PTG", 1, 50000.0, "AED"),
            fee_row("p2", "PTG", 1, 30000.0, "AED"),
            fee_row("p3", "PTG", 1, 20000.0, "AED"),
        ];
        let report = aggregate_win_ratios(&projects, &fees, &company_names(&[("PTG", "Petrus Group")]));

        assert_eq!(report.unattributed_decided_count, 0);
        assert_eq!(report.clients.len(), 1);
        let ptg = &report.clients[0];
        assert_eq!(ptg.company_name, "Petrus Group");
        assert_eq!(ptg.won_count, 1);
        assert_eq!(ptg.no_response_count, 2);
        assert_eq!(ptg.decided_count, 3);
        assert_eq!(ptg.win_ratio, Some(1.0 / 3.0));
        assert_eq!(
            ptg.won_value,
            vec![CurrencyAmount {
                currency: "AED".into(),
                amount: 50000.0
            }]
        );
        assert_eq!(
            ptg.lost_value,
            vec![CurrencyAmount {
                currency: "AED".into(),
                amount: 50000.0
            }]
        );
    }

    #[test]
    fn superseded_lineage_counts_once_via_the_terminal_record() {
        // p1 (Superseded, outcome Won, successor -> p2); p2 is the terminal
        // record, still open (outcome None/pending). The lineage must not
        // register as a Won in the ratio - it registers as pending, because
        // that is p2's honest current state and p1 is superseded.
        let projects = vec![
            project_row("p1", Some("Won"), Some("p2")),
            project_row("p2", None, None),
        ];
        let fees = vec![fee_row("p1", "PTG", 1, 40000.0, "AED"), fee_row("p2", "PTG", 1, 42000.0, "AED")];
        let report = aggregate_win_ratios(&projects, &fees, &company_names(&[("PTG", "Petrus Group")]));

        let ptg = &report.clients[0];
        assert_eq!(ptg.won_count, 0, "p1 is superseded and must not count directly");
        assert_eq!(ptg.pending_count, 1, "p2 is the terminal record and is still pending");
        assert_eq!(ptg.decided_count, 0);
        assert_eq!(ptg.win_ratio, None);
    }

    #[test]
    fn superseded_lineage_that_completes_counts_the_win_once() {
        let projects = vec![
            project_row("p1", Some("Won"), Some("p2")),
            project_row("p2", Some("Won"), None),
        ];
        let fees = vec![fee_row("p1", "PTG", 1, 40000.0, "AED"), fee_row("p2", "PTG", 1, 55000.0, "AED")];
        let report = aggregate_win_ratios(&projects, &fees, &company_names(&[("PTG", "Petrus Group")]));

        let ptg = &report.clients[0];
        assert_eq!(ptg.won_count, 1, "the lineage is one win, not two");
        assert_eq!(
            ptg.won_value,
            vec![CurrencyAmount {
                currency: "AED".into(),
                amount: 55000.0
            }],
            "value comes from the terminal record's own fee, not the superseded one"
        );
    }

    #[test]
    fn decided_project_with_no_linked_fee_is_reported_as_unattributed_not_dropped() {
        let projects = vec![project_row("p1", Some("Won"), None)];
        let report = aggregate_win_ratios(&projects, &[], &BTreeMap::new());

        assert!(report.clients.is_empty());
        assert_eq!(report.unattributed_decided_count, 1);
    }

    #[test]
    fn pending_project_with_no_linked_fee_is_silently_skipped() {
        let projects = vec![project_row("p1", None, None)];
        let report = aggregate_win_ratios(&projects, &[], &BTreeMap::new());

        assert!(report.clients.is_empty());
        assert_eq!(
            report.unattributed_decided_count, 0,
            "a not-yet-proposed pending project is not a data gap"
        );
    }

    #[test]
    fn amounts_are_grouped_by_currency_not_summed_across_currencies() {
        let projects = vec![project_row("p1", Some("Won"), None), project_row("p2", Some("Won"), None)];
        let fees = vec![
            fee_row("p1", "PTG", 1, 50000.0, "AED"),
            fee_row("p2", "PTG", 1, 12000.0, "USD"),
        ];
        let report = aggregate_win_ratios(&projects, &fees, &company_names(&[("PTG", "Petrus Group")]));

        let ptg = &report.clients[0];
        assert_eq!(ptg.won_count, 2);
        let mut values = ptg.won_value.clone();
        values.sort_by(|a, b| a.currency.cmp(&b.currency));
        assert_eq!(
            values,
            vec![
                CurrencyAmount { currency: "AED".into(), amount: 50000.0 },
                CurrencyAmount { currency: "USD".into(), amount: 12000.0 },
            ]
        );
    }

    #[test]
    fn cancelled_projects_are_counted_but_excluded_from_the_ratio() {
        let projects = vec![project_row("p1", Some("Won"), None), project_row("p2", Some("Cancelled"), None)];
        let fees = vec![fee_row("p1", "PTG", 1, 50000.0, "AED"), fee_row("p2", "PTG", 1, 10000.0, "AED")];
        let report = aggregate_win_ratios(&projects, &fees, &company_names(&[("PTG", "Petrus Group")]));

        let ptg = &report.clients[0];
        assert_eq!(ptg.cancelled_count, 1);
        assert_eq!(ptg.decided_count, 1, "cancelled is not a competitive decision");
        assert_eq!(ptg.win_ratio, Some(1.0));
    }

    #[test]
    fn clients_sort_by_decided_count_desc_then_name_as_a_stable_tiebreak() {
        let projects = vec![
            project_row("p1", Some("Won"), None),
            project_row("p2", Some("Won"), None),
            project_row("p3", Some("Won"), None),
        ];
        let fees = vec![
            fee_row("p1", "AAA", 1, 1000.0, "AED"),
            fee_row("p2", "ZZZ", 1, 1000.0, "AED"),
            fee_row("p3", "ZZZ", 1, 1000.0, "AED"),
        ];
        let report = aggregate_win_ratios(
            &projects,
            &fees,
            &company_names(&[("AAA", "Alpha Co"), ("ZZZ", "Zulu Co")]),
        );

        assert_eq!(report.clients[0].company_name, "Zulu Co", "2 decided beats 1 decided");
        assert_eq!(report.clients[1].company_name, "Alpha Co");
    }
    // ---- Review findings (2026-08): broken-lineage drop + fee status ----

    #[test]
    fn a_decided_project_with_a_dangling_successor_pointer_is_unattributed_not_dropped() {
        // p1's successor points at "p2", but no such record exists in this
        // dataset (a realistic shape: successor was set by folder-text
        // matching, not a DB constraint). Before the fix, p1 was silently
        // excluded from outcome_by_project (not terminal) AND never reached
        // by any other project's lookup - it vanished from every counter,
        // including unattributed_decided_count. It must surface there
        // instead, exactly like a decided project with no linked fee does.
        let projects = vec![project_row("p1", Some("Won"), Some("p2"))];
        let fees = vec![fee_row("p1", "PTG", 1, 50000.0, "AED")];
        let report = aggregate_win_ratios(&projects, &fees, &company_names(&[("PTG", "Petrus Group")]));

        assert!(report.clients.is_empty(), "no terminal record exists to attribute the win to");
        assert_eq!(
            report.unattributed_decided_count, 1,
            "a Won project with a dangling successor pointer must not vanish silently"
        );
    }

    #[test]
    fn a_pending_project_with_a_dangling_successor_pointer_is_silently_skipped() {
        // Same broken pointer, but no decision has been made yet - matches
        // the existing "pending with no fee" behaviour: not a data gap.
        let projects = vec![project_row("p1", None, Some("p2"))];
        let report = aggregate_win_ratios(&projects, &[], &BTreeMap::new());

        assert!(report.clients.is_empty());
        assert_eq!(report.unattributed_decided_count, 0);
    }

    #[test]
    fn a_self_referencing_successor_is_a_broken_lineage_not_a_valid_terminal() {
        // p1's successor points at itself. Its own outcome is Won, and
        // without cycle detection this record is neither "terminal"
        // (successor is Some) nor reachable by walking a real chain -
        // it must fold into unattributed_decided_count, not vanish.
        let projects = vec![project_row("p1", Some("Won"), Some("p1"))];
        let fees = vec![fee_row("p1", "PTG", 1, 50000.0, "AED")];
        let report = aggregate_win_ratios(&projects, &fees, &company_names(&[("PTG", "Petrus Group")]));

        assert!(report.clients.is_empty());
        assert_eq!(report.unattributed_decided_count, 1);
    }

    #[test]
    fn a_two_record_cycle_is_a_broken_lineage_for_both_records() {
        // p1 -> p2 -> p1: each successor pointer resolves to a real record,
        // so the naive "does the immediate successor exist" check would
        // miss this - only walking the full chain with cycle detection
        // catches it. Both carry a decided outcome, so both surface.
        let projects = vec![
            project_row("p1", Some("Won"), Some("p2")),
            project_row("p2", Some("Lost"), Some("p1")),
        ];
        let fees = vec![
            fee_row("p1", "PTG", 1, 50000.0, "AED"),
            fee_row("p2", "PTG", 1, 40000.0, "AED"),
        ];
        let report = aggregate_win_ratios(&projects, &fees, &company_names(&[("PTG", "Petrus Group")]));

        assert!(report.clients.is_empty());
        assert_eq!(report.unattributed_decided_count, 2);
    }

    #[test]
    fn latest_fee_selection_prefers_accepted_status_over_a_higher_revision_draft() {
        // rev1 is the fee that was actually Accepted at 50k; rev2 is an
        // abandoned Draft renegotiation at 65k that was never accepted.
        // Before the fix, "highest rev wins" picked rev2 and inflated the
        // won total by 15k on a proposal nobody actually agreed to.
        let projects = vec![project_row("p1", Some("Won"), None)];
        let fees = vec![
            fee_row_full("p1", "PTG", 1, 50000.0, "AED", "Accepted", "260101"),
            fee_row_full("p1", "PTG", 2, 65000.0, "AED", "Draft", "260201"),
        ];
        let report = aggregate_win_ratios(&projects, &fees, &company_names(&[("PTG", "Petrus Group")]));

        let ptg = &report.clients[0];
        assert_eq!(
            ptg.won_value,
            vec![CurrencyAmount { currency: "AED".into(), amount: 50000.0 }],
            "the Accepted rev1 amount must win over the higher-rev abandoned Draft"
        );
    }

    #[test]
    fn latest_fee_selection_falls_back_to_highest_rev_when_no_row_is_accepted() {
        // No Accepted row exists at all (e.g. still mid-negotiation) - the
        // highest-revision row is still the best available signal.
        let projects = vec![project_row("p1", Some("No Response"), None)];
        let fees = vec![
            fee_row_full("p1", "PTG", 1, 30000.0, "AED", "Sent", "260101"),
            fee_row_full("p1", "PTG", 2, 35000.0, "AED", "Sent", "260201"),
        ];
        let report = aggregate_win_ratios(&projects, &fees, &company_names(&[("PTG", "Petrus Group")]));

        let ptg = &report.clients[0];
        assert_eq!(ptg.lost_value, vec![CurrencyAmount { currency: "AED".into(), amount: 35000.0 }]);
    }

    #[test]
    fn latest_fee_selection_breaks_an_equal_rev_tie_on_the_later_issue_date() {
        // Two rows at the same revision (a correction re-issued same-day
        // numbering) - the later issue_date represents the fee's true
        // current state.
        let projects = vec![project_row("p1", Some("Won"), None)];
        let fees = vec![
            fee_row_full("p1", "PTG", 1, 40000.0, "AED", "Accepted", "260101"),
            fee_row_full("p1", "PTG", 1, 45000.0, "AED", "Accepted", "260115"),
        ];
        let report = aggregate_win_ratios(&projects, &fees, &company_names(&[("PTG", "Petrus Group")]));

        let ptg = &report.clients[0];
        assert_eq!(ptg.won_value, vec![CurrencyAmount { currency: "AED".into(), amount: 45000.0 }]);
    }
}
