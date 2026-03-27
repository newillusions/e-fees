# Country Normalization — Design Spec

**Date**: 2026-03-27
**Motivation**: PA auto-creates e-fees projects/companies from RFP emails, sending ISO 2-letter country codes ("AE", "SA"). The country table only has 3-letter codes and display names. No normalization exists — API consumers must guess the exact display name or records get inconsistent country values.
**Approach**: SurrealDB stored function `fn::resolve_country` + new `iso2` field on country table.

---

## Problem

1. **Country table lacks 2-letter ISO codes** — `code` is 3-letter IOC ("UAE", "KSA"), `code_alt` is 3-letter ISO 3166-1 alpha-3 ("ARE", "SAU"). PA and standard APIs use 2-letter ISO 3166-1 alpha-2 ("AE", "SA").
2. **No normalization** — country is stored as a free-text string. Production data uses abbreviated display names ("U.A.E.", "K.S.A.") that don't match any code field.
3. **Fragile dial code lookup** — `generate_next_project_number` does exact-match `WHERE name = $name`. If the string doesn't match `country.name` exactly, project number generation fails.
4. **Multiple consumers** — Tauri app, standalone API (`e-fees-api`), agent server, and PA all create records. Each would need its own normalization logic without a shared solution.

## Solution

### 1. Add `iso2` field to country table

```sql
DEFINE FIELD iso2 ON country TYPE string;
DEFINE INDEX idx_country_iso2 ON country FIELDS iso2;
```

Populate from ISO 3166-1 alpha-2 standard. ~250 UPDATE statements.

Key mappings:
| iso2 | code (3-letter) | name | dial_code |
|------|-----------------|------|-----------|
| AE | UAE | U.A.E. | 971 |
| SA | KSA | K.S.A. | 966 |
| GB | GBR | UK | 44 |
| US | USA | U.S.A. | 1 |
| IN | IND | India | 91 |
| FR | FRA | France | 33 |
| KW | KUW | Kuwait | 965 |

### 2. Stored function: `fn::resolve_country`

```sql
DEFINE FUNCTION fn::resolve_country($input: string) {
    -- Returns: { name, code, iso2, dial_code } or NONE
    -- Match cascade (first match wins):

    -- 1. code exact (3-letter: "UAE", "KSA")
    -- 2. iso2 exact (2-letter: "AE", "SA")
    -- 3. code_alt exact (alt 3-letter: "ARE", "SAU")
    -- 4. name exact ("U.A.E.", "K.S.A.", "India")
    -- 5. name dot-stripped ("UAE" matches "U.A.E." after removing dots)
    -- 6. name_official / name_formal case-insensitive contains
    --    ("Saudi Arabia", "United Arab Emirates")
    -- 7. dial_code if input is numeric string ("971" → UAE)
    -- 8. NONE if no match
};
```

**Return shape**: `{ name: string, code: string, iso2: string, dial_code: int }`
- `name` = canonical display name for storage ("U.A.E.", "K.S.A.")
- `code` = 3-letter code for record ID reference
- `iso2` = 2-letter ISO code for API interop
- `dial_code` = for project number generation

### 3. Consumer changes

**Standalone API (`e-fees-api`)**:
- `create_project`: Replace `lookup_dial_code` with `fn::resolve_country`. Use returned `name` for storage, `dial_code` for project number.
- `create_company`: Add `fn::resolve_country` call before storage.
- Comment out old `lookup_dial_code` function (preserve for revert).

**Agent server (`agent_server.rs`)**:
- `create_project_handler`: Add `fn::resolve_country` call. Currently caller must supply pre-formatted project number — normalization here ensures the country field is consistent even when number is provided externally.
- `create_company_handler`: Add `fn::resolve_country` call.
- Comment out any existing country handling (preserve for revert).

**Tauri desktop app (`client.rs`)**:
- `generate_next_project_number`: Replace exact-match `WHERE name = $name` with `fn::resolve_country` call for dial code lookup.
- `create_new_project` / `create_company`: Use resolved `name` for storage.
- Comment out old logic (preserve for revert).

### 4. Error handling

When `fn::resolve_country` returns `NONE`:
- API endpoints return 400 with message: `"Unknown country: '{input}'. Provide an ISO code (AE, SA), name (UAE, India), or dial code (971)."`
- Tauri commands return `Err("Unknown country: ...")` — frontend displays in toast.

### 5. What is NOT changing

- **Country table structure** — only adding `iso2` field, not modifying existing fields
- **Frontend typeahead** — already returns `country.name`, unaffected
- **Existing production data** — already uses canonical `name` values ("U.A.E.", "K.S.A.")
- **Project number format** — YY-CCCNN unchanged
- **Record IDs** — `country:UAE` format unchanged

## Migration

1. Add `iso2` field + index
2. Populate `iso2` for all ~250 countries
3. Define `fn::resolve_country`
4. Test function with representative inputs
5. Update API consumers

All migration SQL in a single `004-country-normalization.surql` file.

## Test Cases

| Input | Expected match | Via |
|-------|---------------|-----|
| "UAE" | U.A.E. (971) | code exact |
| "AE" | U.A.E. (971) | iso2 exact |
| "ARE" | U.A.E. (971) | code_alt exact |
| "U.A.E." | U.A.E. (971) | name exact |
| "United Arab Emirates" | U.A.E. (971) | name_official contains |
| "KSA" | K.S.A. (966) | code exact |
| "SA" | K.S.A. (966) | iso2 exact |
| "Saudi Arabia" | K.S.A. (966) | name_official contains |
| "971" | U.A.E. (971) | dial_code |
| "India" | India (91) | name exact |
| "GB" | UK (44) | iso2 exact |
| "uk" | UK (44) | name dot-stripped (case-insensitive) |
| "XYZZY" | NONE | no match |
