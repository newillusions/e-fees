//! Folder Reconcile Commands
//!
//! Reconciles on-disk project folders after a DB-level project merge or
//! cascade-delete (see `db/project_lifecycle.rs`). Those two operations are
//! deliberately DB-only and never touch the filesystem - this module is the
//! separate, OPTIONAL, purely filesystem-level follow-up a user may choose
//! to run afterward. A cancelled or failed reconcile here must never roll
//! back the (already-committed) DB merge/delete: the two are decoupled by
//! design, and this module has no code path that can undo either.
//!
//! ## Safety invariants (non-negotiable, per design review obs:v74ffyd1v6n1go8kf4c7)
//! - **Never a silent overwrite.** `keep_both` renames the INCOMING file,
//!   never clobbers the existing one. `overwrite` first moves the
//!   pre-existing destination file into `.reconcile-backups/{timestamp}/` -
//!   a location OUTSIDE the 4 status directories (`00 Inactive`,
//!   `01 RFPs`, `11 Current`, `99 Completed`), so `folder_sync.rs`'s
//!   `is_project_folder()` pattern (only matches folder NAMES directly
//!   inside one of those 4 dirs) never mistakes a backup for an orphan
//!   project folder.
//! - **Dry-run before any write.** `execute_folder_reconcile` and
//!   `execute_trash_project_folder`'s preview counterpart both let a caller
//!   compute exactly what WOULD happen (`dry_run: true`) before doing it -
//!   the frontend must show that summary and get an explicit confirm
//!   before calling again with `dry_run: false`.
//! - **Conflict detection is size-first** (Martin's decision, 2026-08-18):
//!   file size is compared first as a cheap short-circuit; a SHA-256
//!   content hash is computed ONLY when sizes are equal, to distinguish an
//!   identical file from a same-size collision. Never mtime - project
//!   folders may sit on a slow/lossy network mount (matches this
//!   workspace's standing mtime-distrust pattern).
//! - **Every write re-validates against current reality at execute time**,
//!   never trusting a caller-supplied preview: a `copy` whose destination
//!   now exists, or an `overwrite`/`keep_both` whose destination no longer
//!   exists, is refused rather than silently reinterpreted.
//! - **Files are paired by project-number-agnostic identity, not raw path.**
//!   `create_project_with_template`/`copy_project_template`
//!   (`commands/template_ops.rs`) stage every project folder from a
//!   `_yy-cccnn Project Name` template and then call
//!   `rename_template_files_cross_platform(&dest_path, "yy-cccnn",
//!   &project_number)`, which does a literal substring replace of the
//!   `"yy-cccnn"` token with the real (hyphenated) project number in every
//!   file AND directory name under the copied tree - e.g. the template's
//!   `"yy-cccnn-var Default Values.json"` becomes
//!   `"26-97110-var Default Values.json"`. That means the SAME logical
//!   base/template file carries a DIFFERENT literal name in every project
//!   folder it was ever created in - pairing purely by relative path would
//!   report a renamed base file as two unrelated NEW files instead of one
//!   real conflict/identical pair, and would leave a stray source-numbered
//!   file sitting in the target folder after a copy. `canonical_dest_relpath`
//!   reverses that exact substitution (source number -> target number, per
//!   path component) so classification, conflict resolution, and the
//!   destination filename actually written all agree with the naming
//!   convention. Plain user-added files (no project-number substring) are
//!   unaffected and keep pairing by literal relative path, as before.

use chrono::{DateTime, NaiveDateTime, Utc};
use log::info;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{command, AppHandle};
use walkdir::WalkDir;

use super::folder_management::{find_project_folder, get_projects_base_path, move_folder_cross_platform};

/// Name of the directory (relative to the project-folder base path, or to a
/// reconciled folder's own base-of-status-dirs) that backups are written
/// under. Deliberately dot-prefixed and outside the 4 status dirs.
const BACKUP_DIR_NAME: &str = ".reconcile-backups";

/// Default age threshold (days) for `preview_backup_cleanup` /
/// `execute_backup_cleanup` when the caller doesn't pass `cutoff_days`
/// explicitly. A backup exactly this many days old is KEPT - only one
/// STRICTLY older is eligible (see `backup_is_eligible`).
const DEFAULT_BACKUP_CLEANUP_CUTOFF_DAYS: i64 = 30;

