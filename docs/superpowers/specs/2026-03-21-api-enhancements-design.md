# E-Fees API Enhancements — Design Spec

**Date:** 2026-03-21
**Status:** Approved
**Motivation:** PA instance needs API endpoints for automated RFP intake pipeline — project creation, folder provisioning, document storage, and fee JSON export.

## Overview

Four new API capabilities for `e-fees-api` (10.0.21.80:3200), plus shared code extraction to `e-fees-core`:

1. **Fee JSON Export** — `POST /fees/{id}/json-export`
2. **Project Search** — `GET /projects?search=...` (enhance existing)
3. **Document Upload** — `POST /projects/{id}/documents`
4. **Fee JSON Status** — `GET /fees/{id}/json-status`

## Shared Code: `e-fees-core/src/export.rs`

Move fee JSON export logic from `src-tauri/src/commands/fee_json.rs` to the shared crate so both the desktop app and API can use it.

### Functions to move

```rust
/// Build the 21-field InDesign variable JSON from fee + linked entities.
pub fn build_fee_json(fee, project, company, contact) -> serde_json::Value

/// Parse YYMMDD string to formatted date "29 Jul 2022".
pub fn format_issue_date(date_str: &str) -> String
```

### Constants

```rust
/// Template placeholder values — used by json-status to detect unpopulated fields.
pub const PLACEHOLDER_VALUES: &[&str] = &[
    "Fee Proposal", "Project Name", "Project Activity", "Project Stage",
    "Project Area", "Project City", "Project Country",
    "Client Company", "Client City", "Contact Name", "Contact Position",
    "+971 contact phone", "client@contact.email",
    "dd MMM yyyy", "sensory design studio",
];
```

### Dependencies

`chrono` and `serde_json` — already in `e-fees-core/Cargo.toml`.

### Shared helper: `clean_record_key`

SurrealDB v3 RecordId keys may contain angle-bracket wrappers (`⟨`, `⟩`). Add a utility to `e-fees-core` for stripping these:

```rust
/// Strip angle-bracket wrappers from SurrealDB v3 RecordId key strings.
pub fn clean_record_key(key: &str) -> String {
    key.trim_start_matches('⟨').trim_end_matches('⟩').to_string()
}
```

Used by `build_fee_json` (for project number) and by the API endpoint (for constructing file paths). The desktop's `fee_json.rs:133-135` already does this inline — replace with the shared helper.

### Desktop app update

`src-tauri/src/commands/fee_json.rs` changes:

**Remove:** `build_fee_json()`, `format_issue_date()` local implementations.

**Add:**
```rust
use e_fees_core::export::{build_fee_json, format_issue_date, clean_record_key};
```

**Keep unchanged** (these depend on `crate::db` types and filesystem — desktop-only):
- `find_fee_by_id` (uses `crate::db::types::record_key_string`)
- `find_project_for_fee`, `find_company_for_fee`, `find_contact_for_fee` (use `crate::db::types::record_key_string`)
- `FeeJsonPaths`, `build_fee_json_paths` — update to use `clean_record_key()` instead of inline `.replace("⟨", "").replace("⟩", "")`
- `rename_template_file_if_needed`, `write_json_to_file` (filesystem operations)

**Import note:** The `find_*` helpers reference `crate::db::types::record_key_string` which stays in the desktop crate. Only pure-logic functions move to core.

## SSH Helper: `e-fees-api/src/ssh.rs`

Extract reusable SSH operations from `folders.rs`. Used by json-export, document upload, and folder creation.

```rust
pub struct SshOps {
    host: String,
    user: String,
    key_path: String,
}

impl SshOps {
    pub fn from_folder_config(cfg: &FolderConfig) -> Self

    /// Execute a command on the remote server. Returns stdout on success.
    pub async fn exec(&self, command: &str) -> Result<String, ApiError>

    /// Write content to a file on the remote server via stdin pipe.
    pub async fn write_file(&self, remote_path: &str, content: &[u8]) -> Result<(), ApiError>

    /// Check if a remote path exists.
    pub async fn path_exists(&self, remote_path: &str) -> Result<bool, ApiError>

    /// Copy/move a remote file.
    pub async fn copy_file(&self, from: &str, to: &str) -> Result<(), ApiError>

    /// Trigger Nextcloud rescan for a specific path.
    pub async fn nc_rescan(&self, subpath: &str) -> Result<(), ApiError>
}
```

### Shell quoting for paths with spaces

Nextcloud paths contain spaces by design (e.g. `01 Projects/01 RFPs/26-97101 Project Name/`). SSH requires a remote command string, so pure argument-array isolation is not possible for the remote side. The `SshOps` methods must:

