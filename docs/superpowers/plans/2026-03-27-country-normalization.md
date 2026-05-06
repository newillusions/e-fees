# Country Normalization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Accept any reasonable country input (ISO 2/3-letter codes, display names, official names) across all API consumers and normalize to canonical `country.name` before storage.

**Architecture:** SurrealDB stored function `fn::resolve_country` with cascading match logic. New `iso2` field on country table for 2-letter ISO codes. All create handlers call the function and store the canonical name. Old logic commented out for easy revert.

**Tech Stack:** SurrealDB v3.0.4 (stored functions), Rust (e-fees-api, agent_server, Tauri app)

**Spec:** `docs/superpowers/specs/2026-03-27-country-normalization-design.md`

---

### Task 0: Schema Migration — Add `iso2` Field + Stored Function

**Files:**
- Create: `scripts/migration/005-country-normalization.surql`

This task runs directly against the **production** database (10.0.23.11). The function has already been validated on dev DB (10.0.23.12).

- [ ] **Step 1: Create migration file**

```sql
-- 005-country-normalization.surql
-- Adds iso2 field to country table and defines fn::resolve_country

-- 1. Add iso2 field and index
DEFINE FIELD iso2 ON country TYPE option<string>;
DEFINE INDEX idx_country_iso2 ON country FIELDS iso2;

-- 2. Populate iso2 for all countries
-- Key GCC + common countries (PA's primary coverage)
UPDATE country:UAE SET iso2 = "AE";
UPDATE country:KSA SET iso2 = "SA";
UPDATE country:KUW SET iso2 = "KW";
UPDATE country:GBR SET iso2 = "GB";
UPDATE country:USA SET iso2 = "US";
UPDATE country:IND SET iso2 = "IN";
UPDATE country:FRA SET iso2 = "FR";
UPDATE country:CAN SET iso2 = "CA";
UPDATE country:AUS SET iso2 = "AU";
-- Full ISO 3166-1 alpha-2 population follows
-- (see Step 2 for the complete list generated from the country table)

-- 3. Define resolve function
DEFINE FUNCTION fn::resolve_country($input: string) {
    LET $clean = string::trim($input);
    LET $lower = string::lowercase($clean);
    LET $nodots = string::replace($lower, ".", "");

    -- 1. code exact (3-letter: "UAE", "KSA")
    LET $r = SELECT name, code, iso2, dial_code FROM country
        WHERE code = string::uppercase($clean) LIMIT 1;
    IF array::len($r) > 0 { RETURN $r[0]; };

    -- 2. iso2 exact (2-letter: "AE", "SA")
    LET $r = SELECT name, code, iso2, dial_code FROM country
        WHERE iso2 = string::uppercase($clean) LIMIT 1;
    IF array::len($r) > 0 { RETURN $r[0]; };

    -- 3. code_alt exact (alt 3-letter: "ARE", "SAU")
    LET $r = SELECT name, code, iso2, dial_code FROM country
        WHERE code_alt = string::uppercase($clean) LIMIT 1;
    IF array::len($r) > 0 { RETURN $r[0]; };

    -- 4. name exact ("U.A.E.", "K.S.A.", "India")
    LET $r = SELECT name, code, iso2, dial_code FROM country
        WHERE name = $clean LIMIT 1;
    IF array::len($r) > 0 { RETURN $r[0]; };

    -- 5. name dot-stripped ("UAE" → matches "U.A.E." after removing dots)
    LET $r = SELECT name, code, iso2, dial_code FROM country
        WHERE string::lowercase(string::replace(name, ".", "")) = $nodots LIMIT 1;
    IF array::len($r) > 0 { RETURN $r[0]; };

    -- 6. name_official / name_formal exact (case-insensitive)
    LET $r = SELECT name, code, iso2, dial_code FROM country
        WHERE string::lowercase(name_official) = $lower
           OR string::lowercase(name_formal) = $lower LIMIT 1;
    IF array::len($r) > 0 { RETURN $r[0]; };

    -- 7. name_official / name_formal contains (case-insensitive)
    LET $r = SELECT name, code, iso2, dial_code FROM country
        WHERE string::lowercase(name_official) CONTAINS $lower
           OR string::lowercase(name_formal) CONTAINS $lower LIMIT 1;
    IF array::len($r) > 0 { RETURN $r[0]; };

    RETURN NONE;
};
```

- [ ] **Step 2: Generate full iso2 population script**

