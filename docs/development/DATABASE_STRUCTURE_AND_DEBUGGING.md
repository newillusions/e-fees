# Database Structure and Debugging Guide

## Summary of Issues Found

**Date**: July 19, 2025  
**Issue**: Contact creation failing and contacts page showing 0 records despite database containing 25+ contacts

### Root Causes Identified

1. **Schema Conflict**: `full_name` field is auto-computed but backend was trying to set it manually
2. **Deserialization Issue**: Rust SurrealDB client can't deserialize malformed contact records
3. **Data Integrity**: Existing contacts in database only have `id` and `time` fields - missing actual data

---

## Complete Database Schema Analysis

### Contacts Table Schema

```sql
DEFINE TABLE contacts TYPE NORMAL SCHEMAFULL PERMISSIONS FULL;

-- Field Definitions
DEFINE FIELD company ON contacts TYPE record<company> PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD email ON contacts TYPE string ASSERT string::is::email($value) PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD first_name ON contacts TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD full_name ON contacts TYPE string VALUE string::concat(first_name, ' ', last_name) PERMISSIONS FOR select WHERE FULL, FOR create, update FULL;
DEFINE FIELD last_name ON contacts TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD phone ON contacts TYPE string ASSERT string::len($value) > 0 AND string::contains($value, '+') PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD position ON contacts TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD time ON contacts TYPE object PERMISSIONS FULL;
DEFINE FIELD time.created_at ON contacts TYPE datetime DEFAULT time::now() VALUE $before OR time::now() PERMISSIONS FULL;
DEFINE FIELD time.updated_at ON contacts TYPE datetime DEFAULT time::now() VALUE time::now() PERMISSIONS FULL;

-- Indexes
DEFINE INDEX contact_company ON contacts FIELDS company;
DEFINE INDEX contact_created ON contacts FIELDS time.created_at;
DEFINE INDEX contact_last_name ON contacts FIELDS last_name;
```

### Critical Schema Rules

1. **Required Fields**: `first_name`, `last_name`, `email`, `company`
2. **Computed Field**: `full_name` is AUTO-GENERATED - NEVER set manually
3. **Phone Validation**: Must contain `+` character and have length > 0
4. **Email Validation**: Must pass `string::is::email()` assertion
5. **Company Field**: Must be a valid `record<company>` reference
6. **Time Fields**: Auto-managed by database

### Working Contact Record Example

```json
{
  "id": "contacts:3ffcc3qc2sld8zifysvq",
  "company": "company:SAN",
  "email": "meenakshi.chauhan@sandersonuae.com",
  "first_name": "Meenakshi",
  "full_name": "Meenakshi Chauhan",
  "last_name": "Chauhan",
  "phone": "+971 58 842 9529",
  "position": "Design manager",
  "time": {
    "created_at": "2025-06-15T14:08:44.075324551Z",
    "updated_at": "2025-06-15T18:26:05.796251608Z"
  }
}
```

---

## Company Table Schema

```sql
DEFINE TABLE company TYPE NORMAL SCHEMAFULL PERMISSIONS FULL;

DEFINE FIELD abbreviation ON company TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD city ON company TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD country ON company TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD name ON company TYPE string ASSERT string::len($value) > 0 PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD name_short ON company TYPE string PERMISSIONS FOR select, create, update WHERE FULL;
DEFINE FIELD reg_no ON company TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD tax_no ON company TYPE option<string> PERMISSIONS FULL;
DEFINE FIELD time ON company TYPE object PERMISSIONS FULL;
```

### Available Companies

- `company:IMTZ` - Imtiaz Developments (recently created, incomplete data)
- `company:CHE` - Conrad Hilton Etihad Towers
- `company:SAN` - Sanderson International Theme LLC
- `company:EMT` - emittiv LLC-FZ
- Plus 17 other companies

---

## SurrealDB Rust SDK Best Practices

### Correct Contact Creation Pattern

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ContactCreate {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub position: String,
    pub company: String,  // Company ID as string
    // Note: Do NOT include full_name - it's auto-computed
}

#[derive(Debug, Serialize, Deserialize)]
struct Contact {
    pub id: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,  // Will be populated by database
    pub email: String,
    pub phone: String,
    pub position: String,
    pub company: String,
    pub time: TimeStamp,
}