// ============================================================================
// TYPES - preview_folder_reconcile / execute_folder_reconcile
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconcileFileStatus {
    /// Exists in the source folder only.
    New,
    /// Exists in both folders with a different size, or same size but a
    /// differing content hash.
    Conflict,
    /// Exists in both folders with the same size and content hash.
    Identical,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReconcileFileEntry {
    /// Forward-slash-normalized path relative to the SOURCE folder root.
    pub relative_path: String,
    /// Forward-slash-normalized path relative to the TARGET folder root -
    /// where this file actually lands. Equal to `relative_path` unless the
    /// name carries the source project's number, in which case it's been
    /// re-numbered to the target project's number (see module doc comment).
    pub dest_relative_path: String,
    pub status: ReconcileFileStatus,
    pub source_size: u64,
    pub dest_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FolderReconcilePreview {
    pub source_path: String,
    pub target_path: String,
    pub entries: Vec<ReconcileFileEntry>,
    pub new_count: usize,
    pub conflict_count: usize,
    pub identical_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconcileAction {
    /// NEW file: copy from source into target.
    Copy,
    /// CONFLICT: back up the existing destination file, then copy source over it.
    Overwrite,
    /// CONFLICT: copy source in under a renamed, non-clobbering name.
    KeepBoth,
    /// Do nothing.
    Skip,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReconcileResolution {
    pub relative_path: String,
    pub action: ReconcileAction,
}

#[derive(Debug, Clone, Serialize)]
pub struct FolderReconcileOutcome {
    pub dry_run: bool,
    pub copied: usize,
    pub overwritten: usize,
    pub kept_both: usize,
    pub skipped: usize,
    /// Absolute paths of every backup file written (or that WOULD be
    /// written, when `dry_run: true`).
    pub backups_created: Vec<String>,
    /// Per-file failures. A non-empty list does not mean the whole call
    /// failed - other resolutions may have succeeded; the caller should
    /// surface these for follow-up.
    pub errors: Vec<String>,
}

// ============================================================================
// TYPES - preview_trash_project_folder / execute_trash_project_folder
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct TrashFolderPreview {
    pub project_number: String,
    pub folder_exists: bool,
    pub source_path: Option<String>,
    pub file_count: u64,
    pub total_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrashFolderOutcome {
    pub project_number: String,
    pub moved: bool,
    pub backup_path: Option<String>,
    pub message: String,
}

// ============================================================================
// TYPES - preview_backup_cleanup / execute_backup_cleanup
// ============================================================================

/// One `.reconcile-backups/{timestamp}/` directory eligible for cleanup.
/// Both `execute_folder_reconcile`'s `overwrite` action and
/// `execute_trash_project_folder` write under the same
/// `{PROJECT_FOLDER_PATH}/.reconcile-backups/{timestamp}/` root (see
/// `backup_dir_for` and `execute_trash_project_folder`'s doc comment), so
/// one scan covers backups from either source.
#[derive(Debug, Clone, Serialize)]
pub struct BackupCleanupEntry {
    pub timestamp: String,
    pub path: String,
    pub age_days: i64,
    pub file_count: u64,
    pub total_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupCleanupPreview {
    pub base_path: String,
    pub cutoff_days: i64,
    pub eligible: Vec<BackupCleanupEntry>,
    pub eligible_count: usize,
    pub total_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupCleanupOutcome {
    pub dry_run: bool,
    pub deleted_count: usize,
    pub freed_bytes: u64,
    /// Per-directory failures. A non-empty list does not mean the whole
    /// call failed - other deletions may have succeeded.
    pub errors: Vec<String>,
}

// ============================================================================
// PURE / FS HELPERS
// ============================================================================

fn validate_existing_dir(path_str: &str) -> Result<PathBuf, String> {
    if path_str.trim().is_empty() {
        return Err("Path cannot be empty".to_string());
    }
    let path = PathBuf::from(path_str);
    if !path.is_absolute() {
        return Err(format!("Path must be absolute: {}", path_str));
    }
    if !path.exists() || !path.is_dir() {
        return Err(format!("Directory does not exist: {}", path_str));
    }
    Ok(path)
}

/// Validates a resolution's `relative_path` is a genuinely relative,
/// traversal-free path - never absolute, never containing a `..`
/// component. Rejecting these up front (before any write in a batch)
/// is what keeps a malformed entry from being silently reinterpreted.
fn safe_relative_path(rel: &str) -> Result<PathBuf, String> {
    if rel.trim().is_empty() {
        return Err("relative_path cannot be empty".to_string());
    }
    let candidate = Path::new(rel);
    if candidate.is_absolute() {
        return Err(format!("relative_path must be relative: {}", rel));
    }
    if candidate
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(format!("relative_path must not contain '..': {}", rel));
    }
    Ok(candidate.to_path_buf())
}

/// Compute where a source-relative path `rel` should actually land in the
/// target folder, given `source_number` (the source project's hyphenated
/// number, e.g. `"26-97110"`) and `target_number` (the target's). Reverses
/// `rename_template_files_cross_platform`'s `"yy-cccnn"` -> real-number
/// substitution: any path COMPONENT (file or directory name) that contains
/// `source_number` as a literal substring gets that substring replaced with
/// `target_number`; components with no match pass through unchanged. A
/// no-op (returns `rel` untouched) when either number is empty or the two
/// numbers are equal - guards against `str::replace` with an empty pattern,
/// which would insert `target_number` between every character instead of
/// leaving the path alone.
fn canonical_dest_relpath(rel: &Path, source_number: &str, target_number: &str) -> PathBuf {
    if source_number.is_empty() || target_number.is_empty() || source_number == target_number {
        return rel.to_path_buf();
    }

    let mut out = PathBuf::new();
    for component in rel.components() {
        match component {
            std::path::Component::Normal(os_str) => {
                let s = os_str.to_string_lossy();
                if s.contains(source_number) {
                    out.push(s.replace(source_number, target_number));
                } else {
                    out.push(os_str);
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file =
        fs::File::open(path).map_err(|e| format!("Failed to open '{}': {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)
        .map_err(|e| format!("Failed to hash '{}': {}", path.display(), e))?;
    Ok(format!("{:x}", hasher.finalize()))
}

/// Classify one source file against its would-be destination. Size-first
/// per Martin's decision: only reads (and hashes) file content when the
/// sizes already match - a real collision is settled by size alone without
/// ever touching the file bytes.
fn classify_entry(
    source_file: &Path,
    dest_file: &Path,
) -> Result<(ReconcileFileStatus, u64, Option<u64>), String> {
    let source_size = fs::metadata(source_file)
        .map_err(|e| format!("Failed to stat '{}': {}", source_file.display(), e))?
        .len();

    if !dest_file.exists() {
        return Ok((ReconcileFileStatus::New, source_size, None));
    }

    let dest_size = fs::metadata(dest_file)
        .map_err(|e| format!("Failed to stat '{}': {}", dest_file.display(), e))?
        .len();

    if source_size != dest_size {
        return Ok((ReconcileFileStatus::Conflict, source_size, Some(dest_size)));
    }

    // Same size - only now is a full-content hash justified.
    let source_hash = sha256_file(source_file)?;
    let dest_hash = sha256_file(dest_file)?;
    let status = if source_hash == dest_hash {
        ReconcileFileStatus::Identical
    } else {
        ReconcileFileStatus::Conflict
    };
    Ok((status, source_size, Some(dest_size)))
}

fn move_file_cross_platform(from: &Path, to: &Path) -> Result<(), String> {
    match fs::rename(from, to) {
        Ok(_) => Ok(()),
        Err(_) => {
            // Cross-filesystem fallback, mirroring
            // folder_management::move_folder_cross_platform's approach but
            // for a single file rather than a whole directory tree.
            fs::copy(from, to)
                .map_err(|e| format!("Failed to copy '{}' to '{}': {}", from.display(), to.display(), e))?;
            fs::remove_file(from)
                .map_err(|e| format!("Failed to remove original '{}': {}", from.display(), e))?;
            Ok(())
        }
    }
}

/// Compute a non-clobbering destination path for a "keep both" copy: append
/// " (from source)" before the extension, then a numeric suffix if that is
/// also taken. Never overwrites - loops until a free name is found.
fn keep_both_path(dest_file: &Path) -> PathBuf {
    let parent = dest_file.parent().unwrap_or_else(|| Path::new(""));
    let stem = dest_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = dest_file.extension().and_then(|s| s.to_str());

    let build_name = |suffix: &str| -> String {
        match ext {
            Some(ext) => format!("{} (from source{}).{}", stem, suffix, ext),
            None => format!("{} (from source{})", stem, suffix),
        }
    };

    let mut candidate = parent.join(build_name(""));
    let mut n = 2;
    while candidate.exists() {
        candidate = parent.join(build_name(&format!(" {}", n)));
        n += 1;
    }
    candidate
}

/// Resolve the `.reconcile-backups/{timestamp}/` directory for a reconcile
/// targeting `target_root` (a project folder inside one of the 4 status
/// dirs) - i.e. `target_root`'s grandparent, which is the project-folder
/// base path itself, sitting OUTSIDE every status dir. Returns `None` when
/// `target_root` doesn't have two ancestors (defensive - refuses rather
/// than guessing a backup location).
fn backup_dir_for(target_root: &Path, timestamp: &str) -> Option<PathBuf> {
    target_root
        .parent()
        .and_then(|status_dir| status_dir.parent())
        .map(|base| base.join(BACKUP_DIR_NAME).join(timestamp))
}

fn reconcile_timestamp() -> String {
    Utc::now().format("%Y%m%dT%H%M%SZ").to_string()
}

/// Parse a `.reconcile-backups/` subdirectory name back into the instant it
/// was created, using the exact format `reconcile_timestamp()` writes
/// (`%Y%m%dT%H%M%SZ`). Returns `None` for anything that doesn't match -
/// defensive, so a stray non-conforming entry under that directory (a
/// future format change, something dropped there by hand) is silently
/// skipped by the cleanup scan rather than mis-parsed or deleted.
fn parse_backup_timestamp(name: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(name, "%Y%m%dT%H%M%SZ")
        .ok()
        .map(|naive| DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
}

/// Whether a backup created at `created_at` is eligible for cleanup as of
/// `now`, given a `cutoff_days` threshold. Strictly-greater-than: a backup
/// exactly `cutoff_days` old is KEPT, only one genuinely OLDER than the
/// cutoff is eligible. Takes `now` as a parameter rather than calling
/// `Utc::now()` internally so the boundary is exactly testable, same
/// reasoning as every other pure/fs helper in this file.
fn backup_is_eligible(created_at: DateTime<Utc>, now: DateTime<Utc>, cutoff_days: i64) -> bool {
    now.signed_duration_since(created_at) > chrono::Duration::days(cutoff_days)
}

/// Total file count and byte size under `root` (recursive).
fn dir_stats(root: &Path) -> (u64, u64) {
    let mut file_count = 0u64;
    let mut total_size_bytes = 0u64;
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            file_count += 1;
            if let Ok(meta) = entry.metadata() {
                total_size_bytes += meta.len();
            }
        }
    }
    (file_count, total_size_bytes)
}

/// List every `.reconcile-backups/{timestamp}/` directory under
/// `base_path` eligible for cleanup as of `now`, given `cutoff_days`. Pure
/// `base_path` + fs, no `AppHandle` - directly unit-testable, mirroring
/// this file's existing split of pure fs logic (`classify_entry`,
/// `backup_dir_for`) from the thin `#[command]` glue that resolves
/// `AppHandle` -> `base_path`. Returns an empty list (not an error) when
/// `.reconcile-backups/` doesn't exist yet - "no backups" is a normal
/// outcome, not exceptional.
fn list_backup_entries(
    base_path: &Path,
    cutoff_days: i64,
    now: DateTime<Utc>,
) -> Result<Vec<BackupCleanupEntry>, String> {
    let backups_root = base_path.join(BACKUP_DIR_NAME);
    if !backups_root.exists() {
        return Ok(Vec::new());
    }

    let read_dir = fs::read_dir(&backups_root)
        .map_err(|e| format!("Failed to read '{}': {}", backups_root.display(), e))?;

    let mut entries = Vec::new();
    for dir_entry in read_dir {
        let dir_entry = dir_entry.map_err(|e| format!("Failed to read backup entry: {}", e))?;
        let path = dir_entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(created_at) = parse_backup_timestamp(name) else {
            continue;
        };
        if !backup_is_eligible(created_at, now, cutoff_days) {
            continue;
        }

        let (file_count, total_size_bytes) = dir_stats(&path);
        entries.push(BackupCleanupEntry {
            timestamp: name.to_string(),
            path: path.to_string_lossy().to_string(),
            age_days: now.signed_duration_since(created_at).num_days(),
            file_count,
            total_size_bytes,
        });
    }

    entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    Ok(entries)
}

/// Delete (or, with `dry_run: true`, simulate deleting) every entry in
/// `entries`. Pure fs operation over an already-computed entry list - no
/// `AppHandle` needed, so real deletion behavior (not just the listing) is
/// directly unit-testable. A failure on one entry is recorded and the loop
/// continues, matching `execute_folder_reconcile`'s per-file error
/// accumulation contract.
fn delete_backup_dirs(entries: &[BackupCleanupEntry], dry_run: bool) -> (usize, u64, Vec<String>) {
    let mut deleted_count = 0usize;
    let mut freed_bytes = 0u64;
    let mut errors = Vec::new();

    for entry in entries {
        if !dry_run {
            if let Err(e) = fs::remove_dir_all(&entry.path) {
                errors.push(format!("Failed to delete '{}': {}", entry.path, e));
                continue;
            }
        }
        deleted_count += 1;
        freed_bytes += entry.total_size_bytes;
    }

    (deleted_count, freed_bytes, errors)
}

// ============================================================================
// COMMANDS - folder reconcile (merge follow-up)
// ============================================================================

/// Walk `source_path` and classify every file against `target_path`. Purely
/// read-only - safe to call repeatedly while the user adjusts resolutions
/// in the UI. `source_number`/`target_number` (hyphenated project numbers,
/// e.g. `"26-97110"`) drive the project-number-agnostic file pairing
/// described in the module doc comment - pass empty strings to disable it
/// and pair strictly by relative path.
#[command]
pub async fn preview_folder_reconcile(
    source_path: String,
    target_path: String,
    source_number: String,
    target_number: String,
) -> Result<FolderReconcilePreview, String> {
    let source_root = validate_existing_dir(&source_path)?;
    let target_root = validate_existing_dir(&target_path)?;

    let mut entries = Vec::new();
    let mut new_count = 0usize;
    let mut conflict_count = 0usize;
    let mut identical_count = 0usize;

    for entry in WalkDir::new(&source_root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(&source_root)
            .map_err(|e| format!("Path error for '{}': {}", entry.path().display(), e))?;

        // Defensive: a previous reconcile's own backup artifact must never
        // be surfaced as something to reconcile again.
        if rel
            .components()
            .any(|c| c.as_os_str() == BACKUP_DIR_NAME)
        {
            continue;
        }

        let dest_rel = canonical_dest_relpath(rel, &source_number, &target_number);
        let dest_file = target_root.join(&dest_rel);
        let (status, source_size, dest_size) = classify_entry(entry.path(), &dest_file)?;

        match status {
            ReconcileFileStatus::New => new_count += 1,
            ReconcileFileStatus::Conflict => conflict_count += 1,
            ReconcileFileStatus::Identical => identical_count += 1,
        }

        entries.push(ReconcileFileEntry {
            relative_path: rel.to_string_lossy().replace('\\', "/"),
            dest_relative_path: dest_rel.to_string_lossy().replace('\\', "/"),
            status,
            source_size,
            dest_size,
        });
    }

    Ok(FolderReconcilePreview {
        source_path: source_root.to_string_lossy().to_string(),
        target_path: target_root.to_string_lossy().to_string(),
        entries,
        new_count,
        conflict_count,
        identical_count,
    })
}

/// Apply (or, with `dry_run: true`, simulate) a set of per-file resolutions
/// from a prior `preview_folder_reconcile` call. Every `relative_path` is
/// validated as traversal-free BEFORE any write happens, in either mode -
/// one unsafe entry aborts the whole call rather than partially applying
/// around it. Re-validates each file's expected state (dest exists /
/// doesn't) at execute time rather than trusting the preview, since the
/// filesystem may have changed since it was rendered. `source_number`/
/// `target_number` are RECOMPUTED into each destination path fresh here
/// (never trusts a client-supplied destination) - same convention as
/// `preview_folder_reconcile`, so a `copy` of a source-numbered file lands
/// under the target's number, not the source's.
#[command]
pub async fn execute_folder_reconcile(
    source_path: String,
    target_path: String,
    resolutions: Vec<ReconcileResolution>,
    dry_run: bool,
    source_number: String,
    target_number: String,
) -> Result<FolderReconcileOutcome, String> {
    let source_root = validate_existing_dir(&source_path)?;
    let target_root = validate_existing_dir(&target_path)?;

    let mut safe_resolutions = Vec::with_capacity(resolutions.len());
    for r in &resolutions {
        let rel = safe_relative_path(&r.relative_path)?;
        let dest_rel = canonical_dest_relpath(&rel, &source_number, &target_number);
        safe_resolutions.push((rel, dest_rel, r.action));
    }

    let mut outcome = FolderReconcileOutcome {
        dry_run,
        copied: 0,
        overwritten: 0,
        kept_both: 0,
        skipped: 0,
        backups_created: Vec::new(),
        errors: Vec::new(),
    };

    // One timestamp per call, so every backup this batch produces lands
    // under the same .reconcile-backups/{timestamp}/ directory.
    let timestamp = reconcile_timestamp();
    let backup_dir = backup_dir_for(&target_root, &timestamp);

    for (rel, dest_rel, action) in safe_resolutions {
        let source_file = source_root.join(&rel);
        let dest_file = target_root.join(&dest_rel);

        match action {
            ReconcileAction::Skip => {
                outcome.skipped += 1;
            }

            ReconcileAction::Copy => {
                if dest_file.exists() {
                    outcome.errors.push(format!(
                        "Refusing copy for '{}': destination now exists (changed since preview)",
                        rel.display()
                    ));
                    continue;
                }
                if !dry_run {
                    if let Some(parent) = dest_file.parent() {
                        if let Err(e) = fs::create_dir_all(parent) {
                            outcome
                                .errors
                                .push(format!("Failed to create '{}': {}", parent.display(), e));
                            continue;
                        }
                    }
                    if let Err(e) = fs::copy(&source_file, &dest_file) {
                        outcome
                            .errors
                            .push(format!("Failed to copy '{}': {}", rel.display(), e));
                        continue;
                    }
                }
                outcome.copied += 1;
            }

            ReconcileAction::Overwrite => {
                if !dest_file.exists() {
                    outcome.errors.push(format!(
                        "Refusing overwrite for '{}': destination no longer exists (changed since preview)",
                        rel.display()
                    ));
                    continue;
                }
                let backup_base = match &backup_dir {
                    Some(b) => b.clone(),
                    None => {
                        outcome.errors.push(format!(
                            "Refusing overwrite for '{}': could not resolve a backup location outside the status directories",
                            rel.display()
                        ));
                        continue;
                    }
                };
                let backup_dest = backup_base.join(&dest_rel);

                if !dry_run {
                    if let Some(parent) = backup_dest.parent() {
                        if let Err(e) = fs::create_dir_all(parent) {
                            outcome.errors.push(format!(
                                "Failed to create backup dir for '{}': {}",
                                rel.display(),
                                e
                            ));
                            continue;
                        }
                    }
                    if let Err(e) = move_file_cross_platform(&dest_file, &backup_dest) {
                        outcome.errors.push(format!(
                            "Failed to back up '{}' before overwrite: {}",
                            rel.display(),
                            e
                        ));
                        continue;
                    }
                    if let Some(parent) = dest_file.parent() {
                        if let Err(e) = fs::create_dir_all(parent) {
                            outcome
                                .errors
                                .push(format!("Failed to create '{}': {}", parent.display(), e));
                            continue;
                        }
                    }
                    if let Err(e) = fs::copy(&source_file, &dest_file) {
                        outcome.errors.push(format!(
                            "Failed to copy '{}' over backed-up destination: {}",
                            rel.display(),
                            e
                        ));
                        continue;
                    }
                }
                outcome
                    .backups_created
                    .push(backup_dest.to_string_lossy().to_string());
                outcome.overwritten += 1;
            }

            ReconcileAction::KeepBoth => {
                if !dest_file.exists() {
                    outcome.errors.push(format!(
                        "Refusing keep-both for '{}': destination no longer exists (changed since preview) - use copy instead",
                        rel.display()
                    ));
                    continue;
                }
                if !dry_run {
                    let renamed_dest = keep_both_path(&dest_file);
                    if let Some(parent) = renamed_dest.parent() {
                        if let Err(e) = fs::create_dir_all(parent) {
                            outcome
                                .errors
                                .push(format!("Failed to create '{}': {}", parent.display(), e));
                            continue;
                        }
                    }
                    if let Err(e) = fs::copy(&source_file, &renamed_dest) {
                        outcome.errors.push(format!(
                            "Failed to copy '{}' as a keep-both file: {}",
                            rel.display(),
                            e
                        ));
                        continue;
                    }
                }
                outcome.kept_both += 1;
            }
        }
    }

    if !dry_run {
        info!(
            "Folder reconcile executed: {} -> {} ({} copied, {} overwritten, {} kept-both, {} skipped, {} error(s))",
            source_root.display(),
            target_root.display(),
            outcome.copied,
            outcome.overwritten,
            outcome.kept_both,
            outcome.skipped,
            outcome.errors.len()
        );
    }

    Ok(outcome)
}

// ============================================================================
// COMMANDS - trash-on-delete (cascade-delete follow-up)
// ============================================================================

/// Preview what `execute_trash_project_folder` would move. Read-only -
/// resolves the project's on-disk folder (if any) and totals its file
/// count / size for the cascade-delete confirmation dialog.
#[command]
pub async fn preview_trash_project_folder(
    app_handle: AppHandle,
    project_number: String,
) -> Result<TrashFolderPreview, String> {
    let info = find_project_folder(&app_handle, &project_number).await?;

    if !info.exists {
        return Ok(TrashFolderPreview {
            project_number,
            folder_exists: false,
            source_path: None,
            file_count: 0,
            total_size_bytes: 0,
        });
    }

    let root = PathBuf::from(&info.full_path);
    let mut file_count = 0u64;
    let mut total_size_bytes = 0u64;
    for entry in WalkDir::new(&root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            file_count += 1;
            if let Ok(meta) = entry.metadata() {
                total_size_bytes += meta.len();
            }
        }
    }

    Ok(TrashFolderPreview {
        project_number,
        folder_exists: true,
        source_path: Some(info.full_path),
        file_count,
        total_size_bytes,
    })
}

/// Move a deleted project's on-disk folder to
/// `{PROJECT_FOLDER_PATH}/.reconcile-backups/{timestamp}/{folder_name}`
/// rather than leaving an orphan or hard-deleting it. Opt-in only - the
/// caller decides whether to invoke this at all (default OFF, per Martin's
/// decision) - and it is called AFTER `delete_project_cascade` has already
/// committed, so a failure here never blocks or reverses the DB delete.
/// Re-resolves the folder location fresh rather than trusting a preview.
#[command]
pub async fn execute_trash_project_folder(
    app_handle: AppHandle,
    project_number: String,
) -> Result<TrashFolderOutcome, String> {
    let info = find_project_folder(&app_handle, &project_number).await?;

    if !info.exists {
        return Ok(TrashFolderOutcome {
            project_number,
            moved: false,
            backup_path: None,
            message: "No on-disk folder found for this project - nothing to trash.".to_string(),
        });
    }

    let base_path = get_projects_base_path(&app_handle).await?;
    let source = PathBuf::from(&info.full_path);
    let folder_name = source
        .file_name()
        .ok_or_else(|| "Could not determine folder name".to_string())?;

    let timestamp = reconcile_timestamp();
    let backup_dir = base_path.join(BACKUP_DIR_NAME).join(&timestamp);
    fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;
    let dest = backup_dir.join(folder_name);

    move_folder_cross_platform(&source, &dest)?;

    info!(
        "Trashed on-disk folder for project {}: {} -> {}",
        project_number,
        info.full_path,
        dest.display()
    );

    Ok(TrashFolderOutcome {
        project_number,
        moved: true,
        backup_path: Some(dest.to_string_lossy().to_string()),
        message: format!("Folder moved to {}", dest.display()),
    })
}

// ============================================================================
// COMMANDS - backup cleanup (reconcile-backups maintenance)
// ============================================================================

/// List `.reconcile-backups/{timestamp}/` directories older than
/// `cutoff_days` (defaults to `DEFAULT_BACKUP_CLEANUP_CUTOFF_DAYS` when
/// omitted). Read-only - the frontend renders this as a dry-run summary
/// (count + total size) before the user confirms `execute_backup_cleanup`.
#[command]
pub async fn preview_backup_cleanup(
    app_handle: AppHandle,
    cutoff_days: Option<i64>,
) -> Result<BackupCleanupPreview, String> {
    let cutoff_days = cutoff_days.unwrap_or(DEFAULT_BACKUP_CLEANUP_CUTOFF_DAYS);
    let base_path = get_projects_base_path(&app_handle).await?;
    let eligible = list_backup_entries(&base_path, cutoff_days, Utc::now())?;
    let total_size_bytes = eligible.iter().map(|e| e.total_size_bytes).sum();

    Ok(BackupCleanupPreview {
        base_path: base_path.to_string_lossy().to_string(),
        cutoff_days,
        eligible_count: eligible.len(),
        total_size_bytes,
        eligible,
    })
}

/// Permanently delete every `.reconcile-backups/{timestamp}/` directory
/// older than `cutoff_days` (or, with `dry_run: true`, report what WOULD be
/// deleted without touching the filesystem - same dry-run contract as
/// `execute_folder_reconcile`). Re-scans fresh rather than trusting a prior
/// `preview_backup_cleanup` call, so a backup that appeared or aged out of
/// scope between preview and execute is handled correctly instead of acted
/// on stale data. These backups are the LAST safety net for an overwrite
/// or a trashed project folder - deletion here is genuinely permanent, no
/// further backup is taken.
#[command]
pub async fn execute_backup_cleanup(
    app_handle: AppHandle,
    cutoff_days: Option<i64>,
    dry_run: bool,
) -> Result<BackupCleanupOutcome, String> {
    let cutoff_days = cutoff_days.unwrap_or(DEFAULT_BACKUP_CLEANUP_CUTOFF_DAYS);
    let base_path = get_projects_base_path(&app_handle).await?;
    let eligible = list_backup_entries(&base_path, cutoff_days, Utc::now())?;
    let (deleted_count, freed_bytes, errors) = delete_backup_dirs(&eligible, dry_run);

    if !dry_run {
        info!(
            "Backup cleanup executed: {} backup dir(s) deleted, {} bytes freed (cutoff {}d, {} error(s))",
            deleted_count,
            freed_bytes,
            cutoff_days,
            errors.len()
        );
    }

    Ok(BackupCleanupOutcome {
        dry_run,
        deleted_count,
        freed_bytes,
        errors,
    })
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

    /// Self-cleaning temp directory - removed on drop (including on test
    /// panic/assert failure via unwinding), so a failing test never leaves
    /// litter in the OS temp dir. Avoids adding `tempfile` as a new
    /// dev-dependency purely for this module.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let n = TEST_DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
            let path = std::env::temp_dir().join(format!(
                "efees-folder-reconcile-test-{}-{}-{}",
                label,
                std::process::id(),
                n
            ));
            fs::create_dir_all(&path).expect("failed to create test temp dir");
            TempDir(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write_file(dir: &Path, rel: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, content).unwrap();
        path
    }

    // -- classify_entry: size-first, hash only on a size match --------

    #[test]
    fn classify_new_when_dest_missing() {
        let src = TempDir::new("classify-new");
        let f = write_file(src.path(), "a.txt", b"hello");
        let dest = src.path().join("does-not-exist.txt");

        let (status, source_size, dest_size) = classify_entry(&f, &dest).unwrap();
        assert_eq!(status, ReconcileFileStatus::New);
        assert_eq!(source_size, 5);
        assert_eq!(dest_size, None);
    }

    #[test]
    fn classify_conflict_on_differing_size_without_hashing() {
        let src = TempDir::new("classify-size-conflict");
        let a = write_file(src.path(), "a.txt", b"short");
        let b = write_file(src.path(), "b.txt", b"a much longer piece of content");

        let (status, source_size, dest_size) = classify_entry(&a, &b).unwrap();
        assert_eq!(status, ReconcileFileStatus::Conflict);
        assert_eq!(source_size, 5);
        assert_eq!(dest_size, Some(30));
    }

    #[test]
    fn classify_identical_on_same_size_and_hash() {
        let src = TempDir::new("classify-identical");
        let a = write_file(src.path(), "a.txt", b"same bytes");
        let b = write_file(src.path(), "b.txt", b"same bytes");

        let (status, ..) = classify_entry(&a, &b).unwrap();
        assert_eq!(status, ReconcileFileStatus::Identical);
    }

    #[test]
    fn classify_conflict_on_same_size_different_content_requires_hash() {
        // Same length (5 bytes each) but different bytes - this is exactly
        // the case size-first alone cannot resolve; the hash step must run.
        let src = TempDir::new("classify-same-size-conflict");
        let a = write_file(src.path(), "a.txt", b"aaaaa");
        let b = write_file(src.path(), "b.txt", b"bbbbb");

        let (status, source_size, dest_size) = classify_entry(&a, &b).unwrap();
        assert_eq!(status, ReconcileFileStatus::Conflict);
        assert_eq!(source_size, 5);
        assert_eq!(dest_size, Some(5));
    }

    // -- canonical_dest_relpath: project-number-agnostic file pairing -----

    #[test]
    fn canonical_dest_relpath_renumbers_a_component_containing_the_source_number() {
        let rel = Path::new("26-97110-var Default Values.json");
        let dest = canonical_dest_relpath(rel, "26-97110", "26-97104");
        assert_eq!(
            dest,
            PathBuf::from("26-97104-var Default Values.json")
        );
    }

    #[test]
    fn canonical_dest_relpath_renumbers_within_a_subdirectory_name_too() {
        // rename_template_files_cross_platform renames directory entries as
        // well as files - the reverse mapping must too.
        let rel = Path::new("02 Proposal/26-97110-var.json");
        let dest = canonical_dest_relpath(rel, "26-97110", "26-97104");
        assert_eq!(dest, PathBuf::from("02 Proposal/26-97104-var.json"));
    }

    #[test]
    fn canonical_dest_relpath_leaves_a_plain_user_file_unchanged() {
        // No occurrence of the source number - passes through untouched,
        // pairing by plain relative path exactly as before this feature.
        let rel = Path::new("notes.txt");
        let dest = canonical_dest_relpath(rel, "26-97110", "26-97104");
        assert_eq!(dest, PathBuf::from("notes.txt"));
    }

    #[test]
    fn canonical_dest_relpath_is_a_noop_when_numbers_are_equal() {
        let rel = Path::new("26-97104-var.json");
        let dest = canonical_dest_relpath(rel, "26-97104", "26-97104");
        assert_eq!(dest, rel);
    }

    #[test]
    fn canonical_dest_relpath_is_a_noop_when_either_number_is_empty() {
        let rel = Path::new("anything.txt");
        assert_eq!(
            canonical_dest_relpath(rel, "", "26-97104"),
            rel,
            "empty source_number must not trigger str::replace(\"\", ...) corruption"
        );
        assert_eq!(canonical_dest_relpath(rel, "26-97110", ""), rel);
    }

    // -- keep_both_path: never clobbers, always finds a free name -----

    #[test]
    fn keep_both_path_appends_suffix_when_free() {
        let dir = TempDir::new("keep-both-free");
        let dest = dir.path().join("report.pdf");
        // dest itself doesn't need to exist for this call - only the
        // candidate names are checked.
        let candidate = keep_both_path(&dest);
        assert_eq!(candidate, dir.path().join("report (from source).pdf"));
    }

    #[test]
    fn keep_both_path_increments_when_first_candidate_taken() {
        let dir = TempDir::new("keep-both-taken");
        let dest = dir.path().join("report.pdf");
        write_file(dir.path(), "report (from source).pdf", b"already here");

        let candidate = keep_both_path(&dest);
        assert_eq!(candidate, dir.path().join("report (from source 2).pdf"));
        assert!(
            !candidate.exists(),
            "keep_both_path must never return an existing path"
        );
    }

    #[test]
    fn keep_both_path_handles_extensionless_files() {
        let dir = TempDir::new("keep-both-noext");
        let dest = dir.path().join("README");
        let candidate = keep_both_path(&dest);
        assert_eq!(candidate, dir.path().join("README (from source)"));
    }

    // -- backup_dir_for: always resolves OUTSIDE the 4 status dirs ----

    #[test]
    fn backup_dir_for_lands_outside_the_status_dir_as_a_sibling_of_it() {
        // Mirrors the real layout: {base}/{status dir}/{project folder}.
        let base = TempDir::new("backup-dir-layout");
        let status_dir = base.path().join("11 Current");
        let target_root = status_dir.join("26-97104 Real Project");
        fs::create_dir_all(&target_root).unwrap();

        let backup = backup_dir_for(&target_root, "20260818T120000Z").unwrap();

        assert_eq!(
            backup,
            base.path()
                .join(BACKUP_DIR_NAME)
                .join("20260818T120000Z")
        );
        // Must NOT be nested inside the status dir it was derived from -
        // that's the exact case that would let is_project_folder() see it.
        assert!(!backup.starts_with(&status_dir));
        assert!(backup.starts_with(base.path()));
    }

    #[test]
    fn backup_dir_for_refuses_when_too_shallow_to_have_a_grandparent() {
        // A root-level path (e.g. "/") has no grandparent to resolve a
        // base path from - must refuse rather than guess.
        let shallow = Path::new("/only-one-level");
        assert_eq!(backup_dir_for(shallow, "ts"), None);
    }

    // -- safe_relative_path: traversal guard -----------------------------

    #[test]
    fn safe_relative_path_accepts_a_plain_relative_path() {
        assert!(safe_relative_path("sub/dir/file.txt").is_ok());
    }

    #[test]
    fn safe_relative_path_rejects_absolute_paths() {
        assert!(safe_relative_path("/etc/passwd").is_err());
    }

    #[test]
    fn safe_relative_path_rejects_parent_dir_traversal() {
        assert!(safe_relative_path("../../etc/passwd").is_err());
        assert!(safe_relative_path("sub/../../escape.txt").is_err());
    }

    #[test]
    fn safe_relative_path_rejects_empty_string() {
        assert!(safe_relative_path("").is_err());
        assert!(safe_relative_path("   ").is_err());
    }

    // -- execute_folder_reconcile: dry_run never touches the filesystem --

    #[tokio::test]
    async fn dry_run_copy_does_not_create_the_destination_file() {
        let src = TempDir::new("dryrun-src");
        let dst = TempDir::new("dryrun-dst");
        write_file(src.path(), "new.txt", b"hello");

        let outcome = execute_folder_reconcile(
            src.path().to_string_lossy().to_string(),
            dst.path().to_string_lossy().to_string(),
            vec![ReconcileResolution {
                relative_path: "new.txt".to_string(),
                action: ReconcileAction::Copy,
            }],
            true,
            String::new(),
            String::new(),
        )
        .await
        .unwrap();

        assert_eq!(outcome.copied, 1);
        assert!(outcome.errors.is_empty());
        assert!(!dst.path().join("new.txt").exists());
    }

    #[tokio::test]
    async fn real_run_copy_creates_the_destination_file() {
        let src = TempDir::new("realrun-src");
        let dst = TempDir::new("realrun-dst");
        write_file(src.path(), "new.txt", b"hello");

        let outcome = execute_folder_reconcile(
            src.path().to_string_lossy().to_string(),
            dst.path().to_string_lossy().to_string(),
            vec![ReconcileResolution {
                relative_path: "new.txt".to_string(),
                action: ReconcileAction::Copy,
            }],
            false,
            String::new(),
            String::new(),
        )
        .await
        .unwrap();

        assert_eq!(outcome.copied, 1);
        assert!(outcome.errors.is_empty());
        assert_eq!(fs::read(dst.path().join("new.txt")).unwrap(), b"hello");
    }

    #[tokio::test]
    async fn execute_reconcile_rejects_unsafe_relative_path_before_any_write() {
        let src = TempDir::new("unsafe-src");
        let dst = TempDir::new("unsafe-dst");
        write_file(src.path(), "ok.txt", b"safe file");

        let result = execute_folder_reconcile(
            src.path().to_string_lossy().to_string(),
            dst.path().to_string_lossy().to_string(),
            vec![
                ReconcileResolution {
                    relative_path: "ok.txt".to_string(),
                    action: ReconcileAction::Copy,
                },
                ReconcileResolution {
                    relative_path: "../../escape.txt".to_string(),
                    action: ReconcileAction::Copy,
                },
            ],
            false,
            String::new(),
            String::new(),
        )
        .await;

        assert!(result.is_err(), "must refuse the whole batch");
        // The safe entry must NOT have been applied either - validation
        // happens for every resolution before any write starts.
        assert!(!dst.path().join("ok.txt").exists());
    }

    // -- end-to-end: project-number-agnostic pairing through the real
    // -- preview/execute commands, not just the pure helper -------------

    #[tokio::test]
    async fn renamed_base_file_pairs_as_conflict_not_two_news() {
        // Source's "var.json" was created from the same template as
        // target's, but each carries its OWN project's number in the
        // filename per rename_template_files_cross_platform - and the two
        // have DIFFERENT content (a real, resolvable conflict), not two
        // unrelated files that both happen to be "new".
        let src = TempDir::new("renumber-conflict-src");
        let dst = TempDir::new("renumber-conflict-dst");
        write_file(src.path(), "26-97110-var.json", b"source version");
        write_file(dst.path(), "26-97104-var.json", b"target version - different");

        let preview = preview_folder_reconcile(
            src.path().to_string_lossy().to_string(),
            dst.path().to_string_lossy().to_string(),
            "26-97110".to_string(),
            "26-97104".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(
            preview.new_count, 0,
            "the renamed base file must NOT be reported as two unrelated NEW files"
        );
        assert_eq!(preview.conflict_count, 1);
        assert_eq!(preview.entries.len(), 1);
        assert_eq!(preview.entries[0].status, ReconcileFileStatus::Conflict);
        assert_eq!(preview.entries[0].relative_path, "26-97110-var.json");
        assert_eq!(preview.entries[0].dest_relative_path, "26-97104-var.json");
    }

    #[tokio::test]
    async fn identical_renamed_base_files_pair_as_identical() {
        let src = TempDir::new("renumber-identical-src");
        let dst = TempDir::new("renumber-identical-dst");
        write_file(src.path(), "26-97110-var.json", b"same template content");
        write_file(dst.path(), "26-97104-var.json", b"same template content");

        let preview = preview_folder_reconcile(
            src.path().to_string_lossy().to_string(),
            dst.path().to_string_lossy().to_string(),
            "26-97110".to_string(),
            "26-97104".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(preview.identical_count, 1);
        assert_eq!(preview.new_count, 0);
        assert_eq!(preview.conflict_count, 0);
        assert_eq!(preview.entries[0].status, ReconcileFileStatus::Identical);
    }

    #[tokio::test]
    async fn non_prefixed_user_file_still_pairs_by_plain_relative_path() {
        // A user-added file with no project number in its name at all -
        // must keep exact-path pairing, unaffected by the renumbering.
        let src = TempDir::new("renumber-userfile-src");
        let dst = TempDir::new("renumber-userfile-dst");
        write_file(src.path(), "site photo.jpg", b"aaaa");
        write_file(dst.path(), "site photo.jpg", b"bbbb");

        let preview = preview_folder_reconcile(
            src.path().to_string_lossy().to_string(),
            dst.path().to_string_lossy().to_string(),
            "26-97110".to_string(),
            "26-97104".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(preview.entries.len(), 1);
        assert_eq!(preview.entries[0].relative_path, "site photo.jpg");
        assert_eq!(preview.entries[0].dest_relative_path, "site photo.jpg");
        assert_eq!(preview.entries[0].status, ReconcileFileStatus::Conflict);
    }

    #[tokio::test]
    async fn re_prefix_on_copy_produces_the_targets_number() {
        // A source-numbered file with NO destination counterpart (a real
        // NEW) must still be copied in under the TARGET's number, not left
        // with the source's number, so the merged folder stays internally
        // consistent with the naming convention.
        let src = TempDir::new("reprefix-copy-src");
        let dst = TempDir::new("reprefix-copy-dst");
        write_file(src.path(), "26-97110-var.json", b"only exists in source");

        let outcome = execute_folder_reconcile(
            src.path().to_string_lossy().to_string(),
            dst.path().to_string_lossy().to_string(),
            vec![ReconcileResolution {
                relative_path: "26-97110-var.json".to_string(),
                action: ReconcileAction::Copy,
            }],
            false,
            "26-97110".to_string(),
            "26-97104".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(outcome.copied, 1);
        assert!(outcome.errors.is_empty());
        assert!(
            !dst.path().join("26-97110-var.json").exists(),
            "must not land under the SOURCE's number"
        );
        assert_eq!(
            fs::read(dst.path().join("26-97104-var.json")).unwrap(),
            b"only exists in source",
            "must land under the TARGET's number"
        );
    }

    // -- backup_is_eligible: age boundary is strictly-greater-than -------

    #[test]
    fn backup_is_eligible_keeps_a_backup_exactly_at_the_cutoff() {
        let now = Utc::now();
        let created_at = now - chrono::Duration::days(30);
        assert!(
            !backup_is_eligible(created_at, now, 30),
            "exactly 30 days old must be KEPT, not eligible - only strictly older is"
        );
    }

    #[test]
    fn backup_is_eligible_deletes_a_backup_one_second_past_the_cutoff() {
        let now = Utc::now();
        let created_at = now - chrono::Duration::days(30) - chrono::Duration::seconds(1);
        assert!(backup_is_eligible(created_at, now, 30));
    }

    #[test]
    fn backup_is_eligible_keeps_a_backup_just_under_the_cutoff() {
        let now = Utc::now();
        let created_at = now - chrono::Duration::days(29) - chrono::Duration::hours(23);
        assert!(!backup_is_eligible(created_at, now, 30));
    }

    #[test]
    fn backup_is_eligible_deletes_a_much_older_backup() {
        let now = Utc::now();
        let created_at = now - chrono::Duration::days(90);
        assert!(backup_is_eligible(created_at, now, 30));
    }

    #[test]
    fn backup_is_eligible_respects_a_custom_cutoff() {
        let now = Utc::now();
        let created_at = now - chrono::Duration::days(8);
        assert!(backup_is_eligible(created_at, now, 7));
        assert!(!backup_is_eligible(created_at, now, 9));
    }

    // -- parse_backup_timestamp: matches reconcile_timestamp()'s format --

    #[test]
    fn parse_backup_timestamp_round_trips_reconcile_timestamp_format() {
        let ts = reconcile_timestamp();
        assert!(
            parse_backup_timestamp(&ts).is_some(),
            "must parse the exact format reconcile_timestamp() produces"
        );
    }

    #[test]
    fn parse_backup_timestamp_rejects_non_conforming_names() {
        assert!(parse_backup_timestamp("not-a-timestamp").is_none());
        assert!(parse_backup_timestamp("").is_none());
        assert!(parse_backup_timestamp("2026-08-18").is_none());
    }

    // -- list_backup_entries: listing, filtering, sizing -------------------

    fn write_backup_dir(base: &Path, timestamp: &str, files: &[(&str, &[u8])]) -> PathBuf {
        let dir = base.join(BACKUP_DIR_NAME).join(timestamp);
        for (rel, content) in files {
            write_file(&dir, rel, content);
        }
        dir
    }

    #[test]
    fn list_backup_entries_returns_empty_when_no_backups_dir_exists() {
        let base = TempDir::new("cleanup-no-backups-dir");
        let entries = list_backup_entries(base.path(), 30, Utc::now()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn list_backup_entries_only_returns_backups_older_than_cutoff() {
        let base = TempDir::new("cleanup-mixed-ages");
        let now = Utc::now();

        let old_ts = (now - chrono::Duration::days(45))
            .format("%Y%m%dT%H%M%SZ")
            .to_string();
        let recent_ts = (now - chrono::Duration::days(5))
            .format("%Y%m%dT%H%M%SZ")
            .to_string();
        write_backup_dir(base.path(), &old_ts, &[("file.txt", b"old backup content")]);
        write_backup_dir(base.path(), &recent_ts, &[("file.txt", b"recent")]);

        let entries = list_backup_entries(base.path(), 30, now).unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].timestamp, old_ts);
        assert_eq!(entries[0].file_count, 1);
        assert_eq!(
            entries[0].total_size_bytes,
            "old backup content".len() as u64
        );
    }

    #[test]
    fn list_backup_entries_ignores_non_conforming_directory_names() {
        let base = TempDir::new("cleanup-stray-dir");
        let now = Utc::now();
        // A directory that doesn't match the timestamp format - must be
        // skipped defensively rather than mis-parsed or (worse) deleted.
        write_file(
            &base.path().join(BACKUP_DIR_NAME).join("not-a-timestamp"),
            "stray.txt",
            b"x",
        );

        let entries = list_backup_entries(base.path(), 30, now).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn list_backup_entries_sums_multiple_files_per_backup() {
        let base = TempDir::new("cleanup-multi-file");
        let now = Utc::now();
        let old_ts = (now - chrono::Duration::days(60))
            .format("%Y%m%dT%H%M%SZ")
            .to_string();
        write_backup_dir(
            base.path(),
            &old_ts,
            &[("a.txt", b"12345"), ("sub/b.txt", b"1234567890")],
        );

        let entries = list_backup_entries(base.path(), 30, now).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file_count, 2);
        assert_eq!(entries[0].total_size_bytes, 15);
    }

    #[test]
    fn list_backup_entries_sorts_by_timestamp_ascending() {
        let base = TempDir::new("cleanup-sort-order");
        let now = Utc::now();
        let older = (now - chrono::Duration::days(90))
            .format("%Y%m%dT%H%M%SZ")
            .to_string();
        let newer = (now - chrono::Duration::days(31))
            .format("%Y%m%dT%H%M%SZ")
            .to_string();
        // Written newer-first, must still come back older-first.
        write_backup_dir(base.path(), &newer, &[("f.txt", b"n")]);
        write_backup_dir(base.path(), &older, &[("f.txt", b"o")]);

        let entries = list_backup_entries(base.path(), 30, now).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].timestamp, older);
        assert_eq!(entries[1].timestamp, newer);
    }

    // -- delete_backup_dirs: dry_run vs real deletion, error accumulation --

    #[test]
    fn delete_backup_dirs_dry_run_does_not_touch_the_filesystem() {
        let base = TempDir::new("cleanup-dryrun");
        let dir = write_backup_dir(base.path(), "20260101T000000Z", &[("a.txt", b"12345")]);
        let entries = vec![BackupCleanupEntry {
            timestamp: "20260101T000000Z".to_string(),
            path: dir.to_string_lossy().to_string(),
            age_days: 999,
            file_count: 1,
            total_size_bytes: 5,
        }];

        let (deleted, freed, errors) = delete_backup_dirs(&entries, true);

        assert_eq!(deleted, 1);
        assert_eq!(freed, 5);
        assert!(errors.is_empty());
        assert!(dir.exists(), "dry_run must never delete anything");
    }

    #[test]
    fn delete_backup_dirs_real_run_removes_the_directory() {
        let base = TempDir::new("cleanup-realrun");
        let dir = write_backup_dir(base.path(), "20260101T000000Z", &[("a.txt", b"12345")]);
        let entries = vec![BackupCleanupEntry {
            timestamp: "20260101T000000Z".to_string(),
            path: dir.to_string_lossy().to_string(),
            age_days: 999,
            file_count: 1,
            total_size_bytes: 5,
        }];

        let (deleted, freed, errors) = delete_backup_dirs(&entries, false);

        assert_eq!(deleted, 1);
        assert_eq!(freed, 5);
        assert!(errors.is_empty());
        assert!(!dir.exists(), "real run must actually remove the directory");
    }

    #[test]
    fn delete_backup_dirs_reports_a_per_entry_error_and_continues() {
        // One entry points at a path that doesn't exist - remove_dir_all
        // must fail for that entry without aborting the batch (matches
        // execute_folder_reconcile's per-file error accumulation contract).
        let base = TempDir::new("cleanup-error-continue");
        let missing = base.path().join(BACKUP_DIR_NAME).join("20260101T000000Z");
        let real_dir = write_backup_dir(base.path(), "20260102T000000Z", &[("a.txt", b"ok")]);
        let entries = vec![
            BackupCleanupEntry {
                timestamp: "20260101T000000Z".to_string(),
                path: missing.to_string_lossy().to_string(),
                age_days: 999,
                file_count: 0,
                total_size_bytes: 0,
            },
            BackupCleanupEntry {
                timestamp: "20260102T000000Z".to_string(),
                path: real_dir.to_string_lossy().to_string(),
                age_days: 999,
                file_count: 1,
                total_size_bytes: 2,
            },
        ];

        let (deleted, freed, errors) = delete_backup_dirs(&entries, false);

        assert_eq!(deleted, 1, "the real entry must still succeed");
        assert_eq!(freed, 2);
        assert_eq!(
            errors.len(),
            1,
            "the missing entry must report an error, not panic"
        );
        assert!(!real_dir.exists());
    }

    // -- end-to-end: the full reconcile flow against a REALISTIC on-disk
    // -- layout (two different status dirs sharing one base path, exactly
    // -- as FolderReconcileModal drives it after a real project merge) -
    // -- pre-release confidence check, not just isolated-helper coverage.
    // -- Every prior test in this file uses flat sibling TempDirs for
    // -- source/target; NONE puts them under DIFFERENT status dirs the way
    // -- production folders actually sit (`{base}/{status dir}/{project}`),
    // -- and none exercises ReconcileAction::Overwrite through
    // -- execute_folder_reconcile at all - this closes both gaps at once.

    #[tokio::test]
    async fn end_to_end_reconcile_flow_against_realistic_status_dir_layout() {
        // Mirrors backup_dir_for_lands_outside_the_status_dir_as_a_sibling_of_it's
        // layout, but with source and target under TWO DIFFERENT status
        // dirs sharing one base - the case a real merge/reconcile always
        // hits (an RFP being merged into a Current project), which no
        // other test in this file covers.
        let base = TempDir::new("e2e-reconcile-realistic-layout");
        let source_root = base
            .path()
            .join("01 RFPs")
            .join("26-97110 Old Duplicate");
        let target_root = base
            .path()
            .join("11 Current")
            .join("26-97104 Real Project");
        fs::create_dir_all(&source_root).unwrap();
        fs::create_dir_all(&target_root).unwrap();

        // NEW: only in source.
        write_file(&source_root, "new-invoice.pdf", b"invoice content");
        // CONFLICT: same logical base file, renamed per project number,
        // genuinely different content (a real merge decision, not a
        // same-size collision).
        write_file(&source_root, "26-97110-var.json", b"source variables v2");
        write_file(
            &target_root,
            "26-97104-var.json",
            b"target variables v1 (older)",
        );
        // IDENTICAL: present in both, byte-for-byte equal.
        write_file(&source_root, "site photo.jpg", b"same photo bytes");
        write_file(&target_root, "site photo.jpg", b"same photo bytes");

        let source_path = source_root.to_string_lossy().to_string();
        let target_path = target_root.to_string_lossy().to_string();

        // --- Step 1: preview classifies all three correctly -------------
        let preview = preview_folder_reconcile(
            source_path.clone(),
            target_path.clone(),
            "26-97110".to_string(),
            "26-97104".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(preview.entries.len(), 3);
        assert_eq!(preview.new_count, 1);
        assert_eq!(preview.conflict_count, 1);
        assert_eq!(preview.identical_count, 1);

        let conflict_entry = preview
            .entries
            .iter()
            .find(|e| e.status == ReconcileFileStatus::Conflict)
            .expect("conflict entry must be present");
        assert_eq!(conflict_entry.relative_path, "26-97110-var.json");
        assert_eq!(
            conflict_entry.dest_relative_path, "26-97104-var.json",
            "conflict must be paired via canonical_dest_relpath, not raw path"
        );

        // --- Step 2: resolve as a realistic UI session would - take the
        // --- new file, overwrite the conflict, skip the identical one ---
        let resolutions = vec![
            ReconcileResolution {
                relative_path: "new-invoice.pdf".to_string(),
                action: ReconcileAction::Copy,
            },
            ReconcileResolution {
                relative_path: "26-97110-var.json".to_string(),
                action: ReconcileAction::Overwrite,
            },
            ReconcileResolution {
                relative_path: "site photo.jpg".to_string(),
                action: ReconcileAction::Skip,
            },
        ];

        // --- Step 3: dry run must report the same outcome shape but
        // --- touch NOTHING on disk ---------------------------------------
        let dry_run_outcome = execute_folder_reconcile(
            source_path.clone(),
            target_path.clone(),
            resolutions.clone(),
            true,
            "26-97110".to_string(),
            "26-97104".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(dry_run_outcome.copied, 1);
        assert_eq!(dry_run_outcome.overwritten, 1);
        assert_eq!(dry_run_outcome.skipped, 1);
        assert_eq!(dry_run_outcome.kept_both, 0);
        assert!(dry_run_outcome.errors.is_empty());
        assert_eq!(
            dry_run_outcome.backups_created.len(),
            1,
            "dry run must still report what WOULD be backed up"
        );

        assert!(
            !target_root.join("new-invoice.pdf").exists(),
            "dry run must not copy the new file"
        );
        assert_eq!(
            fs::read(target_root.join("26-97104-var.json")).unwrap(),
            b"target variables v1 (older)",
            "dry run must not touch the conflicting file's content"
        );
        assert!(
            !base.path().join(BACKUP_DIR_NAME).exists(),
            "dry run must not create the backup directory at all"
        );

        // --- Step 4: the real run actually applies everything -----------
        let outcome = execute_folder_reconcile(
            source_path,
            target_path,
            resolutions,
            false,
            "26-97110".to_string(),
            "26-97104".to_string(),
        )
        .await
        .unwrap();

        assert_eq!(outcome.copied, 1);
        assert_eq!(outcome.overwritten, 1);
        assert_eq!(outcome.skipped, 1);
        assert!(outcome.errors.is_empty());
        assert_eq!(outcome.backups_created.len(), 1);

        // NEW file landed in target with source's content.
        assert_eq!(
            fs::read(target_root.join("new-invoice.pdf")).unwrap(),
            b"invoice content"
        );
        // CONFLICT: target now holds source's content, under the TARGET's
        // number (not left as the source-numbered file).
        assert_eq!(
            fs::read(target_root.join("26-97104-var.json")).unwrap(),
            b"source variables v2"
        );
        assert!(!target_root.join("26-97110-var.json").exists());
        // SKIP: identical file genuinely untouched.
        assert_eq!(
            fs::read(target_root.join("site photo.jpg")).unwrap(),
            b"same photo bytes"
        );
        // Source folder itself is never written to.
        assert_eq!(
            fs::read(source_root.join("26-97110-var.json")).unwrap(),
            b"source variables v2"
        );

        // The pre-overwrite target content is preserved in the backup -
        // this is the actual safety invariant the "never a silent
        // overwrite" doc comment promises, verified by CONTENT not just by
        // a path string existing.
        let backup_path = PathBuf::from(&outcome.backups_created[0]);
        assert_eq!(
            fs::read(&backup_path).unwrap(),
            b"target variables v1 (older)",
            "backup must hold the ORIGINAL destination content, not the new one"
        );

        // The backup must land outside BOTH status dirs - the exact
        // production scenario backup_dir_for's doc comment describes,
        // now proven against two DIFFERENT status dirs rather than one.
        assert!(!backup_path.starts_with(base.path().join("01 RFPs")));
        assert!(!backup_path.starts_with(base.path().join("11 Current")));
        assert!(backup_path.starts_with(base.path().join(BACKUP_DIR_NAME)));
    }

    // -- end-to-end: list + delete pipeline together ----------------------

    #[test]
    fn end_to_end_cleanup_deletes_old_backups_and_keeps_recent_ones() {
        let base = TempDir::new("cleanup-e2e");
        let now = Utc::now();
        let old_ts = (now - chrono::Duration::days(40))
            .format("%Y%m%dT%H%M%SZ")
            .to_string();
        let recent_ts = (now - chrono::Duration::days(2))
            .format("%Y%m%dT%H%M%SZ")
            .to_string();
        let old_dir = write_backup_dir(base.path(), &old_ts, &[("f.txt", b"old")]);
        let recent_dir = write_backup_dir(base.path(), &recent_ts, &[("f.txt", b"new")]);

        let eligible = list_backup_entries(base.path(), 30, now).unwrap();
        let (deleted, freed, errors) = delete_backup_dirs(&eligible, false);

        assert_eq!(deleted, 1);
        assert_eq!(freed, 3);
        assert!(errors.is_empty());
        assert!(!old_dir.exists(), "old backup must be removed");
        assert!(recent_dir.exists(), "recent backup must survive");
    }
}