Query the country table to get all records and their `code` / `code_alt` fields. Map each to its ISO 3166-1 alpha-2 code. The `code_alt` field already has 3-letter ISO 3166-1 alpha-3 codes — use a standard mapping from alpha-3 to alpha-2.

For records where `code_alt` maps cleanly to a 2-letter code (most countries), generate:
```sql
UPDATE country:<id> SET iso2 = "<2-letter>";
```

Append all UPDATE statements to the migration file.

- [ ] **Step 3: Run migration on production DB**

```bash
# Use /rpc endpoint (not /sql) per KB rule for complex text
curl -s -X POST "http://10.0.23.11:8000/rpc" \
  -u "martin:<redacted-rotated-2026-05-06>" \
  -H "surreal-ns: emittiv" \
  -H "surreal-db: projects" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{"method":"query","params":["<migration SQL here>"]}'
```

Expected: All statements return `"status": "OK"`.

- [ ] **Step 4: Verify function on production**

```bash
# Test key inputs
for input in "UAE" "AE" "ARE" "U.A.E." "KSA" "SA" "India" "GB" "uk" "XYZZY"; do
  curl -s -X POST "http://10.0.23.11:8000/rpc" \
    -u "martin:<redacted-rotated-2026-05-06>" \
    -H "surreal-ns: emittiv" \
    -H "surreal-db: projects" \
    -H "Accept: application/json" \
    -H "Content-Type: application/json" \
    -d "{\"method\":\"query\",\"params\":[\"RETURN fn::resolve_country(\\\"$input\\\");\"]}"
done
```

Expected: Same results as dev DB testing — all inputs resolve correctly, "XYZZY" returns NONE.

- [ ] **Step 5: Commit migration file**

```bash
git add scripts/migration/005-country-normalization.surql
git commit -m "feat(db): add country normalization function and iso2 field

Adds fn::resolve_country() stored function with 7-step cascade matching.
Adds iso2 field (ISO 3166-1 alpha-2) to country table with index.
Populates iso2 for all ~250 countries.

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

### Task 1: Update `e-fees-api` Project Creation

**Files:**
- Modify: `e-fees-api/src/routes/projects.rs:166-253`

- [ ] **Step 1: Add resolve_country helper function**

Add a new function that calls `fn::resolve_country` via the DB, replacing the old `lookup_dial_code`. Place it after the commented-out `lookup_dial_code` function.

```rust
/// Resolve a country input to canonical name and dial code using DB function.
/// Accepts: ISO 2/3-letter codes, display names, official names.
/// Returns: (canonical_name, dial_code)
async fn resolve_country(db: &Surreal<Client>, input: &str) -> Result<(String, u16), ApiError> {
    if input.is_empty() || input.len() > 100 {
        return Err(ApiError::bad_request("Invalid country input"));
    }

    let mut response = db
        .query("RETURN fn::resolve_country($input);")
        .bind(("input", input.to_string()))
        .await?;
    let result: Option<serde_json::Value> = response.take(0)?;

    let obj = result.ok_or_else(|| {
        ApiError::bad_request(format!(
            "Unknown country: '{}'. Provide an ISO code (AE, SA), name (UAE, India), or display name (U.A.E.).",
            input
        ))
    })?;

    let name = obj.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::internal("Country resolve returned no name"))?
        .to_string();

    let dial_code = obj.get("dial_code")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ApiError::internal("Country resolve returned no dial_code"))?;

    let dc = u16::try_from(dial_code).map_err(|_| {
        ApiError::bad_request(format!("Dial code {} out of range", dial_code))
    })?;

    Ok((name, dc))
}
```

- [ ] **Step 2: Comment out old `lookup_dial_code`**

At `e-fees-api/src/routes/projects.rs:223-253`, wrap the entire function in a block comment:

```rust
/* COMMENTED OUT — replaced by resolve_country() which uses fn::resolve_country DB function
/// Look up a country's dial code from the reference data table.
async fn lookup_dial_code(db: &Surreal<Client>, country_name: &str) -> Result<u16, ApiError> {
    ...existing code...
}
*/
```

- [ ] **Step 3: Update `create_project` to use `resolve_country`**

At `e-fees-api/src/routes/projects.rs:166-221`, change the project creation to normalize country before storage:

Replace lines 177-178:
```rust
            let country_code = lookup_dial_code(&state.db, &body.country).await?;