1. Use `tokio::process::Command` argument arrays for the **local** SSH invocation (host, key, user)
2. Build the **remote** command as a single string passed as the final SSH argument (same pattern as existing `folders.rs:63-68`)
3. Shell-quote all path arguments using single quotes with internal single-quote escaping (`'` → `'\''`). Add a `fn shell_quote(s: &str) -> String` helper.
4. For `write_file`, pipe content via stdin: `ssh user@host "cat > '/quoted/path/file.json'"` with content on stdin via `Command::stdin(Stdio::piped())`

User-supplied values (filenames, subfolder paths) are sanitized before shell-quoting as defense in depth.

Refactor `folders.rs` to use `SshOps` instead of inline `Command::new("ssh")`.

## Endpoint 1: `POST /fees/{id}/json-export`

**File:** `e-fees-api/src/routes/fee_export.rs`
**Tag:** `Fees`

### Request

```
POST /fees/{id}/json-export
Header: X-API-Key: ...
```

No request body. The fee ID determines everything.

### Logic

1. Validate ID
2. Fetch fee: `state.db.select(("fee", &*id))` -> 404 if not found
3. Fetch linked records:
   - `state.db.select(("projects", project_key))` using `fee.project_id`
   - `state.db.select(("company", company_key))` using `fee.company_id`
   - `state.db.select(("contacts", contact_key))` using `fee.contact_id`
   - 404 with descriptive message if any linked record missing
4. Call `e_fees_core::export::build_fee_json(&fee, &project, &company, &contact)`
5. Determine paths — **IMPORTANT: path naming inconsistency**:
   - `folders.rs` (folder creation) uses `project.name` (full name) for the folder
   - `fee_json.rs` (desktop export) uses `project.name_short` for path lookup
   - The nc-project-create.sh script receives `name` as its second argument
   - **Resolution:** The folder is created with `project.name`. All path lookups must use `project.name` to match. The desktop's `fee_json.rs:136` using `name_short` is a latent bug — fix it when moving to core.
   - `number` must be cleaned via `clean_record_key()` and have underscores replaced with dashes (DB stores `25_97101`, filesystem uses `25-97101`)
   - `project_dir = "{nc_base}/01 RFPs/{number} {name}"`
   - `json_path = "{project_dir}/02 Proposal/{number}-var.json"`
   - `template_path = "{project_dir}/02 Proposal/{number}-var Default Values.json"`
   - `archive_path = "{project_dir}/02 Proposal/00 Archive/{number}-var-{YYYYMMDD-HHMMSS}.json"`
6. Via SSH:
   - If template file exists and var.json doesn't -> rename template to var.json
   - If var.json exists -> copy to archive path (create `00 Archive/` dir if needed)
   - Write new JSON content to var.json
   - Trigger NC rescan for project folder
7. Return response

### Response (200)

```json
{
  "status": "exported",
  "fee_id": "fee:26_97101_1",
  "path": "01 RFPs/26-97101 Project Name/02 Proposal/26-97101-var.json",
  "fields_populated": 21,
  "archived_previous": true
}
```

### Errors

- 404: Fee, project, company, or contact not found
- 503: SSH/folder operations failed (FolderConfig not set, SSH error)

### Nextcloud base path

Add `nc_base_path` field to `FolderConfig`:
```
NC_BASE_PATH=/mnt/user/emittiv/nc/__groupfolders/1/01 Projects
```
Default: `/mnt/user/emittiv/nc/__groupfolders/1/01 Projects`

## Endpoint 2: `GET /projects?search=...`

**File:** `e-fees-api/src/routes/projects.rs` (modify existing)

### Changes

Add `search` parameter to `ProjectListParams`:

```rust
#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ProjectListParams {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub status: Option<String>,
    pub search: Option<String>,  // NEW
}
```

### Filter logic

When `search` is provided, add to WHERE clause:
```sql
(string::lowercase(name) CONTAINS string::lowercase($search)
 OR string::lowercase(name_short) CONTAINS string::lowercase($search)
 OR number.id CONTAINS $search)
```

Combine with existing `status` filter using AND when both present.

### Response

Same paginated format — no change to response shape.

## Endpoint 3: `POST /projects/{id}/documents`

**File:** `e-fees-api/src/routes/documents.rs`
**Tag:** `Projects`
**Dependency:** `axum-extra` with `multipart` feature

### Request

```
POST /projects/{id}/documents
Header: X-API-Key: ...
Content-Type: multipart/form-data

Parts:
  - file: binary (required) — the document to upload
  - subfolder: string (optional) — destination subfolder within project
    Default: "01 Client Info/01 Pre Award"
```

No file size limit enforced by the API.

### Logic