// Correct creation method
async fn create_contact(db: &Database, contact: ContactCreate) -> Result<Contact, Error> {
    let created: Option<Contact> = db
        .create("contacts")
        .content(contact)
        .await?;
    
    Ok(created.ok_or("Failed to create contact")?)
}
```

### Wrong Patterns to Avoid

❌ **DO NOT** manually set `full_name`:
```rust
// WRONG - This will cause "Found NONE for field full_name" error
let query = format!(
    "CREATE contacts:{} SET first_name = '{}', last_name = '{}', full_name = '{}'...",
    id, first_name, last_name, full_name  // This breaks schema
);
```

❌ **DO NOT** use raw string queries for complex operations:
```rust
// WRONG - Use .create().content() instead
db.query("CREATE contacts:xyz SET first_name = 'John'...").await
```

✅ **DO** use structured creation:
```rust
// CORRECT - Let database compute full_name
let contact = ContactCreate {
    first_name: "Irene".to_string(),
    last_name: "B Nale".to_string(),
    email: "i.nale@imtiaz.ae".to_string(),
    phone: "+971 4 430 3703".to_string(),
    position: "Design Administration Coordinator".to_string(),
    company: "IMTZ".to_string(),
};

let created: Option<Contact> = db.create("contacts").content(contact).await?;
```

---

## Required Fixes Applied

### 1. Backend Fix (COMPLETED)

**File**: `/src-tauri/src/db/mod.rs`  
**Issue**: Manual `full_name` setting in CREATE query  
**Fix**: Removed `full_name` from query, let database auto-compute

```rust
// OLD (BROKEN)
let query = format!(
    "CREATE contacts:{} SET first_name = '{}', last_name = '{}', full_name = '{}', ...",
    contact_id, first_name, last_name, full_name  // WRONG
);

// NEW (FIXED)
let query = format!(
    "CREATE contacts:{} SET first_name = '{}', last_name = '{}', email = '{}', ...",
    contact_id, first_name, last_name, email  // full_name omitted
);
```

### 2. Recommended: Use SDK Instead of Raw Queries

Replace raw string queries with proper SurrealDB Rust SDK methods:

```rust
// Instead of manual query building, use:
let created: Option<Contact> = self.client
    .create("contacts")
    .content(contact_data)
    .await?;
```

---

## Test Data Creation

### Test Contact (Safe to Delete)

Use this pattern for creating test contacts:

```json
{
  "first_name": "DELETE ME Test",
  "last_name": "Contact",
  "email": "delete.me@test.example.com",
  "phone": "+971 50 000 0000",
  "position": "Test Position - DELETE ME",
  "company": "IMTZ"
}
```

### Test Companies

```json
{
  "name": "DELETE ME Test Company",
  "name_short": "DELETE ME",
  "abbreviation": "TEST",
  "city": "Test City",
  "country": "Test Country"
}
```

---

## Current Status

### Issues Resolved ✅
- Backend contact creation query fixed
- Schema understanding documented
- SurrealDB SDK patterns identified

### Issues Remaining ❌
1. **Contact Retrieval**: App still shows 0 contacts due to malformed existing data
2. **Data Cleanup**: Need to clean/recreate malformed contact records
3. **UI Testing**: Need to verify contact modal works end-to-end

### Next Steps
1. Test contact creation with fixed backend
2. Clean up malformed contact records in database
3. Verify contact listing works with properly formatted records
4. Document working test procedures

---

## Debugging Commands

### Verify Contact Structure
```bash
# Count contacts
curl -X POST "http://10.0.1.17:8000/sql" -H "Content-Type: text/plain" -u "martin:th38ret3ch" -d "SELECT count() FROM contacts GROUP ALL;"

# Check contact fields
curl -X POST "http://10.0.1.17:8000/sql" -H "Content-Type: text/plain" -u "martin:th38ret3ch" -d "SELECT id, first_name, last_name, email FROM contacts LIMIT 3;"
```

### Test Contact Creation
```python
# Use Python WebSocket client to test creation
python3 -c "
import asyncio, websockets, json
async def test():
    uri = 'ws://10.0.1.17:8000/rpc'
    async with websockets.connect(uri) as ws:
        await ws.send(json.dumps({'id': 1, 'method': 'use', 'params': ['emittiv', 'projects']}))
        await ws.recv()
        
        create_sql = '''CREATE contacts:DELETE_ME_TEST SET 
            first_name = 'DELETE ME',
            last_name = 'Test Contact',
            email = 'delete.me@test.com',
            phone = '+971 50 000 0000',
            position = 'DELETE ME Position',
            company = company:IMTZ;'''
        
        await ws.send(json.dumps({'id': 2, 'method': 'query', 'params': [create_sql]}))
        response = await ws.recv()
        print('Result:', response)
asyncio.run(test())
"
```

---

**Last Updated**: July 19, 2025  
**Status**: Backend fixed, testing required  
**Author**: Claude Code Assistant