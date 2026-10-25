//! Award-time project folder setup (pure filesystem logic, no Tauri types).
//!
//! When a project is awarded the owner does three things by hand:
//!   1. moves `01 RFPs/<number> <name>` to `11 Current/<number> <name>`
//!      (handled by `move_project_folder`),
//!   2. copies the contents of `11 Current/00 Design Folders` into it without
//!      overwriting anything already there,
//!   3. renames every file starting with the placeholder `yy-cccnn` to start
//!      with the project number.
//!
//! This module implements 2 and 3, plus the pure lookup/destination checks the
//! move step relies on, so they can be unit tested against a temp directory.

use std::fs;
use std::path::{Path, PathBuf};

/// Placeholder prefix used by the design-folder and RFP templates.
pub(crate) const PLACEHOLDER_PREFIX: &str = "yy-cccnn";

/// Status directories searched for a project folder, in search order.
pub(crate) const STATUS_DIRS: [&str; 4] = ["00 Inactive", "01 RFPs", "11 Current", "99 Completed"];

/// Template folder names under `11 Current`, preferred name first. The legacy
/// name is kept as a fallback for installs that still use it.
const TEMPLATE_DIR_NAMES: [&str; 2] = ["00 Design Folders", "00 Additional Folders"];

/// What the award setup did, for the user-facing message.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct AwardSetupReport {
    pub files_copied: usize,
    pub files_skipped_existing: usize,
    pub files_renamed: usize,
    /// Renames not done because the target name already exists.
    pub rename_conflicts: Vec<String>,
}

impl AwardSetupReport {
    pub fn summary(&self) -> String {
        let mut s = format!(
            "Design folders merged ({} copied, {} already present), {} file(s) renamed to the project number",
            self.files_copied, self.files_skipped_existing, self.files_renamed
        );
        if !self.rename_conflicts.is_empty() {
            s.push_str(&format!(
                "; {} rename(s) skipped because the target exists: {}",
                self.rename_conflicts.len(),
                self.rename_conflicts.join(", ")
            ));
        }
        s
    }
}

/// Reject empty/whitespace/path-like project numbers before they are used as
/// a path prefix or component.
pub(crate) fn validate_project_number(project_number: &str) -> Result<(), String> {
    let n = project_number.trim();
    if n.is_empty() || n != project_number {
        return Err("Project number is empty or has surrounding whitespace".to_string());
    }
    if !n.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(format!("Invalid project number: {}", project_number));
    }
    Ok(())
}

/// A folder belongs to a project when its name is exactly the number or the
/// number followed by a space ("26-97113 Lulu Sphere"), so "26-9711" cannot
/// match "26-97113 ...".
pub(crate) fn folder_matches_number(folder_name: &str, project_number: &str) -> bool {
    !project_number.is_empty()
        && (folder_name == project_number
            || folder_name.starts_with(&format!("{} ", project_number)))
}

/// Locate a project folder (name starts with `project_number`) in the status
/// directories under `base`. Returns `(status_dir, full_path)`.
pub(crate) fn find_in_base(base: &Path, project_number: &str) -> Option<(String, PathBuf)> {
    for status_dir in STATUS_DIRS {
        let Ok(entries) = fs::read_dir(base.join(status_dir)) else {
            continue;
        };
        for entry in entries.flatten() {
            if folder_matches_number(&entry.file_name().to_string_lossy(), project_number) {
                return Some((status_dir.to_string(), entry.path()));
            }
        }
    }
    None
}

/// Destination path for moving `folder_name` into `dest_dir`. Errors clearly
/// when the destination directory is missing or the folder already exists.
pub(crate) fn check_move_destination(
    dest_dir: &Path,
    folder_name: &std::ffi::OsStr,
) -> Result<PathBuf, String> {
    if !dest_dir.is_dir() {
        return Err(format!(
            "Destination directory {} does not exist",
            dest_dir.display()
        ));
    }
    let new_path = dest_dir.join(folder_name);
    if new_path.exists() {
        return Err(format!(
            "Destination folder already exists: {}",
            new_path.display()
        ));
    }
    Ok(new_path)
}

/// Find the design-folder template under `current_dir` (`11 Current`).
pub(crate) fn resolve_template_dir(current_dir: &Path) -> Result<PathBuf, String> {
    for name in TEMPLATE_DIR_NAMES {
        let p = current_dir.join(name);
        if p.is_dir() {
            return Ok(p);
        }
    }
    Err(format!(
        "Design folder template not found: expected '{}' in {}",
        TEMPLATE_DIR_NAMES[0],
        current_dir.display()
    ))
}

/// The path `p` would have after the yy-cccnn rename, if it carries the prefix.
fn renamed_twin(p: &Path, project_number: &str) -> Option<PathBuf> {
    let name = p.file_name()?.to_string_lossy().to_string();
    let rest = name.strip_prefix(PLACEHOLDER_PREFIX)?;
    Some(p.with_file_name(format!("{}{}", project_number, rest)))
}

