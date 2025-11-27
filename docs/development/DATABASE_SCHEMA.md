# Fee Proposal App - Database Schema Documentation

## Database Connection Details

**SurrealDB Instance**: `ws://10.0.1.17:8000`  
**Namespace**: `emittiv`  
**Database**: `projects`  
**Authentication**: User-based (martin/password via environment variables)

## Database Overview

The database contains **6 main tables** with **27 performance indexes** across all tables. All tables use auto-managed timestamps and have proper foreign key relationships.

---

## Table Structures

### 1. `projects` Table

**Purpose**: Project opportunities emittiv is bidding on  
**ID Format**: `projects:⟨YY_CCCNN⟩` (auto-generated)  
**Type**: SCHEMAFULL

```sql
DEFINE TABLE projects SCHEMAFULL;
DEFINE FIELD name ON projects TYPE string ASSERT $value != NONE AND string::len($value) > 0;
DEFINE FIELD name_short ON projects TYPE string ASSERT $value != NONE;
DEFINE FIELD status ON projects TYPE string ASSERT $value INSIDE ['Draft', 'RFP', 'Active', 'On Hold', 'Completed', 'Cancelled'] DEFAULT 'Draft';
DEFINE FIELD area ON projects TYPE string ASSERT $value != NONE;
DEFINE FIELD city ON projects TYPE string ASSERT $value != NONE;
DEFINE FIELD country ON projects TYPE string ASSERT $value != NONE;
DEFINE FIELD folder ON projects TYPE string ASSERT $value != NONE;
DEFINE FIELD number ON projects TYPE object ASSERT $value != NONE;
DEFINE FIELD number.year ON projects TYPE int ASSERT $value >= 20 AND $value <= 50;
DEFINE FIELD number.country ON projects TYPE int ASSERT $value != NONE;
DEFINE FIELD number.seq ON projects TYPE int ASSERT $value >= 1 AND $value <= 999;
DEFINE FIELD number.id ON projects TYPE string ASSERT $value != NONE;
DEFINE FIELD time ON projects TYPE object VALUE { created_at: time::now(), updated_at: time::now() };
DEFINE FIELD time.created_at ON projects TYPE datetime VALUE time::now();
DEFINE FIELD time.updated_at ON projects TYPE datetime VALUE time::now();

-- Unique constraint on project number
DEFINE INDEX project_number_unique ON projects FIELDS number.id UNIQUE;
```

**Business Logic - Project Numbering**:
- Format: `YY-CCCNN` (e.g., `25-97105`)
- YY: 2-digit year (25 = 2025)
- CCC: Country dial code (971=UAE, 966=Saudi)  
- NN: Sequence number (05 = 5th project)
- **Critical**: Sequence stored as integer but represents 2-digit padded values
- UAE projects start from sequence 1 (display as 97101)
- Saudi projects start from sequence 1 (display as 96601)
- Sequence resets annually per country

**Sample Project Record**:
```json
{
  "id": "projects:⟨25_97105⟩",
  "name": "Shanghai Tang Revision",
  "name_short": "Shanghai Tang v2", 
  "status": "Active",
  "area": "Etihad Towers",
  "city": "Abu Dhabi",
  "country": "U.A.E.",
  "folder": "25-97105 Shanghai Tang v2",
  "number": {
    "year": 25,
    "country": 971,
    "seq": 5,
    "id": "25-97105"
  },
  "time": {
    "created_at": "2025-06-14T17:52:39.604Z",
    "updated_at": "2025-06-15T11:09:14.587Z"
  }
}
```

---

### 2. `fee` Table

**Purpose**: Fee proposals created by emittiv staff  
**ID Format**: `fee:⟨YY_CCCNN_R⟩` (auto-generated)  
**Type**: SCHEMAFULL

