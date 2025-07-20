# RFP Implementation Notes & Troubleshooting Guide

**Date**: July 20, 2025  
**Session**: RFP CRUD Implementation and SurrealDB Thing Format Resolution

## Overview
This document captures the critical learnings from implementing RFP (Request for Proposal) creation functionality and resolving persistent SurrealDB Thing format errors that took hours to debug.

## Major Issues Resolved

### 1. SurrealDB Thing Format Error
**Error**: `invalid type: string "projects:25_97107", expected struct Thing`

**Root Cause**: Attempting to pass SurrealDB Thing objects directly from Rust structs to SurrealDB, causing serialization conflicts.

**Solution**: Use raw SQL queries like contacts module instead of struct-based creation.

#### Before (Failed Approach):
```rust
// This FAILED - trying to use Thing structs directly
pub async fn create_rfp(&self, rfp: Rfp) -> Result<Option<Rfp>, Error> {
    match self {
        DatabaseClient::Http(client) => client.create("rfp").content(rfp).await,
        DatabaseClient::WebSocket(client) => client.create("rfp").content(rfp).await,
    }
}
```

#### After (Working Solution):
```rust
// This WORKS - using raw SQL like contacts
pub async fn create_rfp(&self, rfp: RfpCreate) -> Result<Option<Rfp>, Error> {
    let rfp_id = format!("{}_{}", rfp.project_id.replace("-", "_"), rfp.rev);
    
    let query = format!(
        "CREATE rfp:{} SET name = '{}', number = '{}', rev = {}, project_id = projects:{}, company_id = company:{}, contact_id = contacts:{}, status = '{}', stage = '{}', issue_date = '{}', activity = '{}', package = '{}', strap_line = '{}', staff_name = '{}', staff_email = '{}', staff_phone = '{}', staff_position = '{}', revisions = [], time = {{ created_at: time::now(), updated_at: time::now() }}",
        rfp_id,
        rfp.name.replace("'", "''"),
        rfp.number.replace("'", "''"),
        rfp.rev,
        rfp.project_id.replace("'", "''"),
        rfp.company_id.replace("'", "''"), 
        rfp.contact_id.replace("'", "''"),
        // ... other fields
    );
    
    let mut response = match self {
        DatabaseClient::Http(client) => client.query(&query).await?,
        DatabaseClient::WebSocket(client) => client.query(&query).await?,
    };
    
    let result: Result<Vec<Rfp>, _> = response.take(0);
    match result {
        Ok(mut rfps) => Ok(rfps.pop()),
        Err(e) => Err(e),
    }
}
```

### 2. RFP ID Format Issues
**Problem**: RFP IDs were being generated without proper underscore formatting.

**Solution**: 
- **Format**: `project_number_revision` (e.g., `"25_97107_1"`)
- **Implementation**: `let rfp_id = format!("{}_{}", rfp.project_id.replace("-", "_"), rfp.rev);`

### 3. Incorrect Company and Contact References
**Problem**: Test functions used hardcoded incorrect IDs (`"EMITTIV"`, `"contacts:1"`).

**Solution**: Connected directly to SurrealDB to find actual IDs:
- **EMITTIV Company**: `company:EMT` (ID: "EMT")
- **Martin Robert Contact**: `contacts:hqpz6h9z1v5w6uj46tl2` (ID: "hqpz6h9z1v5w6uj46tl2")

### 4. Timestamp Handling Issues
**Error**: `Found '' for field time.created_at, but expected a datetime`

**Solution**: Let SurrealDB auto-generate timestamps using `time::now()` instead of manual datetime strings.

#### Working Pattern:
```sql
CREATE rfp:25_97107_1 SET 
  name = 'Fee Proposal',
  -- ... other fields
  time = { created_at: time::now(), updated_at: time::now() }
```

### 5. Frontend/Backend ID Mismatch
**Problem**: Frontend was adding table prefixes (`projects:`, `company:`) but backend expected raw IDs.

**Solution**: 
- **Frontend**: Send raw IDs (`"25_97107"`, `"EMT"`)  
- **Backend**: Add prefixes in SQL generation (`projects:25_97107`, `company:EMT`)

#### Frontend Fix:
```typescript
// Before (caused double prefixes)
project_id: formData.project_id ? `projects:${formData.project_id}` : null,

// After (send raw IDs)
project_id: formData.project_id || null,
```

## Critical Implementation Pattern

### The Working RFP Creation Flow:

1. **RfpCreate Struct** (simple string IDs):
```rust
pub struct RfpCreate {
    pub name: String,
    pub project_id: String,  // Raw ID: "25_97107"
    pub company_id: String,  // Raw ID: "EMT" 
    pub contact_id: String,  // Raw ID: "hqpz6h9z1v5w6uj46tl2"
    // ... other fields
}
```