/// Recursively copy `src` into `dst`, never overwriting an existing file.
/// Returns `(copied, skipped_existing)` file counts.
pub(crate) fn merge_copy_no_overwrite(
    src: &Path,
    dst: &Path,
    project_number: &str,
) -> Result<(usize, usize), String> {
    fs::create_dir_all(dst)
        .map_err(|e| format!("Failed to create {}: {}", dst.display(), e))?;
    let mut copied = 0;
    let mut skipped = 0;
    let entries =
        fs::read_dir(src).map_err(|e| format!("Failed to read {}: {}", src.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read {}: {}", src.display(), e))?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Failed to stat {}: {}", from.display(), e))?;
        if file_type.is_dir() {
            let (c, s) = merge_copy_no_overwrite(&from, &to, project_number)?;
            copied += c;
            skipped += s;
        } else if to.exists() || renamed_twin(&to, project_number).is_some_and(|t| t.exists()) {
            // Already present, either under the template name or - on a
            // retry - under its yy-cccnn-renamed name.
            skipped += 1;
        } else {
            fs::copy(&from, &to).map_err(|e| {
                format!("Failed to copy {} to {}: {}", from.display(), to.display(), e)
            })?;
            copied += 1;
        }
    }
    Ok((copied, skipped))
}

/// Rename every file under `root` whose name starts with `yy-cccnn` so it
/// starts with `project_number`. Existing targets are never overwritten; they
/// are reported in the returned conflict list. Returns `(renamed, conflicts)`.
pub(crate) fn rename_placeholder_files(
    root: &Path,
    project_number: &str,
) -> Result<(usize, Vec<String>), String> {
    let mut renamed = 0;
    let mut conflicts = Vec::new();
    let entries =
        fs::read_dir(root).map_err(|e| format!("Failed to read {}: {}", root.display(), e))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read {}: {}", root.display(), e))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| format!("Failed to stat {}: {}", path.display(), e))?;
        if file_type.is_dir() {
            let (r, c) = rename_placeholder_files(&path, project_number)?;
            renamed += r;
            conflicts.extend(c);
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(rest) = name.strip_prefix(PLACEHOLDER_PREFIX) {
            let target = root.join(format!("{}{}", project_number, rest));
            if target.exists() {
                conflicts.push(name);
                continue;
            }
            fs::rename(&path, &target).map_err(|e| {
                format!("Failed to rename {} to {}: {}", path.display(), target.display(), e)
            })?;
            renamed += 1;
        }
    }
    Ok((renamed, conflicts))
}