```

With:
```rust
            let (_, country_code) = resolve_country(&state.db, &body.country).await?;
```

And add country normalization before the DB write. Before line 206 (the `.bind("country", ...)` line), add normalization:

```rust
    // Normalize country name via DB function
    let country_name = match resolve_country(&state.db, &body.country).await {
        Ok((name, _)) => name,
        Err(_) => body.country.clone(), // Fall back to raw input if resolve fails
    };
```

Then change the bind from:
```rust
        .bind(("country", body.country))
```
To:
```rust
        .bind(("country", country_name))
```

Note: When `body.number` is `Some(n)` (number provided), the `resolve_country` call for dial code is skipped, but we still normalize the country name for storage.

- [ ] **Step 4: Build and verify**

```bash
cd e-fees-api && cargo build 2>&1 | tail -5
```

Expected: Compiles with no errors. Warning about unused `lookup_dial_code` is expected (it's commented out).

- [ ] **Step 5: Commit**

```bash
git add e-fees-api/src/routes/projects.rs
git commit -m "feat(api): use fn::resolve_country for project creation

Replaces lookup_dial_code with resolve_country() that calls the DB stored
function. Normalizes country name before storage. Old function commented out.

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

### Task 2: Update `e-fees-api` Company Creation

**Files:**
- Modify: `e-fees-api/src/routes/companies.rs:100-123`

- [ ] **Step 1: Add resolve_country to companies module**

The `resolve_country` function is in `projects.rs`. Either:
- (a) Move it to a shared module (e.g., `e-fees-api/src/country.rs`) and re-export, OR
- (b) Make it `pub(crate)` in projects.rs and import in companies.rs

Option (b) is simpler — change `async fn resolve_country` to `pub(crate) async fn resolve_country` in `projects.rs`.

Then in `companies.rs`, add the import:
```rust
use crate::routes::projects::resolve_country;
```

- [ ] **Step 2: Normalize country in `create_company`**

At `e-fees-api/src/routes/companies.rs:100-123`, add normalization before the DB write.

After the `require_non_empty` check, add:
```rust
    // Normalize country name
    let country_name = match resolve_country(&state.db, &body.country).await {
        Ok((name, _)) => name,
        Err(_) => body.country.clone(),
    };
```

Then change the bind from:
```rust
        .bind(("country", body.country))
```
To:
```rust
        .bind(("country", country_name))
```

- [ ] **Step 3: Build and verify**

```bash
cd e-fees-api && cargo build 2>&1 | tail -5
```

Expected: Compiles cleanly.

- [ ] **Step 4: Commit**

```bash
git add e-fees-api/src/routes/companies.rs e-fees-api/src/routes/projects.rs
git commit -m "feat(api): normalize country in company creation

Uses resolve_country() for company create handler. Made resolve_country
pub(crate) for cross-module access.

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

### Task 3: Update Agent Server

**Files:**
- Modify: `src-tauri/src/agent_server.rs:343-399` (create_project_handler)
- Modify: `src-tauri/src/agent_server.rs:594-630` (create_company_handler)

- [ ] **Step 1: Add resolve_country helper to agent_server**

Add a helper function in `agent_server.rs` that calls the DB stored function via the DatabaseManager:

```rust
/// Resolve country input to canonical name using DB stored function.
async fn resolve_country_name(
    db: &tokio::sync::RwLock<crate::db::DatabaseManager>,
    input: &str,
) -> Result<String, String> {
    let manager = db.read().await;
    let result: Option<serde_json::Value> = manager
        .query("RETURN fn::resolve_country($input);", ("input", input.to_string()))
        .await
        .map_err(|e| format!("Country resolve failed: {}", e))?;

    match result {
        Some(obj) => obj.get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| format!("Country resolve returned no name for '{}'", input)),
        None => Err(format!(
            "Unknown country: '{}'. Provide an ISO code (AE, SA), name (UAE, India), or display name (U.A.E.).",
            input
        )),
    }
}
```

Note: Check the exact `DatabaseManager` query API — it may use `query_bind` or similar. Match the existing pattern in `agent_server.rs` for DB queries.

- [ ] **Step 2: Normalize country in `create_project_handler`**

At line 365, before `country: req.country`, add normalization:

```rust
    let country = resolve_country_name(&state.db, &req.country).await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;
```

Then change line 365 from:
```rust
        country: req.country,
```
To:
```rust
        country,