1. Validate project ID, fetch project -> 404 if not found
2. Extract multipart parts: `file` (required), `subfolder` (optional)
3. Sanitize filename: strip path components, reject names with shell-unsafe chars
4. Determine destination: `{nc_base}/01 RFPs/{number} {name}/{subfolder}/{filename}`
5. Via SSH:
   - Create destination directory if needed (`mkdir -p`)
   - Write file content via stdin pipe
   - Trigger NC rescan for project folder
6. Return response

### Response (200)

```json
{
  "status": "uploaded",
  "project_id": "projects:26_97101",
  "filename": "RFP-Document.pdf",
  "path": "01 RFPs/26-97101 Project Name/01 Client Info/01 Pre Award/RFP-Document.pdf",
  "size_bytes": 2458624
}
```

### Errors

- 400: Missing file part, invalid filename
- 404: Project not found
- 503: SSH/folder operations failed

### Security

- Filename sanitization: no `..`, no `/`, no shell metacharacters
- Subfolder validation: no `..` components, must be relative path
- Content is written as-is (no parsing/validation of file contents)
- SSH commands use argument arrays via tokio::process::Command, not shell interpolation

## Endpoint 4: `GET /fees/{id}/json-status`

**File:** `e-fees-api/src/routes/fee_export.rs` (same as export)
**Tag:** `Fees`

### Request

```
GET /fees/{id}/json-status
Header: X-API-Key: ...
```

### Logic

1. Fetch fee + linked project, company, contact (same as json-export)
2. Call `build_fee_json()` to get the 21-field JSON
3. For each field, classify as:
   - `"populated"` — non-empty AND not in `PLACEHOLDER_VALUES`
   - `"placeholder"` — empty string OR matches a known placeholder

### Response (200)

```json
{
  "fee_id": "fee:26_97101_1",
  "total_fields": 21,
  "populated": 18,
  "placeholder": 3,
  "fields": {
    "01 Document Name": { "status": "populated", "value": "Fee Proposal" },
    "26 Contact Name": { "status": "placeholder", "value": "" }
  }
}
```

## Files Changed

| File | Action | Description |
|------|--------|-------------|
| `crates/e-fees-core/src/export.rs` | **Create** | `build_fee_json()`, `format_issue_date()`, `PLACEHOLDER_VALUES` |
| `crates/e-fees-core/src/lib.rs` | **Modify** | Add `pub mod export;` |
| `src-tauri/src/commands/fee_json.rs` | **Modify** | Replace local fns with `use e_fees_core::export::*` |
| `e-fees-api/src/ssh.rs` | **Create** | `SshOps` helper struct |
| `e-fees-api/src/routes/fee_export.rs` | **Create** | json-export + json-status endpoints |
| `e-fees-api/src/routes/documents.rs` | **Create** | Document upload endpoint |
| `e-fees-api/src/routes/projects.rs` | **Modify** | Add `search` parameter |
| `e-fees-api/src/routes/folders.rs` | **Modify** | Refactor to use `SshOps` |
| `e-fees-api/src/routes/mod.rs` | **Modify** | Add new modules |
| `e-fees-api/src/main.rs` | **Modify** | Register routes + OpenAPI paths |
| `e-fees-api/src/config.rs` | **Modify** | Add `NC_BASE_PATH` to `FolderConfig` |
| `e-fees-api/Cargo.toml` | **Modify** | Add `axum-extra` with multipart |
| `e-fees-api/tests/integration_tests.rs` | **Modify** | Fix issue_date format, add tests for new endpoints |

## Testing Strategy

### Unit tests (in e-fees-core)
- `format_issue_date` — valid YYMMDD, edge cases, invalid input
- `build_fee_json` — verify all 21 fields populated correctly
- Placeholder detection — verify classification logic

### Integration tests (in e-fees-api)
- **JSON export:** Create fee with prerequisites -> export -> verify response -> cleanup
- **Project search:** Create test project -> search by name, name_short, number -> verify results -> cleanup
- **Document upload:** Create project + folder -> upload file -> verify response -> cleanup
- **JSON status:** Create fee -> check status -> verify field counts -> cleanup
- All test data uses `DELETE ME` prefix
- Production safety guard on all tests

### Existing tests
- Desktop app tests (`cargo test -p app --lib`) must still pass after moving functions to core
- Existing API integration tests must pass with issue_date fix

## OpenAPI Updates

New paths added to `ApiDoc` in `main.rs`:
- `routes::fee_export::export_fee_json`
- `routes::fee_export::fee_json_status`
- `routes::documents::upload_document`

Updated paths:
- `routes::projects::list_projects` (new `search` param visible in Swagger)

No new tags needed — all under existing `Fees` and `Projects` tags.

## Config Changes

New env var for API container:
```
NC_BASE_PATH=/mnt/user/emittiv/nc/__groupfolders/1/01 Projects
```

Default value hardcoded if not set. Only needed if the Nextcloud path changes.