/// Full award setup for a project folder already sitting in `11 Current`:
/// merge the design-folder template (no overwrite), then rename placeholders.
pub(crate) fn setup_awarded_folder(
    current_dir: &Path,
    project_dir: &Path,
    project_number: &str,
) -> Result<AwardSetupReport, String> {
    let template = resolve_template_dir(current_dir)?;
    let (files_copied, files_skipped_existing) = merge_copy_no_overwrite(&template, project_dir, project_number)?;
    let (files_renamed, rename_conflicts) = rename_placeholder_files(project_dir, project_number)?;
    Ok(AwardSetupReport {
        files_copied,
        files_skipped_existing,
        files_renamed,
        rename_conflicts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    /// Self-cleaning temp dir (no tempfile dev-dependency in this crate).
    struct Tmp(PathBuf);
    impl Tmp {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "efees-award-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::SeqCst)
            ));
            fs::create_dir_all(&p).unwrap();
            Tmp(p)
        }
    }
    impl Drop for Tmp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write(p: &Path, body: &str) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    }

    /// Base with the four status dirs and the design-folder template.
    fn base_with_template(tmp: &Tmp) -> PathBuf {
        let base = tmp.0.join("01 Projects");
        for d in STATUS_DIRS {
            fs::create_dir_all(base.join(d)).unwrap();
        }
        let t = base.join("11 Current/00 Design Folders");
        write(&t.join("04 Deliverables/01 Drawings/03 In Progress/yy-cccnn-LTG-DWG-101.dwg"), "dwg");
        write(&t.join("04 Deliverables/yy-cccnn-LI-REG-001 r01 Register yymmdd.xlsx"), "reg");
        write(&t.join("03 Contract/03 Invoices/00 Inv Running List.xlsx"), "inv");
        fs::create_dir_all(t.join("98 Outgoing")).unwrap();
        base
    }

    #[test]
    fn find_in_base_locates_rfp_folder() {
        let tmp = Tmp::new();
        let base = base_with_template(&tmp);
        fs::create_dir_all(base.join("01 RFPs/26-97113 Lulu Sphere")).unwrap();
        let (dir, path) = find_in_base(&base, "26-97113").unwrap();
        assert_eq!(dir, "01 RFPs");
        assert_eq!(path, base.join("01 RFPs/26-97113 Lulu Sphere"));
    }

    #[test]
    fn number_prefix_does_not_match_longer_number() {
        let tmp = Tmp::new();
        let base = base_with_template(&tmp);
        fs::create_dir_all(base.join("01 RFPs/26-97113 Lulu Sphere")).unwrap();
        assert!(find_in_base(&base, "26-9711").is_none());
        assert!(find_in_base(&base, "26-97113").is_some());
        fs::create_dir_all(base.join("01 RFPs/26-97200")).unwrap();
        assert!(find_in_base(&base, "26-97200").is_some(), "exact name matches");
    }

    #[test]
    fn empty_or_bad_project_number_rejected() {
        assert!(validate_project_number("").is_err());
        assert!(validate_project_number("  ").is_err());
        assert!(validate_project_number(" 26-97113").is_err());
        assert!(validate_project_number("../x").is_err());
        assert!(validate_project_number("26-97113").is_ok());
        let tmp = Tmp::new();
        let base = base_with_template(&tmp);
        fs::create_dir_all(base.join("01 RFPs/26-97113 Lulu Sphere")).unwrap();
        assert!(find_in_base(&base, "").is_none());
    }

    #[test]
    fn failed_setup_is_recoverable_on_retry() {
        // move ok -> setup fails (no template) -> template appears -> retry completes
        let tmp = Tmp::new();
        let base = tmp.0.join("01 Projects");
        for d in STATUS_DIRS {
            fs::create_dir_all(base.join(d)).unwrap();
        }
        let src = base.join("01 RFPs/26-97113 Lulu Sphere");
        write(&src.join("01 Client Info/brief.txt"), "brief");
        let cur = base.join("11 Current");
        let dest = check_move_destination(&cur, src.file_name().unwrap()).unwrap();
        crate::commands::folder_management::move_folder_cross_platform(&src, &dest).unwrap();
        assert!(setup_awarded_folder(&cur, &dest, "26-97113").is_err());

        write(&cur.join("00 Design Folders/04 Deliverables/yy-cccnn-REG.xlsx"), "reg");
        let report = setup_awarded_folder(&cur, &dest, "26-97113").unwrap();
        assert_eq!(report.files_copied, 1);
        assert!(dest.join("04 Deliverables/26-97113-REG.xlsx").is_file());
        // a further retry is a harmless no-op
        let again = setup_awarded_folder(&cur, &dest, "26-97113").unwrap();
        assert_eq!(again.files_copied, 0, "renamed placeholder is not re-copied");
        assert!(!dest.join("04 Deliverables/yy-cccnn-REG.xlsx").exists());
    }

    #[test]
    fn find_in_base_missing_source_returns_none() {
        let tmp = Tmp::new();
        let base = base_with_template(&tmp);
        assert!(find_in_base(&base, "26-97113").is_none());
    }

    #[test]
    fn destination_already_exists_is_an_error() {
        let tmp = Tmp::new();
        let base = base_with_template(&tmp);
        fs::create_dir_all(base.join("11 Current/26-97113 Lulu Sphere")).unwrap();
        let err = check_move_destination(
            &base.join("11 Current"),
            std::ffi::OsStr::new("26-97113 Lulu Sphere"),
        )
        .unwrap_err();
        assert!(err.contains("already exists"), "{err}");
    }

    #[test]
    fn destination_dir_missing_is_an_error() {
        let tmp = Tmp::new();
        let err = check_move_destination(&tmp.0.join("nope"), std::ffi::OsStr::new("x")).unwrap_err();
        assert!(err.contains("does not exist"), "{err}");
    }

    #[test]
    fn destination_free_returns_joined_path() {
        let tmp = Tmp::new();
        let base = base_with_template(&tmp);
        let p = check_move_destination(&base.join("11 Current"), std::ffi::OsStr::new("26-1 X")).unwrap();
        assert_eq!(p, base.join("11 Current/26-1 X"));
    }

    #[test]
    fn move_keeps_existing_contents() {
        // The move itself is fs::rename via move_folder_cross_platform; verify
        // the contract the award flow depends on (contents survive the move).
        let tmp = Tmp::new();
        let base = base_with_template(&tmp);
        let src = base.join("01 RFPs/26-97113 Lulu Sphere");
        write(&src.join("01 Client Info/brief.txt"), "brief");
        write(&src.join("02 Proposal/26-97113-FP-01.pdf"), "fp");
        let dest = check_move_destination(&base.join("11 Current"), src.file_name().unwrap()).unwrap();
        crate::commands::folder_management::move_folder_cross_platform(&src, &dest).unwrap();
        assert!(!src.exists());
        assert_eq!(fs::read_to_string(dest.join("01 Client Info/brief.txt")).unwrap(), "brief");
        assert_eq!(fs::read_to_string(dest.join("02 Proposal/26-97113-FP-01.pdf")).unwrap(), "fp");
    }

    #[test]
    fn setup_copies_template_and_renames_placeholders() {
        let tmp = Tmp::new();
        let base = base_with_template(&tmp);
        let proj = base.join("11 Current/26-97113 Lulu Sphere");
        write(&proj.join("01 Client Info/brief.txt"), "brief");

        let report = setup_awarded_folder(&base.join("11 Current"), &proj, "26-97113").unwrap();

        assert!(proj.join("04 Deliverables/01 Drawings/03 In Progress/26-97113-LTG-DWG-101.dwg").is_file());
        assert!(proj.join("04 Deliverables/26-97113-LI-REG-001 r01 Register yymmdd.xlsx").is_file());
        assert!(proj.join("03 Contract/03 Invoices/00 Inv Running List.xlsx").is_file());
        assert!(proj.join("98 Outgoing").is_dir());
        assert!(!proj.join("04 Deliverables/01 Drawings/03 In Progress/yy-cccnn-LTG-DWG-101.dwg").exists());
        assert!(proj.join("01 Client Info/brief.txt").is_file(), "existing content kept");
        assert_eq!(report.files_copied, 3);
        assert_eq!(report.files_renamed, 2);
        assert!(report.rename_conflicts.is_empty());
    }

    #[test]
    fn merge_never_overwrites_existing_files() {
        let tmp = Tmp::new();
        let base = base_with_template(&tmp);
        let proj = base.join("11 Current/26-97113 Lulu Sphere");
        write(&proj.join("03 Contract/03 Invoices/00 Inv Running List.xlsx"), "MINE");

        let report = setup_awarded_folder(&base.join("11 Current"), &proj, "26-97113").unwrap();

        assert_eq!(
            fs::read_to_string(proj.join("03 Contract/03 Invoices/00 Inv Running List.xlsx")).unwrap(),
            "MINE"
        );
        assert_eq!(report.files_skipped_existing, 1);
    }

    #[test]
    fn rename_covers_placeholder_files_already_in_project_folder() {
        let tmp = Tmp::new();
        let proj = tmp.0.join("p");
        write(&proj.join("02 Proposal/yy-cccnn-FP-01.docx"), "x");
        write(&proj.join("02 Proposal/other.docx"), "y");
        let (renamed, conflicts) = rename_placeholder_files(&proj, "26-97113").unwrap();
        assert_eq!(renamed, 1);
        assert!(conflicts.is_empty());
        assert!(proj.join("02 Proposal/26-97113-FP-01.docx").is_file());
        assert!(proj.join("02 Proposal/other.docx").is_file());
    }

    #[test]
    fn rename_does_not_clobber_existing_target() {
        let tmp = Tmp::new();
        let proj = tmp.0.join("p");
        write(&proj.join("yy-cccnn-A.dwg"), "template");
        write(&proj.join("26-97113-A.dwg"), "real");
        let (renamed, conflicts) = rename_placeholder_files(&proj, "26-97113").unwrap();
        assert_eq!(renamed, 0);
        assert_eq!(conflicts, vec!["yy-cccnn-A.dwg".to_string()]);
        assert_eq!(fs::read_to_string(proj.join("26-97113-A.dwg")).unwrap(), "real");
        assert!(proj.join("yy-cccnn-A.dwg").exists());
    }

    #[test]
    fn missing_template_is_a_clear_error() {
        let tmp = Tmp::new();
        let base = tmp.0.join("01 Projects");
        fs::create_dir_all(base.join("11 Current/26-97113 Lulu Sphere")).unwrap();
        let err = setup_awarded_folder(
            &base.join("11 Current"),
            &base.join("11 Current/26-97113 Lulu Sphere"),
            "26-97113",
        )
        .unwrap_err();
        assert!(err.contains("00 Design Folders"), "{err}");
    }

    #[test]
    fn legacy_additional_folders_name_still_resolves() {
        let tmp = Tmp::new();
        let cur = tmp.0.join("11 Current");
        fs::create_dir_all(cur.join("00 Additional Folders")).unwrap();
        assert_eq!(resolve_template_dir(&cur).unwrap(), cur.join("00 Additional Folders"));
    }

    #[test]
    fn design_folders_preferred_over_legacy() {
        let tmp = Tmp::new();
        let cur = tmp.0.join("11 Current");
        fs::create_dir_all(cur.join("00 Additional Folders")).unwrap();
        fs::create_dir_all(cur.join("00 Design Folders")).unwrap();
        assert_eq!(resolve_template_dir(&cur).unwrap(), cur.join("00 Design Folders"));
    }
}