```sql
DEFINE TABLE fee SCHEMAFULL;
DEFINE FIELD name ON fee TYPE string ASSERT $value != NONE AND string::len($value) > 0 DEFAULT 'Fee Proposal';
DEFINE FIELD number ON fee TYPE string ASSERT $value != NONE;
DEFINE FIELD project_id ON fee TYPE record<projects> ASSERT $value != NONE;
DEFINE FIELD company_id ON fee TYPE record<company> ASSERT $value != NONE;
DEFINE FIELD contact_id ON fee TYPE record<contacts> ASSERT $value != NONE;
DEFINE FIELD status ON fee TYPE string ASSERT $value INSIDE ['Draft', 'Sent', 'Negotiation', 'Awarded', 'Completed', 'Lost', 'Cancelled', 'On Hold', 'Revised'] DEFAULT 'Draft';
DEFINE FIELD issue_date ON fee TYPE string ASSERT $value != NONE AND string::len($value) = 6;
DEFINE FIELD activity ON fee TYPE option<string>;
DEFINE FIELD package ON fee TYPE option<string>;
DEFINE FIELD strap_line ON fee TYPE option<string> DEFAULT 'sensory design studio';
DEFINE FIELD staff_name ON fee TYPE option<string>;
DEFINE FIELD staff_email ON fee TYPE option<string>;
DEFINE FIELD staff_phone ON fee TYPE option<string>;
DEFINE FIELD staff_position ON fee TYPE option<string>;
DEFINE FIELD rev ON fee TYPE int DEFAULT 1 VALUE math::max($value.revisions[*].revision_number);
DEFINE FIELD revisions ON fee TYPE array<object> DEFAULT [];
DEFINE FIELD time ON fee TYPE object VALUE { created_at: time::now(), updated_at: time::now() };

-- Unique constraint on project + revision
DEFINE INDEX fee_project_rev ON fee FIELDS project_id, rev UNIQUE;
```

**Revision Management**:
- `rev` field auto-computed from revisions array
- Each revision has: revision_number, revision_date, author_email, author_name, notes
- Complete audit trail for all proposal changes

---

### 3. `company` Table

**Purpose**: Client companies that issue project opportunities  
**ID Format**: `company:ABBREVIATION` (e.g., `company:CHE`)  
**Type**: SCHEMAFULL

```sql
DEFINE TABLE company SCHEMAFULL;
DEFINE FIELD name ON company TYPE string ASSERT $value != NONE AND string::len($value) > 0;
DEFINE FIELD name_short ON company TYPE string ASSERT $value != NONE;
DEFINE FIELD abbreviation ON company TYPE string ASSERT $value != NONE;
DEFINE FIELD city ON company TYPE string ASSERT $value != NONE;
DEFINE FIELD country ON company TYPE string ASSERT $value != NONE;
DEFINE FIELD reg_no ON company TYPE option<string>;
DEFINE FIELD tax_no ON company TYPE option<string>;
DEFINE FIELD time ON company TYPE object VALUE { created_at: time::now(), updated_at: time::now() };
```

**Sample Company Record**:
```json
{
  "id": "company:CHE",
  "name": "Conrad Hilton",
  "name_short": "Conrad Etihad", 
  "abbreviation": "CHE",
  "city": "Abu Dhabi",
  "country": "U.A.E.",
  "time": {
    "created_at": "2025-06-14T17:51:01.928Z",
    "updated_at": "2025-06-14T17:51:01.928Z"
  }
}
```

---

### 4. `contacts` Table

**Purpose**: Individual contact persons at client companies  
**ID Format**: `contacts:random_id` (auto-generated)  
**Type**: SCHEMAFULL

```sql
DEFINE TABLE contacts SCHEMAFULL;
DEFINE FIELD first_name ON contacts TYPE string ASSERT $value != NONE;
DEFINE FIELD last_name ON contacts TYPE string ASSERT $value != NONE;
DEFINE FIELD email ON contacts TYPE string ASSERT is::email($value);
DEFINE FIELD phone ON contacts TYPE string ASSERT string::contains($value, '+') AND string::len($value) > 0;
DEFINE FIELD position ON contacts TYPE string ASSERT $value != NONE;
DEFINE FIELD company ON contacts TYPE record<company> ASSERT $value != NONE;
DEFINE FIELD full_name ON contacts TYPE string VALUE string::concat($value.first_name, ' ', $value.last_name);
DEFINE FIELD time ON contacts TYPE object VALUE { created_at: time::now(), updated_at: time::now() };

-- Unique email constraint
DEFINE INDEX contact_email_unique ON contacts FIELDS email UNIQUE;
```