```

- [ ] **Step 3: Normalize country in `create_company_handler`**

At line 603, before `country: req.country`, add normalization:

```rust
    let country = resolve_country_name(&state.db, &req.country).await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })))?;
```

Then change line 603 from:
```rust
        country: req.country,
```
To:
```rust
        country,
```

- [ ] **Step 4: Build and verify**

```bash
cd /Volumes/base/dev/app/e-fees && cargo build -p app 2>&1 | tail -5
```

Expected: Compiles cleanly.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/agent_server.rs
git commit -m "feat(agent): normalize country in project/company creation

Adds resolve_country_name() helper that calls fn::resolve_country DB function.
Applied to both create_project_handler and create_company_handler.

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

### Task 4: Update Tauri Desktop App

**Files:**
- Modify: `src-tauri/src/db/operations.rs:400-474` (generate_next_project_number)
- Modify: `src-tauri/src/db/client.rs:253-278` (create_new_project)
- Modify: `src-tauri/src/db/client.rs:325-344` (create_company)

- [ ] **Step 1: Add resolve_country to DatabaseClient**

In `src-tauri/src/db/client.rs`, add a method to DatabaseClient:

```rust
/// Resolve country input to canonical record using DB stored function.
/// Returns: { name, code, iso2, dial_code } or None.
pub async fn resolve_country(&self, input: &str) -> Result<Option<serde_json::Value>, Error> {
    let query = "RETURN fn::resolve_country($input);";
    let mut response = self.query_bind(query, ("input", input.to_string())).await?;
    let result: Option<serde_json::Value> = response.take(0)?;
    Ok(result)
}
```

Check `query_bind` exists — the explore found it used at operations.rs line 411. If the signature differs, match it.

- [ ] **Step 2: Comment out old dial code lookup in `generate_next_project_number`**

At `operations.rs:410`, comment out the exact-match query:

```rust
    /* COMMENTED OUT — replaced by fn::resolve_country
    let country_lookup_query = "SELECT dial_code FROM country WHERE name = $name LIMIT 1";
    let mut country_response = client.query_bind(country_lookup_query, ("name", country_name.to_string())).await?;
    ...dial code extraction...
    */
```

Replace with:

```rust
    let resolved = client.resolve_country(country_name).await?
        .ok_or_else(|| Error::Api(surrealdb::error::Api::Query(format!(
            "Unknown country: '{}'. Provide an ISO code (AE, SA), name (UAE, India), or display name (U.A.E.).",
            country_name
        ))))?;

    let dial_code = resolved.get("dial_code")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| Error::Api(surrealdb::error::Api::Query(
            "Country resolve returned no dial_code".to_string()
        )))? as u32;
```

Ensure the `dial_code` variable type matches what the rest of the function expects (check if it's u16, u32, or i64 downstream).

- [ ] **Step 3: Normalize country in `create_new_project`**

At `client.rs:262`, before the country format string, resolve:

```rust
    // Resolve country to canonical name
    let country_resolved = self.resolve_country(&project.country).await?;
    let country_name = country_resolved
        .as_ref()
        .and_then(|v| v.get("name").and_then(|n| n.as_str()))
        .unwrap_or(&project.country);
```

Then change the format line from:
```rust
        format!("country = '{}'", project.country.replace("'", "''")),
```
To:
```rust
        format!("country = '{}'", country_name.replace("'", "''")),
```

- [ ] **Step 4: Normalize country in `create_company`**

At `client.rs:333`, same pattern:

```rust
    // Resolve country to canonical name
    let country_resolved = self.resolve_country(&company.country).await?;
    let country_name = country_resolved
        .as_ref()
        .and_then(|v| v.get("name").and_then(|n| n.as_str()))
        .unwrap_or(&company.country);
```

Then change the format from:
```rust
        company.country.replace("'", "''"),
```
To:
```rust
        country_name.replace("'", "''"),
