//! Project-level lifecycle operations: merge and cascade delete.
//!
//! These sit above the basic CRUD primitives in `operations.rs` because they
//! coordinate multiple `fee` records in a single atomic transaction, closing
//! the orphaned-fee / double-counted-revision gap in the plain
//! `delete_project` / `batch_delete_entities` paths (those do a bare record
//! delete with no awareness that `fee.project_id` points at the project
//! being removed).
//!
//! Both operations deliberately never touch a fee's record id or its
//! `number` field - only `project_id` and, when a revision-number collision
//! requires it, `rev`. That is what lets a merge skip e-fees-scope's
//! `scope_assembly` / `scope_revision` tables entirely: those key off the
//! fee's record id (see `e-fees-scope/schema.surql`), and a fee's id is
//! never renamed here, so nothing there goes stale. See
//! `db::client::create_fee`'s doc comment for why a fee's id/number embed
//! its *original* project number - that convention is exactly what this
//! module relies on to avoid a cross-service cascade.

use log::info;
use serde::Serialize;
use surrealdb::Error;

use crate::db::client::validate_record_id;
use crate::db::types::record_key_string;

use super::{DatabaseManager, Fee, Project};

// ============================================================================
// RESPONSE TYPES
// ============================================================================

/// One fee's revision-number change as part of a project merge.
/// `old_rev == new_rev` when no renumbering was needed - the common case,
/// since it only happens when the target project already has a fee at that
/// exact revision number.
#[derive(Debug, Clone, Serialize)]
pub struct FeeRevChange {
    pub fee_id: String,
    pub fee_number: String,
    pub old_rev: i64,
    pub new_rev: i64,
}

/// Non-mutating preview of a project merge - render this in a confirmation
/// dialog before calling `merge_projects`.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectMergePreview {
    pub source: Project,
    pub target: Project,
    pub fees_to_move: usize,
    pub rev_changes: Vec<FeeRevChange>,
}

/// Result of a successful project merge.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectMergeOutcome {
    pub target: Project,
    pub fees_moved: usize,
    pub rev_changes: Vec<FeeRevChange>,
    pub source_deleted_id: String,
}

/// Non-mutating preview of a project delete - render this in a confirmation
/// dialog before calling `delete_project_cascade`. `dependent_fees` is empty
/// for a project with no proposals, in which case the delete needs no
/// cascade confirmation.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectDeletePreview {
    pub project: Project,
    pub dependent_fees: Vec<Fee>,
}

/// Result of a successful (possibly cascading) project delete.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectDeleteOutcome {
    pub deleted_project: Project,
    pub deleted_fees: Vec<Fee>,
}

// ============================================================================
// PURE HELPERS
// ============================================================================

/// Compute which revision number each of `source_fees` should end up with
/// once reparented onto a project that already has `target_fees`.
///
/// A source fee keeps its existing `rev` when that number is free on the
/// target; otherwise it is bumped to one past the highest revision number
/// currently in use across both projects. Fees are processed in ascending
/// original-rev order so a multi-fee merge preserves relative revision
/// history instead of shuffling it. This is what keeps the merge from
/// violating the `fee_project_rev` unique index on `(project_id, rev)`
/// (docs/development/DATABASE_SCHEMA.md) once both projects' fees share a
/// single `project_id`.
fn compute_merge_rev_plan(source_fees: &[Fee], target_fees: &[Fee]) -> Vec<FeeRevChange> {
    let mut used: std::collections::BTreeSet<i64> = target_fees.iter().map(|f| f.rev).collect();

    let mut ordered: Vec<&Fee> = source_fees.iter().collect();
    ordered.sort_by_key(|f| f.rev);

    let mut plan = Vec::with_capacity(ordered.len());
    for fee in ordered {
        let old_rev = fee.rev;
        // BTreeSet::insert returns true when the value was NOT already present.
        let new_rev = if used.insert(old_rev) {
            old_rev
        } else {
            let next = used.iter().next_back().copied().unwrap_or(0) + 1;
            used.insert(next);
            next
        };

        let fee_id = fee
            .id
            .as_ref()
            .map(|id| record_key_string(&id.key))
            .unwrap_or_default();

        plan.push(FeeRevChange {
            fee_id,
            fee_number: fee.number.clone(),
            old_rev,
            new_rev,
        });
    }
    plan
}

