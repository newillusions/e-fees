# E-Fees API Enhancements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 4 API endpoints (fee JSON export, project search, document upload, JSON status) and extract shared export logic to e-fees-core.

**Architecture:** Move pure-logic functions to shared crate (`e-fees-core`), extract reusable SSH helper in the API, add new route modules following existing patterns. TDD with unit tests in core and integration tests against the live API.

**Tech Stack:** Rust, axum 0.8, SurrealDB v3.0.4, tokio, serde_json, chrono, axum-extra (multipart)

**Spec:** `docs/superpowers/specs/2026-03-21-api-enhancements-design.md`

---

### Task 1: Extract shared export logic to e-fees-core

**Files:**
- Create: `crates/e-fees-core/src/export.rs`
- Modify: `crates/e-fees-core/src/lib.rs`
- Modify: `crates/e-fees-core/Cargo.toml` (add `chrono = "0.4"` — NOT already present)

- [ ] **Step 1: Write failing tests for format_issue_date and clean_record_key**

In `crates/e-fees-core/src/export.rs`, add the module with tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_issue_date_valid() {
        assert_eq!(format_issue_date("250115"), "15 Jan 2025");
        assert_eq!(format_issue_date("991231"), "31 Dec 1999");
        assert_eq!(format_issue_date("000101"), "01 Jan 2000");
    }

    #[test]
    fn test_format_issue_date_invalid_returns_fallback() {
        let result = format_issue_date("invalid");
        assert!(!result.is_empty());
        let result = format_issue_date("12345");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_clean_record_key_strips_brackets() {
        assert_eq!(clean_record_key("⟨25_97101⟩"), "25_97101");
        assert_eq!(clean_record_key("25_97101"), "25_97101");
        assert_eq!(clean_record_key(""), "");
    }

    #[test]
    fn test_is_placeholder() {
        assert!(is_placeholder(""));
        assert!(is_placeholder("Project Name"));
        assert!(is_placeholder("sensory design studio"));
        assert!(!is_placeholder("Hilton Dubai"));
        assert!(!is_placeholder("Martin Robert"));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p e-fees-core --lib export`
Expected: FAIL — module doesn't exist yet

- [ ] **Step 3: Implement format_issue_date, clean_record_key, is_placeholder, and PLACEHOLDER_VALUES**

In `crates/e-fees-core/src/export.rs`:

```rust
//! Fee JSON export logic shared between desktop app and API.

use chrono::Utc;
use serde_json::json;

use crate::models::{Fee, Project, Company, Contact};

/// Template placeholder values — used to detect unpopulated fields.
pub const PLACEHOLDER_VALUES: &[&str] = &[
    "Fee Proposal", "Project Name", "Project Activity", "Project Stage",
    "Project Area", "Project City", "Project Country",
    "Client Company", "Client City", "Contact Name", "Contact Position",
    "+971 contact phone", "client@contact.email",
    "dd MMM yyyy", "sensory design studio",
];

/// Strip angle-bracket wrappers from SurrealDB v3 RecordId key strings.
pub fn clean_record_key(key: &str) -> String {
    key.trim_start_matches('\u{27E8}')
        .trim_end_matches('\u{27E9}')
        .to_string()
}

/// Format a fee issue date from YYMMDD format to "dd MMM yyyy".
pub fn format_issue_date(date_str: &str) -> String {
    if date_str.len() == 6 {
        if let (Ok(year), Ok(month), Ok(day)) = (
            date_str[0..2].parse::<i32>(),
            date_str[2..4].parse::<u32>(),
            date_str[4..6].parse::<u32>(),
        ) {
            let full_year = if year >= 50 { 1900 + year } else { 2000 + year };
            if let Some(date) = chrono::NaiveDate::from_ymd_opt(full_year, month, day) {
                return date.format("%d %b %Y").to_string();
            }
        }
    }
    Utc::now().format("%d %b %Y").to_string()
}

/// Check if a field value is a known placeholder.
pub fn is_placeholder(value: &str) -> bool {
    value.is_empty() || PLACEHOLDER_VALUES.contains(&value)
}
```

Add `pub mod export;` to `crates/e-fees-core/src/lib.rs`.

Add `chrono = "0.4"` to `[dependencies]` in `crates/e-fees-core/Cargo.toml`.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p e-fees-core --lib export`
Expected: 4 tests PASS

- [ ] **Step 5: Add build_fee_json with test**

Add to `export.rs`:

```rust
/// Build the 21-field InDesign variable JSON from fee + linked entities.
pub fn build_fee_json(
    fee: &Fee,
    project: &Project,
    company: &Company,
    contact: &Contact,
) -> serde_json::Value {
    let issue_date = format_issue_date(&fee.issue_date);
    let contact_name = contact.full_name.clone().unwrap_or_else(|| {
        let first = contact.first_name.clone().unwrap_or_default();
        let last = contact.last_name.clone().unwrap_or_default();
        format!("{} {}", first, last)
    });

    json!({
        "01 Document Name": fee.name.clone(),
        "02 Document Number": fee.number.clone(),
        "03 Document Release": format!("{:02}", fee.rev),
        "04 Document Issue Date": issue_date,
        "06 Project Name": project.name.clone(),
        "07 Project Activity": fee.activity.clone(),
        "08 Project Package": fee.package.clone(),
        "09 Project Stage": project.status.clone(),
        "11 Project Area": project.area.clone(),
        "12 Project City": project.city.clone(),
        "13 Project Country": project.country.clone(),
        "21 Client Company": company.name.clone(),
        "22 Client City": company.city.clone(),
        "23 Client Country": company.country.clone(),
        "26 Contact Name": contact_name,
        "27 Contact Position": contact.position.clone().unwrap_or_default(),
        "28 Contact Phone": contact.phone.clone().unwrap_or_default(),
        "29 Contact Email": contact.email.clone().unwrap_or_default(),
        "91 Staff Name": fee.staff_name.clone(),
        "92 Staff Position": fee.staff_position.clone(),
        "93 Staff Phone": fee.staff_phone.clone(),
        "94 Staff Email": fee.staff_email.clone(),
        "99 Strap Line": fee.strap_line.clone(),
    })
}

/// Clean a project number for filesystem paths: strip angle brackets, replace underscores with dashes.
/// DB stores `25_97101`, filesystem uses `25-97101`.
pub fn clean_number_for_path(number_id: &str) -> String {
    clean_record_key(number_id).replace('_', "-")
}
```

Add test with actual struct construction:

```rust
    use crate::models::common::TimeStamps;
    use crate::models::project::ProjectNumber;
    use surrealdb::types::RecordId;

    fn test_fee() -> Fee {
        Fee {
            id: None,
            name: "Fee Proposal".into(),
            number: "26-97101-FP".into(),
            rev: 1,
            status: "Draft".into(),
            issue_date: "260315".into(),
            activity: "Design and Consultancy".into(),
            package: "Specialist Lighting".into(),
            project_id: RecordId::from(("projects", "26_97101")),
            company_id: RecordId::from(("company", "test")),
            contact_id: RecordId::from(("contacts", "test")),
            staff_name: "Martin Robert".into(),
            staff_email: "martin@emittiv.com".into(),
            staff_phone: "+971 50 123 4567".into(),
            staff_position: "Director".into(),
            strap_line: "sensory design studio".into(),
            revisions: vec![],
            time: TimeStamps::default(),
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

    fn test_project() -> Project {
        Project {
            id: None,
            name: "Hilton Dubai".into(),
            name_short: "Hilton".into(),
            status: "Design".into(),
            area: "Downtown".into(),
            city: "Dubai".into(),
            country: "UAE".into(),
            folder: "".into(),
            number: ProjectNumber { year: 26, country: 971, seq: 1, id: "26-97101".into() },
            time: TimeStamps::default(),
        }
    }

    fn test_company() -> Company {
        Company {
            id: None,
            name: "Hilton Hotels".into(),
            name_short: "Hilton".into(),
            abbreviation: "HH".into(),
            city: "Dubai".into(),
            country: "UAE".into(),
            reg_no: None,
            tax_no: None,
            time: TimeStamps::default(),
        }
    }

    fn test_contact() -> Contact {
        Contact {
            id: None,
            first_name: Some("John".into()),
            last_name: Some("Doe".into()),
            full_name: Some("John Doe".into()),
            email: Some("john@hilton.com".into()),
            phone: Some("+971 4 123 4567".into()),
            position: Some("Project Manager".into()),
            company: None,
            time: None,
        }
    }

    #[test]
    fn test_build_fee_json_produces_21_fields() {
        let fee = test_fee();
        let project = test_project();
        let company = test_company();
        let contact = test_contact();
        let json = build_fee_json(&fee, &project, &company, &contact);
        let obj = json.as_object().unwrap();
        assert_eq!(obj.len(), 21);
        assert_eq!(obj["06 Project Name"], "Hilton Dubai");
        assert_eq!(obj["04 Document Issue Date"], "15 Mar 2026");
        assert_eq!(obj["21 Client Company"], "Hilton Hotels");
        assert_eq!(obj["26 Contact Name"], "John Doe");
    }

    #[test]
    fn test_clean_number_for_path() {
        assert_eq!(clean_number_for_path("25_97101"), "25-97101");
        assert_eq!(clean_number_for_path("⟨25_97101⟩"), "25-97101");
    }
```

- [ ] **Step 6: Run all core tests**

Run: `cargo test -p e-fees-core --lib`
Expected: All PASS

- [ ] **Step 7: Commit**

```bash
git add crates/e-fees-core/src/export.rs crates/e-fees-core/src/lib.rs
git commit -m "feat(core): extract fee JSON export logic to shared crate"
```

---

### Task 2: Update desktop app to use shared export code

**Files:**
- Modify: `src-tauri/src/commands/fee_json.rs`

- [ ] **Step 1: Replace local implementations with imports**

In `src-tauri/src/commands/fee_json.rs`:

1. Remove the local `format_issue_date` function (lines 64-78)
2. Remove the local `build_fee_json` function (lines 81-119)
3. Add import: `use e_fees_core::export::{build_fee_json, format_issue_date, clean_record_key};`
4. Update `build_fee_json_paths` to use `clean_record_key`:
   ```rust
   let project_number = clean_record_key(&project.number.id.to_string());
   ```
5. **Fix B3 bug** — change `project.name_short` to `project.name` in path construction:
   ```rust
   let project_dir = format!(
       "{}/01 RFPs/{} {}",
       project_folder_path, project_number, project.name
   );
   ```
6. **Remove or update local tests** — the `tests` module at the bottom of `fee_json.rs` (lines 216-236) tests `format_issue_date` directly. These tests now belong in `e-fees-core/src/export.rs` (already added in Task 1). Remove the `#[cfg(test)] mod tests` block from `fee_json.rs` entirely since all tested functions have moved to core.

- [ ] **Step 2: Run desktop tests**

Run: `cargo test -p app --lib`
Expected: All existing tests PASS

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/commands/fee_json.rs
git commit -m "refactor(desktop): use shared export logic from e-fees-core

Fixes B3: path now uses project.name (matches folder creation script)"
```

---

### Task 3: Add SSH helper to API

**Files:**
- Create: `e-fees-api/src/ssh.rs`
- Modify: `e-fees-api/src/main.rs` (add `mod ssh;`)

- [ ] **Step 1: Write shell_quote tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_quote_simple() {
        assert_eq!(shell_quote("hello"), "'hello'");
    }

    #[test]
    fn test_shell_quote_with_spaces() {
        assert_eq!(shell_quote("01 Projects/01 RFPs"), "'01 Projects/01 RFPs'");
    }

    #[test]
    fn test_shell_quote_with_single_quotes() {
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }
}
```

- [ ] **Step 2: Implement SshOps struct**

In `e-fees-api/src/ssh.rs`:

```rust
//! Reusable SSH operations for Nextcloud file management on Primary server.

use tokio::process::Command;
use std::process::Stdio;
use tracing::warn;

use crate::config::FolderConfig;
use crate::error::ApiError;

/// Shell-quote a string for safe use in remote SSH commands.
pub(crate) fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

pub struct SshOps {
    host: String,
    user: String,
    key_path: String,
}

impl SshOps {
    pub fn from_folder_config(cfg: &FolderConfig) -> Self {
        Self {
            host: cfg.ssh_host.clone(),
            user: cfg.ssh_user.clone(),
            key_path: cfg.ssh_key.clone(),
        }
    }

    fn ssh_args(&self) -> Vec<String> {
        vec![
            "-i".into(), self.key_path.clone(),
            "-o".into(), "StrictHostKeyChecking=no".into(),
            "-o".into(), "ConnectTimeout=10".into(),
            format!("{}@{}", self.user, self.host),
        ]
    }

    /// Execute a command string on the remote server.
    pub async fn exec(&self, remote_cmd: &str) -> Result<String, ApiError> {
        let mut args = self.ssh_args();
        args.push(remote_cmd.to_string());
        let output = Command::new("ssh")
            .args(&args)
            .output()
            .await
            .map_err(|e| {
                warn!("SSH exec failed: {}", e);
                ApiError::service_unavailable(format!("SSH connection failed: {}", e))
            })?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("SSH command failed: {}", stderr);
            Err(ApiError::service_unavailable(format!("SSH command failed: {}", stderr.trim())))
        }
    }

    /// Write content to a remote file via stdin pipe.
    pub async fn write_file(&self, remote_path: &str, content: &[u8]) -> Result<(), ApiError> {
        let remote_cmd = format!("cat > {}", shell_quote(remote_path));
        let mut args = self.ssh_args();
        args.push(remote_cmd);
        let mut child = Command::new("ssh")
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ApiError::service_unavailable(format!("SSH spawn failed: {}", e)))?;
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(content).await
                .map_err(|e| ApiError::service_unavailable(format!("SSH write failed: {}", e)))?;
        }
        let output = child.wait_with_output().await
            .map_err(|e| ApiError::service_unavailable(format!("SSH wait failed: {}", e)))?;
        if output.status.success() { Ok(()) } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ApiError::service_unavailable(format!("File write failed: {}", stderr.trim())))
        }
    }

    pub async fn path_exists(&self, remote_path: &str) -> Result<bool, ApiError> {
        let cmd = format!("test -e {} && echo exists", shell_quote(remote_path));
        match self.exec(&cmd).await {
            Ok(out) => Ok(out.trim() == "exists"),
            Err(_) => Ok(false),
        }
    }

    pub async fn copy_file(&self, from: &str, to: &str) -> Result<(), ApiError> {
        self.exec(&format!("cp {} {}", shell_quote(from), shell_quote(to))).await.map(|_| ())
    }

    pub async fn mkdir_p(&self, path: &str) -> Result<(), ApiError> {
        self.exec(&format!("mkdir -p {}", shell_quote(path))).await.map(|_| ())
    }

    pub async fn rename(&self, from: &str, to: &str) -> Result<(), ApiError> {
        self.exec(&format!("mv {} {}", shell_quote(from), shell_quote(to))).await.map(|_| ())
    }

    /// Trigger Nextcloud rescan. Falls back to shallow scan if targeted fails.
    pub async fn nc_rescan(&self, subpath: &str) -> Result<(), ApiError> {
        let cmd = format!(
            "docker exec nextcloud-e occ groupfolders:scan 1 --path={} 2>/dev/null || \
             docker exec nextcloud-e occ groupfolders:scan 1 --shallow 2>/dev/null",
            shell_quote(subpath)
        );
        self.exec(&cmd).await.map(|_| ())
    }
}
```

- [ ] **Step 3: Add `mod ssh;` to main.rs**

Add after existing module declarations.

- [ ] **Step 4: Run tests**

Run: `cargo test -p e-fees-api --lib ssh`
Expected: 3 shell_quote tests PASS

- [ ] **Step 5: Commit**

```bash
git add e-fees-api/src/ssh.rs e-fees-api/src/main.rs
git commit -m "feat(api): add SshOps helper for reusable SSH file operations"
```

---

### Task 4: Refactor folders.rs to use SshOps + add NC_BASE_PATH config

**Files:**
- Modify: `e-fees-api/src/routes/folders.rs`
- Modify: `e-fees-api/src/config.rs`

- [ ] **Step 1: Add nc_base_path to FolderConfig**

In `e-fees-api/src/config.rs`, add field to `FolderConfig`:
```rust
pub nc_base_path: String,
```

Add env var loading:
```rust
nc_base_path: std::env::var("NC_BASE_PATH")
    .unwrap_or_else(|_| "/mnt/user/emittiv/nc/__groupfolders/1/01 Projects".into()),
```

- [ ] **Step 2: Refactor folders.rs to use SshOps**

Replace inline `Command::new("ssh")` block with:
```rust
use crate::ssh::SshOps;

let ssh = SshOps::from_folder_config(folder_config);
let remote_cmd = format!(
    "bash {} {} {}",
    crate::ssh::shell_quote(&folder_config.script_path),
    crate::ssh::shell_quote(&number),
    crate::ssh::shell_quote(name),
);
let stdout = ssh.exec(&remote_cmd).await?;
```

Note: `shell_quote` may need to be made `pub` or `pub(crate)` in `ssh.rs`.

- [ ] **Step 3: Compile check**

Run: `cargo check -p e-fees-api`
Expected: Compiles

- [ ] **Step 4: Commit**

```bash
git add e-fees-api/src/routes/folders.rs e-fees-api/src/config.rs
git commit -m "refactor(api): use SshOps in folder creation, add NC_BASE_PATH config"
```

---

### Task 5: Add project search parameter

**Files:**
- Modify: `e-fees-api/src/routes/projects.rs`

- [ ] **Step 1: Add search to ProjectListParams**

Add `pub search: Option<String>` to the params struct.

- [ ] **Step 2: Update list handler filter logic**

Build combined filter from `status` + `search`:
```rust
let mut clauses = Vec::new();
let mut binds: Vec<(String, serde_json::Value)> = Vec::new();

if let Some(ref status) = params.status {
    validate_status(status, PROJECT_STATUSES, "project")?;
    clauses.push("status = $filter_status".to_string());
    binds.push(("filter_status".into(), json!(status)));
}

if let Some(ref search) = params.search {
    clauses.push(
        "(string::lowercase(name) CONTAINS string::lowercase($search) \
         OR string::lowercase(name_short) CONTAINS string::lowercase($search) \
         OR number.id CONTAINS $search)".to_string()
    );
    binds.push(("search".into(), json!(search)));
}

let filter = if clauses.is_empty() {
    None
} else {
    Some(FilterClause {
        clause: clauses.join(" AND "),
        binds,
    })
};
```

- [ ] **Step 3: Compile check**

Run: `cargo check -p e-fees-api`
Expected: Compiles

- [ ] **Step 4: Commit**

```bash
git add e-fees-api/src/routes/projects.rs
git commit -m "feat(api): add search parameter to project list endpoint"
```

---

### Task 6: Add fee JSON export endpoint

**Files:**
- Create: `e-fees-api/src/routes/fee_export.rs`
- Modify: `e-fees-api/src/routes/mod.rs`
- Modify: `e-fees-api/src/main.rs`

- [ ] **Step 1: Create fee_export module**

Create `e-fees-api/src/routes/fee_export.rs` with:
- `fetch_fee_with_links()` helper — fetches fee + traverses project/company/contact via RecordId keys
- `build_nc_paths()` helper — constructs Nextcloud file paths using `clean_record_key` and `project.name`
- `export_fee_json` handler — full export flow: fetch, build JSON, SSH archive/write, NC rescan

Key implementation details:
- Extract RecordId key for `db.select()`: use `record_id_string(&fee.project_id)` then split on `:` to get the key
- Strip angle brackets via `clean_record_key`, replace underscores with dashes for filesystem paths
- Archive existing `-var.json` to `00 Archive/` with timestamp before overwriting
- Rename template file (`-var Default Values.json` to `-var.json`) if needed
- Best-effort NC rescan (don't fail on rescan errors)

- [ ] **Step 2: Register module and route**

In `routes/mod.rs`: `pub mod fee_export;`
In `main.rs`:
- Route: `.route("/fees/{id}/json-export", post(routes::fee_export::export_fee_json))`
- OpenAPI: add `routes::fee_export::export_fee_json` to paths list

- [ ] **Step 3: Compile check**

Run: `cargo check -p e-fees-api`
Expected: Compiles

- [ ] **Step 4: Commit**

```bash
git add e-fees-api/src/routes/fee_export.rs e-fees-api/src/routes/mod.rs e-fees-api/src/main.rs
git commit -m "feat(api): add fee JSON export endpoint"
```

---

### Task 7: Add fee JSON status endpoint

**Files:**
- Modify: `e-fees-api/src/routes/fee_export.rs`
- Modify: `e-fees-api/src/main.rs`

- [ ] **Step 1: Add fee_json_status handler**

In `fee_export.rs`, add handler that:
- Fetches fee + linked entities via `fetch_fee_with_links`
- Calls `build_fee_json()` to get the 21-field JSON
- Iterates fields, classifies each as `populated` or `placeholder` via `is_placeholder()`
- Returns counts and per-field status

- [ ] **Step 2: Register route and OpenAPI**

In `main.rs`:
- Route: `.route("/fees/{id}/json-status", get(routes::fee_export::fee_json_status))`
- OpenAPI: add to paths list

- [ ] **Step 3: Compile check and commit**

Run: `cargo check -p e-fees-api`

```bash
git add e-fees-api/src/routes/fee_export.rs e-fees-api/src/main.rs
git commit -m "feat(api): add fee JSON status endpoint"
```

---

### Task 8: Add document upload endpoint

**Files:**
- Create: `e-fees-api/src/routes/documents.rs`
- Modify: `e-fees-api/Cargo.toml`
- Modify: `e-fees-api/src/routes/mod.rs`
- Modify: `e-fees-api/src/main.rs`

- [ ] **Step 1: Add axum-extra dependency**

Run: `cd e-fees-api && cargo add axum-extra --features multipart`

Or add manually to `Cargo.toml`. Check version compatibility with axum 0.8.

- [ ] **Step 2: Write sanitization tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename_valid() {
        assert_eq!(sanitize_filename("report.pdf").unwrap(), "report.pdf");
        assert_eq!(sanitize_filename("path/to/file.pdf").unwrap(), "file.pdf");
    }

    #[test]
    fn test_sanitize_filename_rejects_unsafe() {
        assert!(sanitize_filename("file;rm -rf.pdf").is_err());
        assert!(sanitize_filename("$(whoami).pdf").is_err());
        assert!(sanitize_filename("..").is_err());
        assert!(sanitize_filename("").is_err());
    }

    #[test]
    fn test_validate_subfolder_valid() {
        assert!(validate_subfolder("01 Client Info/01 Pre Award").is_ok());
    }

    #[test]
    fn test_validate_subfolder_rejects_traversal() {
        assert!(validate_subfolder("../etc").is_err());
        assert!(validate_subfolder("/absolute/path").is_err());
    }
}
```

- [ ] **Step 3: Implement sanitization helpers and upload handler**

Create `e-fees-api/src/routes/documents.rs` with:
- `sanitize_filename()` — strips path components, rejects shell metacharacters
- `validate_subfolder()` — rejects `..` and absolute paths
- `upload_document` handler — multipart extraction, SSH write via `SshOps`, NC rescan

- [ ] **Step 4: Register module and route**

In `routes/mod.rs`: `pub mod documents;`
In `main.rs`:
- Route: `.route("/projects/{id}/documents", post(routes::documents::upload_document))`
- OpenAPI: add to paths list

- [ ] **Step 5: Run tests and compile check**

Run: `cargo test -p e-fees-api --lib documents`
Run: `cargo check -p e-fees-api`
Expected: All PASS, compiles

- [ ] **Step 6: Commit**

```bash
git add e-fees-api/Cargo.toml e-fees-api/src/routes/documents.rs e-fees-api/src/routes/mod.rs e-fees-api/src/main.rs
git commit -m "feat(api): add document upload endpoint with multipart support"
```

---

### Task 9: Fix issue_date test + add integration tests

**Files:**
- Modify: `e-fees-api/tests/integration_tests.rs`

- [ ] **Step 1: Fix issue_date format**

Find the existing fee creation test. Change `issue_date` from `"202603"` to valid YYMMDD format: `"260315"`.

- [ ] **Step 2: Add project search integration test**

Test: create project with `DELETE ME` prefix, search by name (match), search by nonexistent term (no match), cleanup.

- [ ] **Step 3: Add fee JSON status integration test**

Test: create company/contact/project/fee with `DELETE ME` prefixes, call `GET /fees/{id}/json-status`, verify `total_fields == 21`, verify `populated > 0`, cleanup in reverse order.

- [ ] **Step 4: Run integration tests**

Run: `API_BASE_URL=http://10.0.21.80:3200 API_KEY=efees-api-2026-k8x9m4pq cargo test -p e-fees-api --test integration_tests -- --test-threads=1`
Expected: All PASS

Note: JSON export and document upload integration tests require SSH access to Primary from the test runner. Mark as `#[ignore]` if running without SSH, or test manually after deploy.

- [ ] **Step 5: Commit**

```bash
git add e-fees-api/tests/integration_tests.rs
git commit -m "test(api): fix issue_date format, add search and json-status integration tests"
```

---

### Task 10: Build, deploy, and verify

- [ ] **Step 1: Bump version in Cargo.toml**

Update `e-fees-api/Cargo.toml` version from `"0.2.0"` to `"0.3.0"`. Also update the version string in `health.rs` if it's hardcoded there (check `health::health` handler for version source).

- [ ] **Step 2: Run full test suite**

```bash
cargo test -p e-fees-core --lib
cargo test -p e-fees-api --lib
cargo test -p app --lib
```
All must PASS.

- [ ] **Step 3: Commit version bump**

```bash
git add e-fees-api/Cargo.toml
git commit -m "chore: bump e-fees-api version to 0.3.0"
```

- [ ] **Step 4: Build and deploy API container**

SSH to AI server, pull latest code, rebuild container as v0.3.0 with `NC_BASE_PATH` env var. Follow existing container deployment pattern (check current Docker run config first, use Unraid tools).

- [ ] **Step 5: Verify health**

```bash
curl -s http://10.0.21.80:3200/health
```
Expected: `"status": "ok"`, version `0.3.0`

- [ ] **Step 6: Verify Swagger shows all new endpoints**

Check `http://10.0.21.80:3200/docs/` — confirm 4 new endpoints appear:
- `POST /fees/{id}/json-export`
- `GET /fees/{id}/json-status`
- `GET /projects?search=...`
- `POST /projects/{id}/documents`

- [ ] **Step 7: Manual smoke test**

```bash
# Test project search
curl -s -H "X-API-Key: efees-api-2026-k8x9m4pq" "http://10.0.21.80:3200/projects?search=Hilton"

# Test fee JSON status (use a real fee ID from the DB)
curl -s -H "X-API-Key: efees-api-2026-k8x9m4pq" "http://10.0.21.80:3200/fees/25_97101_1/json-status"

# Test fee JSON export (use a real fee ID — will write to Nextcloud)
curl -s -X POST -H "X-API-Key: efees-api-2026-k8x9m4pq" "http://10.0.21.80:3200/fees/25_97101_1/json-export"
```

- [ ] **Step 8: Notify PA via hub**

Send hub message confirming endpoints are live and ready for validation.