2. **Command Layer** (accepts RfpCreate):
```rust
#[tauri::command]
pub async fn create_rfp(rfp: RfpCreate, state: State<'_, AppState>) -> Result<Rfp, String>
```

3. **Database Layer** (converts to SQL):
```rust
pub async fn create_rfp(&self, rfp: RfpCreate) -> Result<Option<Rfp>, Error>
// Generates: CREATE rfp:25_97107_1 SET ... project_id = projects:25_97107, company_id = company:EMT
```

4. **API Layer** (handles ID conversion):
```typescript
const rfpCreate = {
  // Strip any existing prefixes and ensure clean IDs
  project_id: typeof rfp.project_id === 'string' ? rfp.project_id.replace('projects:', '') : rfp.project_id?.toString() || '',
  company_id: typeof rfp.company_id === 'string' ? rfp.company_id.replace('company:', '') : rfp.company_id?.toString() || '',
  contact_id: typeof rfp.contact_id === 'string' ? rfp.contact_id.replace('contacts:', '') : rfp.contact_id?.toString() || '',
};
```

## Database Connection for ID Lookup

When you need to find actual database IDs, use this Node.js script pattern:

```javascript
import { Surreal } from 'surrealdb';

const db = new Surreal();
await db.connect('http://10.0.1.17:8000/rpc');
await db.signin({ username: 'martin', password: 'th38ret3ch' });
await db.use({ namespace: 'emittiv', database: 'projects' });

const companies = await db.select('company');
const contacts = await db.select('contacts');
```

## Key Learnings

### ✅ DO:
- Follow the **contacts pattern** for new entity creation
- Use **RfpCreate/ContactCreate** structs with string IDs for creation
- Use **raw SQL queries** for database operations  
- Let **SurrealDB auto-generate timestamps**
- Send **raw IDs** from frontend, add prefixes in backend
- Test with **actual database IDs**, not hardcoded guesses

### ❌ DON'T:
- Try to use Thing structs directly in .content() calls
- Manually construct datetime strings
- Add table prefixes in frontend if backend adds them
- Use hardcoded test IDs without verifying they exist
- Assume CRUD patterns work the same across different SurrealDB client versions

## Testing Commands

### Test RFP Creation (Rust):
```rust
let test_rfp = RfpCreate {
    name: "DELETE ME - Test RFP".to_string(),
    project_id: "25_97107".to_string(),
    company_id: "EMT".to_string(),  // Verified: emittiv LLC-FZ
    contact_id: "hqpz6h9z1v5w6uj46tl2".to_string(),  // Verified: Martin Robert
    // ... other fields
};
```

### Test RFP Creation (Frontend):
```typescript
const testRfpData = {
  name: "DELETE ME - Test RFP",
  project_id: 'projects:25_97107',  // Include prefix for frontend
  company_id: 'company:EMT',        // Verified company
  contact_id: 'contacts:hqpz6h9z1v5w6uj46tl2',  // Verified contact
};
```

## Files Modified

### Backend Changes:
- `src-tauri/src/commands/mod.rs` - Updated create_rfp command signature
- `src-tauri/src/db/mod.rs` - Implemented raw SQL approach 
- Added RfpCreate struct with string IDs

### Frontend Changes:
- `src/lib/api.ts` - Fixed ID conversion logic
- `src/lib/components/RfpModal.svelte` - Removed double prefixing
- `src/lib/components/ConnectionStatus.svelte` - Updated test data with correct IDs

## Database Schema Notes

### Verified Company IDs:
- emittiv LLC-FZ: `company:EMT` (abbreviation: "EMT")
- Not: `company:EMITTIV` (this doesn't exist)

### Verified Contact IDs:
- Martin Robert: `contacts:hqpz6h9z1v5w6uj46tl2`
- Email: martin@emittiv.com

### RFP ID Pattern:
- Format: `rfp:{project_number}_{revision}`
- Example: `rfp:25_97107_1` for project 25-97107, revision 1

## Error Prevention Checklist

Before implementing similar CRUD operations:

1. ✅ Check how **contacts** module handles the same operation
2. ✅ Use **{EntityName}Create** structs for creation (not full entities)  
3. ✅ Test with **real database IDs**, not assumed ones
4. ✅ Use **raw SQL approach** for complex operations
5. ✅ Let **SurrealDB handle timestamps** automatically
6. ✅ Keep **frontend ID handling** simple (raw IDs only)
7. ✅ Always **test end-to-end** before considering complete

---

**Remember**: When in doubt, follow the contacts pattern. It works.