impl DatabaseManager {
    /// Preview what `merge_projects(source_id, target_id)` would do, without
    /// changing anything.
    pub async fn preview_project_merge(
        &self,
        source_id: &str,
        target_id: &str,
    ) -> Result<ProjectMergePreview, Error> {
        if source_id == target_id {
            return Err(self.invalid_request_error("Cannot merge a project into itself"));
        }

        let source = self.get_project_by_id(source_id).await?.ok_or_else(|| {
            self.not_found_error(&format!("find source project '{}'", source_id))
        })?;
        let target = self.get_project_by_id(target_id).await?.ok_or_else(|| {
            self.not_found_error(&format!("find target project '{}'", target_id))
        })?;

        let source_fees = self.get_fees_for_project(source_id).await?;
        let target_fees = self.get_fees_for_project(target_id).await?;
        let fees_to_move = source_fees.len();
        let rev_changes = compute_merge_rev_plan(&source_fees, &target_fees);

        Ok(ProjectMergePreview {
            source,
            target,
            fees_to_move,
            rev_changes,
        })
    }

    /// Merge `source_id` into `target_id`: reparent every fee proposal from
    /// the source project onto the target (renumbering revisions only where
    /// `compute_merge_rev_plan` finds a collision), then delete the
    /// now-empty source project. Runs as a single SurrealDB transaction so a
    /// partial failure can never leave a fee reparented without the source
    /// being removed, or vice versa - then re-reads the affected records
    /// afterward to confirm the committed state matches the plan, rather
    /// than trusting the mutation call's return value alone.
    ///
    /// Never trusts a caller-supplied preview: recomputes the plan fresh
    /// against current data, since another session may have changed either
    /// project since a preview was rendered.
    pub async fn merge_projects(
        &self,
        source_id: &str,
        target_id: &str,
    ) -> Result<ProjectMergeOutcome, Error> {
        if source_id == target_id {
            return Err(self.invalid_request_error("Cannot merge a project into itself"));
        }

        self.get_project_by_id(source_id).await?.ok_or_else(|| {
            self.not_found_error(&format!("find source project '{}'", source_id))
        })?;
        self.get_project_by_id(target_id).await?.ok_or_else(|| {
            self.not_found_error(&format!("find target project '{}'", target_id))
        })?;

        let source_fees = self.get_fees_for_project(source_id).await?;
        let target_fees = self.get_fees_for_project(target_id).await?;
        let fees_moved = source_fees.len();
        let rev_changes = compute_merge_rev_plan(&source_fees, &target_fees);

        for change in &rev_changes {
            validate_record_id(&change.fee_id, "fee_id")?;
        }
        validate_record_id(source_id, "source_project_id")?;
        validate_record_id(target_id, "target_project_id")?;

        let client = self.get_client()?;

        if !rev_changes.is_empty() {
            let mut stmts = String::from("BEGIN TRANSACTION;\n");
            let mut bindings = serde_json::Map::new();
            bindings.insert(
                "target_key".to_string(),
                serde_json::Value::String(target_id.to_string()),
            );

            for (i, change) in rev_changes.iter().enumerate() {
                let rev_param = format!("rev_{}", i);
                stmts.push_str(&format!(
                    "UPDATE fee:{fee_key} SET project_id = type::record('projects', $target_key), \
                     rev = ${rev_param}, time.updated_at = time::now() RETURN NONE;\n",
                    fee_key = change.fee_id,
                    rev_param = rev_param,
                ));
                bindings.insert(rev_param, serde_json::json!(change.new_rev));
            }

            stmts.push_str(&format!("DELETE projects:{} RETURN NONE;\n", source_id));
            stmts.push_str("COMMIT TRANSACTION;");

            client.query_bind_map(&stmts, bindings).await?;
        } else {
            // No fees to move - just remove the (empty) source project.
            let query = format!("DELETE projects:{} RETURN NONE;", source_id);
            client.query(&query).await?;
        }

        // Verify from the affected vantage rather than trusting the
        // transaction's return value: re-read every record the merge was
        // supposed to change and confirm the committed state matches the
        // plan exactly.
        if self.get_project_by_id(source_id).await?.is_some() {
            return Err(self.not_found_error(&format!(
                "verify merge: source project '{}' still exists after delete",
                source_id
            )));
        }

        for change in &rev_changes {
            let moved: Fee = self
                .get_by_id("fee", &change.fee_id)
                .await?
                .ok_or_else(|| {
                    self.not_found_error(&format!(
                        "verify merge: fee '{}' missing after reparent",
                        change.fee_id
                    ))
                })?;
            let moved_project_key = record_key_string(&moved.project_id.key);
            if moved_project_key != target_id || moved.rev != change.new_rev {
                return Err(self.invalid_request_error(&format!(
                    "verify merge: fee '{}' expected project '{}' rev {} but found project '{}' rev {}",
                    change.fee_id, target_id, change.new_rev, moved_project_key, moved.rev
                )));
            }
        }

        let target = self.get_project_by_id(target_id).await?.ok_or_else(|| {
            self.not_found_error(&format!(
                "verify merge: re-fetch target project '{}'",
                target_id
            ))
        })?;

        info!(
            "Merged project '{}' into '{}': {} fee(s) moved, {} renumbered",
            source_id,
            target_id,
            fees_moved,
            rev_changes.iter().filter(|c| c.old_rev != c.new_rev).count()
        );

        Ok(ProjectMergeOutcome {
            target,
            fees_moved,
            rev_changes,
            source_deleted_id: source_id.to_string(),
        })
    }