---

### 5. `country` Table (Reference Data)

**Purpose**: Pre-populated reference table for countries  
**Type**: SCHEMAFULL, read-only reference data

Contains ~250 countries with fields:
- `name`, `name_formal`, `name_official`
- `code`, `code_alt`, `dial_code` 
- `currency_code`

**Key Countries for Project Numbering**:
- `country:AE` - dial_code: 971 (UAE)
- `country:SA` - dial_code: 966 (Saudi Arabia)

---

### 6. `currency` Table (Reference Data)

**Purpose**: Pre-populated reference table for currencies  
**Type**: SCHEMAFULL, read-only reference data

Contains ~180 currencies with fields:
- `code` (USD, AED, SAR, etc.)
- `name` (full currency name)

---

## Critical Business Rules

### Project Number Generation Algorithm

```typescript
// Generate next project number
function generateNextProjectNumber(countryDialCode: number, year?: number): string {
  const currentYear = year || (new Date().getFullYear() % 100);
  
  // Query database for highest sequence in this year/country
  const query = `SELECT math::max(number.seq) as max_seq FROM projects 
                 WHERE number.year = ${currentYear} AND number.country = ${countryDialCode}`;
  
  const result = await db.query(query);
  const nextSeq = (result.max_seq || 0) + 1;
  
  // Format: YY-CCCNN
  const projectNumber = `${currentYear.toString().padStart(2, '0')}-${countryDialCode}${nextSeq.toString().padStart(2, '0')}`;
  
  return projectNumber; // e.g., "25-97105"
}
```

### Validation Rules

1. **Project Numbers**: Must be unique across all projects
2. **Email Addresses**: Must be unique across all contacts
3. **Phone Numbers**: Must contain '+' character
4. **Issue Dates**: Must be exactly 6 digits (YYMMDD format)
5. **Sequence Numbers**: 1-999 range, but display as 2-digit padded

### RFP Status Workflow

```
Draft → Active → Sent → Under Review → Negotiation → Awarded/Lost
      ↓
   Cancelled (can happen at any stage)
      ↓  
   Revised (when project restarted with new approach)
```

---

## Database Relationships

```
projects ←──── rfp ────→ company
                ↓
             contacts ────→ company

country ────→ projects (via country name)
currency ────→ rfp (for pricing)
```

---

## Current Data Status (June 2025)

**Production Database Contains**:
- **48 Projects**: Mix of Active, Completed, Cancelled, RFP status
- **37 RFPs**: Various stages from Draft to Awarded
- **19 Companies**: UAE, Saudi, international clients
- **Multiple Contacts**: Linked to their respective companies

**Example Project Numbers in Use**:
- UAE: 22-97113, 24-97101, 25-97105 (sequences: 13, 1, 5)
- Saudi: 22-96601, 24-96606, 25-96601 (sequences: 1, 6, 1)

---

## Integration Notes for Claude Code

### Required Database Operations

1. **Project Creation**:
   ```sql
   -- Check next available sequence
   SELECT math::max(number.seq) FROM projects WHERE number.year = ? AND number.country = ?;
   
   -- Create project with generated number
   CREATE projects SET name = ?, number = { year: ?, country: ?, seq: ?, id: ? }, ...;
   ```

2. **Validation Queries**:
   ```sql
   -- Check if project number exists
   SELECT id FROM projects WHERE number.id = ?;
   
   -- Check if email exists
   SELECT id FROM contacts WHERE email = ?;
   ```

3. **Foreign Key Handling**:
   - Use `record<table>` type for foreign keys
   - Example: `project_id: record<projects>`
   - SurrealDB automatically validates relationships

### Error Handling

- **Unique Constraint Violations**: Project numbers, email addresses
- **Foreign Key Violations**: Invalid company/contact references  
- **Validation Failures**: Invalid email format, phone format, date format
- **Business Logic Violations**: Invalid status transitions, sequence ranges

### Performance Considerations

- All tables have optimized indexes for common queries
- Project number lookup is indexed for fast validation
- Foreign key relationships are indexed for JOIN operations

---

This documentation provides Claude Code with everything needed to implement the project creation workflow without requiring direct database access.