```

- [ ] **Step 5: Forward resolve_country through DatabaseManager**

If `DatabaseManager` wraps `DatabaseClient`, add a forwarding method in `src-tauri/src/db/mod.rs`:

```rust
pub async fn resolve_country(&self, input: &str) -> Result<Option<serde_json::Value>, Error> {
    self.client.resolve_country(input).await
}
```

This is needed for the agent_server (Task 3) which accesses DB through `DatabaseManager`.

- [ ] **Step 6: Build and verify**

```bash
cd /Volumes/base/dev/app/e-fees && cargo build -p app 2>&1 | tail -10
```

Expected: Compiles cleanly.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/db/client.rs src-tauri/src/db/operations.rs src-tauri/src/db/mod.rs
git commit -m "feat(tauri): normalize country via fn::resolve_country

Adds resolve_country() to DatabaseClient. Updates generate_next_project_number,
create_new_project, and create_company to normalize country before storage.
Old exact-match lookup commented out.

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

### Task 5: Integration Test

**Files:**
- No new files — tests run against production DB via API

- [ ] **Step 1: Test via e-fees-api**

Create a test company with ISO 2-letter code to verify end-to-end:

```bash
# Create company with "AE" — should store as "U.A.E."
curl -s -X POST "http://10.0.21.80:3200/api/companies" \
  -H "X-API-Key: efees-api-2026-k8x9m4pq" \
  -H "Content-Type: application/json" \
  -d '{"name":"DELETE ME - Country Test Co","name_short":"DELME","abbreviation":"DELME","city":"Dubai","country":"AE"}' | python3 -m json.tool

# Verify country was normalized
# Check the response — country field should be "U.A.E." not "AE"
```

- [ ] **Step 2: Test with various formats**

```bash
# Test with "SA" (iso2)
curl -s -X POST "http://10.0.21.80:3200/api/companies" \
  -H "X-API-Key: efees-api-2026-k8x9m4pq" \
  -H "Content-Type: application/json" \
  -d '{"name":"DELETE ME - Country Test SA","name_short":"DELSA","abbreviation":"DELSA","city":"Riyadh","country":"SA"}' | python3 -c "import sys,json; print(json.load(sys.stdin).get('country','MISSING'))"

# Should print: K.S.A.
```

- [ ] **Step 3: Test invalid country**

```bash
curl -s -X POST "http://10.0.21.80:3200/api/companies" \
  -H "X-API-Key: efees-api-2026-k8x9m4pq" \
  -H "Content-Type: application/json" \
  -d '{"name":"DELETE ME - Bad Country","name_short":"DELBAD","abbreviation":"DELBAD","city":"Nowhere","country":"XYZZY"}'

# Should return 200 with country="XYZZY" (fallback to raw — not a hard error for companies)
# OR return 400 depending on design choice. Current design uses fallback.
```

- [ ] **Step 4: Clean up test data**

```bash
curl -s -X POST "http://10.0.23.11:8000/rpc" \
  -u "martin:<redacted-rotated-2026-05-06>" \
  -H "surreal-ns: emittiv" \
  -H "surreal-db: projects" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -d '{"method":"query","params":["DELETE company WHERE name CONTAINS \"DELETE ME\""]}'
```

- [ ] **Step 5: Commit (if any test revealed fixes)**

---

### Task 6: Reply to PA and Update Docs

- [ ] **Step 1: Reply to PA hub message**

Use `kb_hub_reply` to respond to `message:sy7kad3iopgvgcisvbrq`:

Content:
```
Country normalization deployed. fn::resolve_country() accepts any of:
- ISO 2-letter codes: AE, SA, GB, US, IN, FR, etc.
- 3-letter codes: UAE, KSA, GBR, etc.
- Display names: U.A.E., K.S.A., UK, India
- Official names: United Arab Emirates, Saudi Arabia

Returns canonical name + dial_code. All API create handlers (e-fees-api, agent server, Tauri) now normalize before storage.

You can drop the COUNTRY_MAP dict in rfp_intake.py and send standard ISO 2-letter codes directly.

Test: POST /api/companies with "country": "AE" → stored as "U.A.E."
```

- [ ] **Step 2: Update CLAUDE.md**

Add to the "Recent Fixes" section:
```
- **Country normalization** ✅ - fn::resolve_country DB function, iso2 field, all API consumers normalized
```

- [ ] **Step 3: Save KB observation**

```javascript
kb_observe({
    entity_name: "Country Normalization",
    content: "fn::resolve_country() DB stored function deployed (2026-03-27). 7-step cascade: code→iso2→code_alt→name→dot-strip→official→formal. New iso2 field on country table with ISO 3166-1 alpha-2 codes. All API consumers (e-fees-api, agent_server, Tauri app) normalize country before storage. PA can send standard 2-letter ISO codes.",
    entity_type: "decision",
    confidence: 1.0,
    source: "implementation"
})
```