    /// Preview what `delete_project_cascade(project_id, true)` would remove,
    /// without changing anything. `dependent_fees` is empty when the
    /// project has no proposals - in that case cascade confirmation isn't
    /// needed.
    pub async fn preview_project_delete(
        &self,
        project_id: &str,
    ) -> Result<ProjectDeletePreview, Error> {
        let project = self
            .get_project_by_id(project_id)
            .await?
            .ok_or_else(|| self.not_found_error(&format!("find project '{}'", project_id)))?;
        let dependent_fees = self.get_fees_for_project(project_id).await?;

        Ok(ProjectDeletePreview {
            project,
            dependent_fees,
        })
    }

    /// Delete a project. If it has fee proposals, `cascade` must be `true`
    /// or the call is refused (no orphaned `fee.project_id` references) -
    /// callers should render `preview_project_delete`'s `dependent_fees` in
    /// a confirmation dialog before setting `cascade: true`. Deletes the
    /// project and all its fees in a single transaction, then verifies by
    /// re-reading that every targeted record is actually gone.
    ///
    /// Deliberately matches the existing single-fee `delete_fee` in scope:
    /// like that command, this does not reach into e-fees-scope's
    /// `scope_assembly` / `scope_revision` tables (a pre-existing gap, not
    /// introduced here - see the module doc comment).
    pub async fn delete_project_cascade(
        &self,
        project_id: &str,
        cascade: bool,
    ) -> Result<ProjectDeleteOutcome, Error> {
        let project = self
            .get_project_by_id(project_id)
            .await?
            .ok_or_else(|| self.not_found_error(&format!("find project '{}'", project_id)))?;
        let dependent_fees = self.get_fees_for_project(project_id).await?;

        if !dependent_fees.is_empty() && !cascade {
            return Err(self.invalid_request_error(&format!(
                "Project '{}' has {} fee proposal(s); pass cascade=true to delete them together with the project",
                project_id,
                dependent_fees.len()
            )));
        }

        validate_record_id(project_id, "project_id")?;
        let fee_ids: Vec<String> = dependent_fees
            .iter()
            .filter_map(|f| f.id.as_ref().map(|id| record_key_string(&id.key)))
            .collect();
        for fee_id in &fee_ids {
            validate_record_id(fee_id, "fee_id")?;
        }

        let client = self.get_client()?;

        if fee_ids.is_empty() {
            let query = format!("DELETE projects:{} RETURN NONE;", project_id);
            client.query(&query).await?;
        } else {
            let mut stmts = String::from("BEGIN TRANSACTION;\n");
            for fee_id in &fee_ids {
                stmts.push_str(&format!("DELETE fee:{} RETURN NONE;\n", fee_id));
            }
            stmts.push_str(&format!("DELETE projects:{} RETURN NONE;\n", project_id));
            stmts.push_str("COMMIT TRANSACTION;");
            client.query(&stmts).await?;
        }

        // Verify from the affected vantage: confirm every targeted record
        // is actually gone rather than trusting the transaction succeeded.
        if self.get_project_by_id(project_id).await?.is_some() {
            return Err(self.not_found_error(&format!(
                "verify delete: project '{}' still exists after delete",
                project_id
            )));
        }
        for fee_id in &fee_ids {
            let still_there: Option<Fee> = self.get_by_id("fee", fee_id).await?;
            if still_there.is_some() {
                return Err(self.invalid_request_error(&format!(
                    "verify delete: fee '{}' still exists after cascade delete",
                    fee_id
                )));
            }
        }

        info!(
            "Deleted project '{}' with {} cascaded fee proposal(s)",
            project_id,
            dependent_fees.len()
        );

        Ok(ProjectDeleteOutcome {
            deleted_project: project,
            deleted_fees: dependent_fees,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::types::TimeStamps;
    use surrealdb::types::RecordId;
    use surrealdb_types::Datetime;

    fn make_fee(id_key: &str, rev: i64) -> Fee {
        Fee {
            id: Some(RecordId::new("fee", id_key)),
            name: String::new(),
            number: format!("test-{}", rev),
            rev,
            status: "Draft".to_string(),
            issue_date: "260101".to_string(),
            activity: String::new(),
            package: String::new(),
            project_id: RecordId::new("projects", "source"),
            company_id: RecordId::new("company", "test"),
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
        }
    }

    #[test]
    fn no_collision_keeps_original_revs() {
        let source = vec![make_fee("s1", 1), make_fee("s2", 2)];
        let target: Vec<Fee> = vec![]; // no overlapping revs on the target side

        let plan = compute_merge_rev_plan(&source, &target);
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].fee_id, "s1");
        assert_eq!(plan[0].old_rev, 1);
        assert_eq!(plan[0].new_rev, 1);
        assert_eq!(plan[1].fee_id, "s2");
        assert_eq!(plan[1].old_rev, 2);
        assert_eq!(plan[1].new_rev, 2);
    }

    #[test]
    fn collision_bumps_to_next_free_rev() {
        // Target already has rev 1; source's rev-1 fee must be renumbered,
        // never silently dropped or double-counted at the same rev.
        let source = vec![make_fee("s1", 1)];
        let target = vec![make_fee("t1", 1)];

        let plan = compute_merge_rev_plan(&source, &target);
        assert_eq!(plan.len(), 1);
        assert_eq!(plan[0].old_rev, 1);
        assert_eq!(plan[0].new_rev, 2, "rev 1 is taken on target, must bump to 2");
    }

    #[test]
    fn multiple_collisions_preserve_relative_order() {
        // Target already has revs 1 and 2. Source has two fees at rev 1 and
        // rev 3 (in that DB order); after remap the rev-1 fee must land
        // above the rev-2 fee that was already there, and the two source
        // fees' relative order (1 before 3) must be preserved.
        let source = vec![make_fee("s_rev1", 1), make_fee("s_rev3", 3)];
        let target = vec![make_fee("t1", 1), make_fee("t2", 2)];

        let plan = compute_merge_rev_plan(&source, &target);
        assert_eq!(plan.len(), 2);

        let s_rev1 = plan.iter().find(|c| c.fee_id == "s_rev1").unwrap();
        let s_rev3 = plan.iter().find(|c| c.fee_id == "s_rev3").unwrap();

        assert_eq!(s_rev1.new_rev, 3, "rev 1 collides, bumps past target's max (2) to 3");
        assert_eq!(s_rev3.new_rev, 3 + 1, "rev 3 now collides with the just-assigned 3, bumps to 4");

        // No two changes may share a new_rev - that's the exact
        // double-counting failure this function exists to prevent.
        let mut new_revs: Vec<i64> = plan.iter().map(|c| c.new_rev).collect();
        new_revs.sort_unstable();
        new_revs.dedup();
        assert_eq!(new_revs.len(), plan.len(), "no two fees may end up at the same rev");
    }

    #[test]
    fn empty_source_produces_empty_plan() {
        let plan = compute_merge_rev_plan(&[], &[make_fee("t1", 1)]);
        assert!(plan.is_empty());
    }

    #[test]
    fn merging_into_empty_target_keeps_all_original_revs() {
        let source = vec![make_fee("s1", 1), make_fee("s2", 5)];
        let plan = compute_merge_rev_plan(&source, &[]);
        assert_eq!(plan[0].new_rev, 1);
        assert_eq!(plan[1].new_rev, 5);
    }
}
