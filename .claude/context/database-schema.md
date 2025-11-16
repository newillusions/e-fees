# E-Fees Database Schema
**SurrealDB Schema Reference & Query Patterns**

## 🎯 Purpose

This document provides comprehensive documentation of the e-fees database schema, including table structures, relationships, indexes, and query patterns for SurrealDB.

---

## 📊 Database Overview

**Database Engine**: SurrealDB (embedded mode)  
**Database Location**: `~/.e-fees/data.db` (production), `memory://` (tests)  
**Schema Version**: 1.0

### Entity Summary

| Entity | Purpose | Relationships |
|--------|---------|---------------|
| **contact** | Individual people | → company, → invoice, → project |
| **company** | Organizations | ← contact, → invoice, → project |
| **invoice** | Billing documents | ← contact, ← company, → project |
| **project** | Work projects | ← contact, ← company, ← invoice |
| **user** | Application users | System management |
| **settings** | App configuration | Global settings |

---

## 📋 Core Tables

### Contact Table

**Purpose**: Stores individual contact information

```sql
DEFINE TABLE contact SCHEMAFULL;

DEFINE FIELD name ON TABLE contact TYPE string;
DEFINE FIELD email ON TABLE contact TYPE string
  ASSERT string::is::email($value);
DEFINE FIELD phone ON TABLE contact TYPE option<string>;
DEFINE FIELD company_id ON TABLE contact TYPE option<record<company>>;
DEFINE FIELD address ON TABLE contact TYPE option<object>;
DEFINE FIELD notes ON TABLE contact TYPE option<string>;
DEFINE FIELD tags ON TABLE contact TYPE option<array<string>>;
DEFINE FIELD created_at ON TABLE contact TYPE datetime VALUE time::now();
DEFINE FIELD updated_at ON TABLE contact TYPE datetime VALUE time::now();
DEFINE FIELD active ON TABLE contact TYPE bool DEFAULT true;

-- Indexes
DEFINE INDEX idx_contact_email ON TABLE contact COLUMNS email UNIQUE;
DEFINE INDEX idx_contact_name ON TABLE contact COLUMNS name;
DEFINE INDEX idx_contact_company ON TABLE contact COLUMNS company_id;
DEFINE INDEX idx_contact_active ON TABLE contact COLUMNS active;
```

**Field Descriptions**:
- `name` (required): Full name of the contact
- `email` (required, unique): Email address with validation
- `phone` (optional): Phone number (no format enforcement)
- `company_id` (optional): Link to company record
- `address` (optional): Structured address object
- `notes` (optional): Freeform text notes
- `tags` (optional): Array of category tags
- `created_at` (auto): Timestamp of creation
- `updated_at` (auto): Timestamp of last update
- `active` (default: true): Soft delete flag

**Example Records**:
```javascript
// Valid contact record
{
  id: contact:abc123,
  name: "John Doe",
  email: "john.doe@example.com",
  phone: "+1-555-0123",
  company_id: company:xyz789,
  address: {
    street: "123 Main St",
    city: "San Francisco",
    state: "CA",
    zip: "94105",
    country: "USA"
  },
  notes: "Prefers email communication",
  tags: ["client", "vip"],
  created_at: "2024-01-15T10:30:00Z",
  updated_at: "2024-01-20T14:45:00Z",
  active: true
}
```

---

### Company Table

**Purpose**: Stores organization information

```sql
DEFINE TABLE company SCHEMAFULL;

DEFINE FIELD name ON TABLE company TYPE string;
DEFINE FIELD email ON TABLE company TYPE option<string>
  ASSERT IF $value != NONE THEN string::is::email($value) ELSE true END;
DEFINE FIELD phone ON TABLE company TYPE option<string>;
DEFINE FIELD website ON TABLE company TYPE option<string>;
DEFINE FIELD industry ON TABLE company TYPE option<string>;
DEFINE FIELD address ON TABLE company TYPE option<object>;
DEFINE FIELD tax_id ON TABLE company TYPE option<string>;
DEFINE FIELD notes ON TABLE company TYPE option<string>;
DEFINE FIELD tags ON TABLE company TYPE option<array<string>>;
DEFINE FIELD created_at ON TABLE company TYPE datetime VALUE time::now();
DEFINE FIELD updated_at ON TABLE company TYPE datetime VALUE time::now();
DEFINE FIELD active ON TABLE company TYPE bool DEFAULT true;

-- Indexes
DEFINE INDEX idx_company_name ON TABLE company COLUMNS name UNIQUE;
DEFINE INDEX idx_company_email ON TABLE company COLUMNS email;
DEFINE INDEX idx_company_industry ON TABLE company COLUMNS industry;
DEFINE INDEX idx_company_active ON TABLE company COLUMNS active;
```

**Field Descriptions**:
- `name` (required, unique): Company name
- `email` (optional): Company email with validation
- `phone` (optional): Main phone number
- `website` (optional): Company website URL
- `industry` (optional): Industry classification
- `address` (optional): Company address object
- `tax_id` (optional): Tax identification number
- `notes` (optional): Additional information
- `tags` (optional): Category tags
- `created_at` (auto): Creation timestamp
- `updated_at` (auto): Last update timestamp
- `active` (default: true): Soft delete flag

**Example Records**:
```javascript
{
  id: company:xyz789,
  name: "Acme Corporation",
  email: "info@acme.com",
  phone: "+1-555-0199",
  website: "https://acme.com",
  industry: "Technology",
  address: {
    street: "456 Tech Blvd",
    city: "San Francisco",
    state: "CA",
    zip: "94103",
    country: "USA"
  },
  tax_id: "12-3456789",
  notes: "Major client since 2020",
  tags: ["enterprise", "tech"],
  created_at: "2020-03-10T09:00:00Z",
  updated_at: "2024-01-15T11:20:00Z",
  active: true
}
```

---

### Invoice Table

**Purpose**: Stores billing and invoice information

```sql
DEFINE TABLE invoice SCHEMAFULL;

DEFINE FIELD invoice_number ON TABLE invoice TYPE string;
DEFINE FIELD contact_id ON TABLE invoice TYPE option<record<contact>>;
DEFINE FIELD company_id ON TABLE invoice TYPE option<record<company>>;
DEFINE FIELD project_id ON TABLE invoice TYPE option<record<project>>;
DEFINE FIELD issue_date ON TABLE invoice TYPE datetime;
DEFINE FIELD due_date ON TABLE invoice TYPE datetime;
DEFINE FIELD status ON TABLE invoice TYPE string 
  ASSERT $value IN ["draft", "sent", "paid", "overdue", "cancelled"];
DEFINE FIELD subtotal ON TABLE invoice TYPE decimal;
DEFINE FIELD tax_rate ON TABLE invoice TYPE decimal DEFAULT 0.0;
DEFINE FIELD tax_amount ON TABLE invoice TYPE decimal VALUE $parent.subtotal * $parent.tax_rate;
DEFINE FIELD total ON TABLE invoice TYPE decimal VALUE $parent.subtotal + $parent.tax_amount;
DEFINE FIELD currency ON TABLE invoice TYPE string DEFAULT "USD";
DEFINE FIELD line_items ON TABLE invoice TYPE array<object>;
DEFINE FIELD notes ON TABLE invoice TYPE option<string>;
DEFINE FIELD terms ON TABLE invoice TYPE option<string>;
DEFINE FIELD created_at ON TABLE invoice TYPE datetime VALUE time::now();
DEFINE FIELD updated_at ON TABLE invoice TYPE datetime VALUE time::now();

-- Indexes
DEFINE INDEX idx_invoice_number ON TABLE invoice COLUMNS invoice_number UNIQUE;
DEFINE INDEX idx_invoice_contact ON TABLE invoice COLUMNS contact_id;
DEFINE INDEX idx_invoice_company ON TABLE invoice COLUMNS company_id;
DEFINE INDEX idx_invoice_project ON TABLE invoice COLUMNS project_id;
DEFINE INDEX idx_invoice_status ON TABLE invoice COLUMNS status;
DEFINE INDEX idx_invoice_due_date ON TABLE invoice COLUMNS due_date;
```

**Field Descriptions**:
- `invoice_number` (required, unique): Invoice identifier (e.g., "INV-2024-001")
- `contact_id` (optional): Link to contact
- `company_id` (optional): Link to company
- `project_id` (optional): Link to project
- `issue_date` (required): Date invoice was issued
- `due_date` (required): Payment due date
- `status` (required): One of: draft, sent, paid, overdue, cancelled
- `subtotal` (required): Amount before tax
- `tax_rate` (default: 0.0): Tax rate as decimal (e.g., 0.08 = 8%)
- `tax_amount` (computed): Calculated from subtotal × tax_rate
- `total` (computed): Subtotal + tax_amount
- `currency` (default: "USD"): Currency code
- `line_items` (required): Array of invoice line items
- `notes` (optional): Invoice notes
- `terms` (optional): Payment terms
- `created_at` (auto): Creation timestamp
- `updated_at` (auto): Last update timestamp

**Line Item Structure**:
```javascript
{
  description: "Consulting Services",
  quantity: 10,
  rate: 150.00,
  amount: 1500.00  // quantity × rate
}
```

**Example Records**:
```javascript
{
  id: invoice:inv001,
  invoice_number: "INV-2024-001",
  contact_id: contact:abc123,
  company_id: company:xyz789,
  project_id: project:proj456,
  issue_date: "2024-01-15T00:00:00Z",
  due_date: "2024-02-15T00:00:00Z",
  status: "sent",
  subtotal: 5000.00,
  tax_rate: 0.08,
  tax_amount: 400.00,
  total: 5400.00,
  currency: "USD",
  line_items: [
    {
      description: "Web Development",
      quantity: 20,
      rate: 150.00,
      amount: 3000.00
    },
    {
      description: "Design Services",
      quantity: 10,
      rate: 200.00,
      amount: 2000.00
    }
  ],
  notes: "Payment via wire transfer",
  terms: "Net 30",
  created_at: "2024-01-15T10:00:00Z",
  updated_at: "2024-01-15T10:00:00Z"
}
```

---

### Project Table

**Purpose**: Tracks work projects and their status

```sql
DEFINE TABLE project SCHEMAFULL;

DEFINE FIELD name ON TABLE project TYPE string;
DEFINE FIELD description ON TABLE project TYPE option<string>;
DEFINE FIELD company_id ON TABLE project TYPE option<record<company>>;
DEFINE FIELD contact_id ON TABLE project TYPE option<record<contact>>;
DEFINE FIELD status ON TABLE project TYPE string
  ASSERT $value IN ["planning", "active", "on_hold", "completed", "cancelled"];
DEFINE FIELD start_date ON TABLE project TYPE option<datetime>;
DEFINE FIELD end_date ON TABLE project TYPE option<datetime>;
DEFINE FIELD budget ON TABLE project TYPE option<decimal>;
DEFINE FIELD currency ON TABLE project TYPE string DEFAULT "USD";
DEFINE FIELD tags ON TABLE project TYPE option<array<string>>;
DEFINE FIELD notes ON TABLE project TYPE option<string>;
DEFINE FIELD created_at ON TABLE project TYPE datetime VALUE time::now();
DEFINE FIELD updated_at ON TABLE project TYPE datetime VALUE time::now();

-- Indexes
DEFINE INDEX idx_project_name ON TABLE project COLUMNS name;
DEFINE INDEX idx_project_company ON TABLE project COLUMNS company_id;
DEFINE INDEX idx_project_contact ON TABLE project COLUMNS contact_id;
DEFINE INDEX idx_project_status ON TABLE project COLUMNS status;
```

**Field Descriptions**:
- `name` (required): Project name
- `description` (optional): Project description
- `company_id` (optional): Link to company
- `contact_id` (optional): Primary contact for project
- `status` (required): One of: planning, active, on_hold, completed, cancelled
- `start_date` (optional): Project start date
- `end_date` (optional): Project end date
- `budget` (optional): Project budget amount
- `currency` (default: "USD"): Currency code
- `tags` (optional): Category tags
- `notes` (optional): Additional notes
- `created_at` (auto): Creation timestamp
- `updated_at` (auto): Last update timestamp

**Example Records**:
```javascript
{
  id: project:proj456,
  name: "Website Redesign 2024",
  description: "Complete redesign of company website with new branding",
  company_id: company:xyz789,
  contact_id: contact:abc123,
  status: "active",
  start_date: "2024-01-01T00:00:00Z",
  end_date: "2024-06-30T00:00:00Z",
  budget: 50000.00,
  currency: "USD",
  tags: ["web", "design", "high-priority"],
  notes: "Client requires weekly progress reports",
  created_at: "2023-12-15T09:00:00Z",
  updated_at: "2024-01-15T14:30:00Z"
}
```

---

### User Table

**Purpose**: Application user accounts (future authentication)

```sql
DEFINE TABLE user SCHEMAFULL;

DEFINE FIELD username ON TABLE user TYPE string;
DEFINE FIELD email ON TABLE user TYPE string
  ASSERT string::is::email($value);
DEFINE FIELD password_hash ON TABLE user TYPE string;
DEFINE FIELD role ON TABLE user TYPE string 
  ASSERT $value IN ["admin", "user", "viewer"];
DEFINE FIELD active ON TABLE user TYPE bool DEFAULT true;
DEFINE FIELD last_login ON TABLE user TYPE option<datetime>;
DEFINE FIELD created_at ON TABLE user TYPE datetime VALUE time::now();
DEFINE FIELD updated_at ON TABLE user TYPE datetime VALUE time::now();

-- Indexes
DEFINE INDEX idx_user_username ON TABLE user COLUMNS username UNIQUE;
DEFINE INDEX idx_user_email ON TABLE user COLUMNS email UNIQUE;
```

**Note**: Currently single-user application. Multi-user support planned for future release.

---

### Settings Table

**Purpose**: Application configuration and preferences

```sql
DEFINE TABLE settings SCHEMAFULL;

DEFINE FIELD key ON TABLE settings TYPE string;
DEFINE FIELD value ON TABLE settings TYPE string;
DEFINE FIELD category ON TABLE settings TYPE string;
DEFINE FIELD description ON TABLE settings TYPE option<string>;
DEFINE FIELD updated_at ON TABLE settings TYPE datetime VALUE time::now();

-- Indexes
DEFINE INDEX idx_settings_key ON TABLE settings COLUMNS key UNIQUE;
DEFINE INDEX idx_settings_category ON TABLE settings COLUMNS category;
```

**Example Settings**:
```javascript
{
  key: "default_currency",
  value: "USD",
  category: "invoicing",
  description: "Default currency for new invoices"
},
{
  key: "default_tax_rate",
  value: "0.08",
  category: "invoicing",
  description: "Default tax rate (8%)"
},
{
  key: "invoice_number_prefix",
  value: "INV",
  category: "invoicing",
  description: "Prefix for invoice numbers"
}
```

---

## 🔗 Relationships

### Entity Relationship Diagram

```
┌─────────────┐         ┌─────────────┐
│   company   │◄────────│   contact   │
└──────┬──────┘         └──────┬──────┘
       │                       │
       │                       │
       │                       │
       ▼                       ▼
┌─────────────┐         ┌─────────────┐
│   invoice   │◄────────│   project   │
└─────────────┘         └─────────────┘
```

### Relationship Details

#### Contact → Company (Many-to-One)
```sql
-- A contact belongs to one company
contact.company_id → company.id

-- Query: Get all contacts for a company
SELECT * FROM contact WHERE company_id = company:xyz789;

-- Query: Get contact with company details
SELECT *, company_id.* FROM contact:abc123;
```

#### Invoice → Contact (Many-to-One)
```sql
-- An invoice is billed to one contact
invoice.contact_id → contact.id

-- Query: Get all invoices for a contact
SELECT * FROM invoice WHERE contact_id = contact:abc123;
```

#### Invoice → Company (Many-to-One)
```sql
-- An invoice is billed to one company
invoice.company_id → company.id

-- Query: Get all invoices for a company
SELECT * FROM invoice WHERE company_id = company:xyz789;
```

#### Invoice → Project (Many-to-One)
```sql
-- An invoice can be linked to one project
invoice.project_id → project.id

-- Query: Get all invoices for a project
SELECT * FROM invoice WHERE project_id = project:proj456;
```

#### Project → Company (Many-to-One)
```sql
-- A project belongs to one company
project.company_id → company.id

-- Query: Get all projects for a company
SELECT * FROM project WHERE company_id = company:xyz789;
```

#### Project → Contact (Many-to-One)
```sql
-- A project has one primary contact
project.contact_id → contact.id

-- Query: Get all projects for a contact
SELECT * FROM project WHERE contact_id = contact:abc123;
```

---

## 🔍 Common Query Patterns

### Basic CRUD Operations

#### Create

```sql
-- Create contact
CREATE contact CONTENT {
  name: "Jane Smith",
  email: "jane.smith@example.com",
  phone: "+1-555-0456",
  company_id: company:xyz789,
  tags: ["client", "marketing"]
};

-- Create with specific ID
CREATE contact:custom_id CONTENT {
  name: "John Doe",
  email: "john@example.com"
};
```

#### Read

```sql
-- Get single record
SELECT * FROM contact:abc123;

-- Get all contacts
SELECT * FROM contact;

-- Get with limit
SELECT * FROM contact LIMIT 50;

-- Get specific fields
SELECT name, email FROM contact;

-- Get with nested data
SELECT *, company_id.* FROM contact WHERE company_id != NONE;
```

#### Update

```sql
-- Update specific fields
UPDATE contact:abc123 SET {
  phone: "+1-555-9999",
  updated_at: time::now()
};

-- Conditional update
UPDATE contact SET active = false 
WHERE email CONTAINS "oldcompany.com";

-- Update and return
UPDATE contact:abc123 SET phone = "+1-555-8888" RETURN AFTER;
```

#### Delete

```sql
-- Hard delete
DELETE contact:abc123;

-- Soft delete
UPDATE contact:abc123 SET active = false;

-- Delete with condition
DELETE FROM contact WHERE name CONTAINS "DELETE ME";
```

---

### Advanced Query Patterns

#### Filtering

```sql
-- Simple filter
SELECT * FROM contact WHERE active = true;

-- Multiple conditions
SELECT * FROM contact 
WHERE active = true 
  AND company_id = company:xyz789;

-- CONTAINS operator
SELECT * FROM contact WHERE name CONTAINS "Smith";

-- IN operator
SELECT * FROM invoice WHERE status IN ["sent", "overdue"];

-- Range queries
SELECT * FROM invoice 
WHERE due_date >= "2024-01-01" 
  AND due_date < "2024-02-01";

-- NULL checks
SELECT * FROM contact WHERE company_id = NONE;
SELECT * FROM contact WHERE company_id != NONE;
```

#### Sorting

```sql
-- Ascending
SELECT * FROM contact ORDER BY name ASC;

-- Descending
SELECT * FROM invoice ORDER BY due_date DESC;

-- Multiple fields
SELECT * FROM contact 
ORDER BY company_id ASC, name ASC;
```

#### Pagination

```sql
-- LIMIT and START
SELECT * FROM contact LIMIT 20 START 0;   -- Page 1
SELECT * FROM contact LIMIT 20 START 20;  -- Page 2
SELECT * FROM contact LIMIT 20 START 40;  -- Page 3

-- With sorting
SELECT * FROM contact 
ORDER BY name ASC 
LIMIT 20 START 0;
```

#### Aggregations

```sql
-- Count
SELECT count() AS total FROM contact;
SELECT count() AS active_contacts FROM contact WHERE active = true;

-- Group by
SELECT company_id, count() AS contact_count 
FROM contact 
GROUP BY company_id;

-- Sum
SELECT sum(total) AS total_revenue FROM invoice WHERE status = "paid";

-- Average
SELECT avg(total) AS average_invoice FROM invoice;

-- Min/Max
SELECT min(due_date) AS earliest, max(due_date) AS latest FROM invoice;
```

#### Joins (via Record Links)

```sql
-- Get contact with company details
SELECT 
  id,
  name,
  email,
  company_id.name AS company_name,
  company_id.industry AS company_industry
FROM contact
WHERE company_id != NONE;

-- Get invoice with related data
SELECT 
  id,
  invoice_number,
  total,
  contact_id.name AS contact_name,
  company_id.name AS company_name,
  project_id.name AS project_name
FROM invoice:inv001;

-- Get all invoices for a company with contact info
SELECT 
  id,
  invoice_number,
  total,
  status,
  contact_id.name AS contact_name
FROM invoice
WHERE company_id = company:xyz789;
```

#### Full-Text Search

```sql
-- Search contacts by name or email
SELECT * FROM contact 
WHERE name CONTAINS "smith" 
   OR email CONTAINS "smith";

-- Case-insensitive search
SELECT * FROM contact 
WHERE string::lowercase(name) CONTAINS string::lowercase("SMITH");

-- Search across multiple tables
SELECT id, name, "contact" AS type FROM contact WHERE name CONTAINS "acme"
UNION
SELECT id, name, "company" AS type FROM company WHERE name CONTAINS "acme";
```

---

## 🎯 Optimized Query Patterns

### Use Indexes Effectively

```sql
-- ✅ GOOD - Uses index on email
SELECT * FROM contact WHERE email = "john@example.com";

-- ✅ GOOD - Uses index on company_id
SELECT * FROM contact WHERE company_id = company:xyz789;

-- ❌ BAD - Full table scan (no index on notes)
SELECT * FROM contact WHERE notes CONTAINS "important";

-- ✅ BETTER - Add index if this is a common query
DEFINE INDEX idx_contact_notes ON TABLE contact COLUMNS notes;
```

### Limit Result Sets

```sql
-- ❌ BAD - Returns all records (could be thousands)
SELECT * FROM contact;

-- ✅ GOOD - Limit results
SELECT * FROM contact LIMIT 50;

-- ✅ BETTER - Paginate
SELECT * FROM contact ORDER BY name LIMIT 50 START 0;
```

### Select Only Needed Fields

```sql
-- ❌ BAD - Returns all fields including large ones
SELECT * FROM contact;

-- ✅ GOOD - Select only what's needed
SELECT id, name, email FROM contact;

-- ✅ GOOD - Exclude large fields
SELECT id, name, email, phone FROM contact;
```

### Batch Operations

```sql
-- ❌ BAD - Multiple individual creates (from application code)
-- for contact in contacts:
--   CREATE contact CONTENT contact

-- ✅ GOOD - Batch insert
INSERT INTO contact [
  { name: "Contact 1", email: "c1@example.com" },
  { name: "Contact 2", email: "c2@example.com" },
  { name: "Contact 3", email: "c3@example.com" }
];
```

### Use Transactions

```rust
// Rust example: Multiple operations in one transaction
let result = db.transaction(|tx| async move {
    // Create invoice
    let invoice = create_invoice_tx(&tx, invoice_data).await?;
    
    // Update project status
    update_project_tx(&tx, project_id, status).await?;
    
    // Create activity log
    create_log_tx(&tx, activity).await?;
    
    Ok(invoice)
}).await?;
```

---

## 🧪 Test Data Patterns

### "DELETE ME" Pattern

**CRITICAL**: All test data MUST include "DELETE ME" prefix

```sql
-- ✅ CORRECT - Test data
CREATE contact CONTENT {
  name: "DELETE ME - Test Contact 1234567890",
  email: "delete-me-1234567890@example.com"
};

-- ❌ WRONG - Will fail code review
CREATE contact CONTENT {
  name: "Test Contact",
  email: "test@example.com"
};
```

### Test Data Cleanup

```sql
-- Find all test data
SELECT * FROM contact WHERE name CONTAINS "DELETE ME";

-- Delete all test data
DELETE FROM contact WHERE name CONTAINS "DELETE ME";

-- Verify cleanup
SELECT count() FROM contact WHERE name CONTAINS "DELETE ME";
-- Should return: 0
```

### Comprehensive Cleanup Query

```sql
-- Clean all tables
DELETE FROM contact WHERE name CONTAINS "DELETE ME";
DELETE FROM company WHERE name CONTAINS "DELETE ME";
DELETE FROM invoice WHERE invoice_number CONTAINS "DELETE ME";
DELETE FROM project WHERE name CONTAINS "DELETE ME";

-- Verify
SELECT 
  (SELECT count() FROM contact WHERE name CONTAINS "DELETE ME") AS contacts,
  (SELECT count() FROM company WHERE name CONTAINS "DELETE ME") AS companies,
  (SELECT count() FROM invoice WHERE invoice_number CONTAINS "DELETE ME") AS invoices,
  (SELECT count() FROM project WHERE name CONTAINS "DELETE ME") AS projects;
```

---

## 📈 Performance Optimization

### Index Strategy

**Current Indexes**:
```sql
-- Contact indexes
idx_contact_email (email) - UNIQUE
idx_contact_name (name)
idx_contact_company (company_id)
idx_contact_active (active)

-- Company indexes
idx_company_name (name) - UNIQUE
idx_company_email (email)
idx_company_industry (industry)
idx_company_active (active)

-- Invoice indexes
idx_invoice_number (invoice_number) - UNIQUE
idx_invoice_contact (contact_id)
idx_invoice_company (company_id)
idx_invoice_project (project_id)
idx_invoice_status (status)
idx_invoice_due_date (due_date)

-- Project indexes
idx_project_name (name)
idx_project_company (company_id)
idx_project_contact (contact_id)
idx_project_status (status)
```

### When to Add Indexes

**Add index if**:
- ✅ Column used frequently in WHERE clauses
- ✅ Column used in JOIN conditions (record links)
- ✅ Column used in ORDER BY frequently
- ✅ Performance profiling shows slow queries

**Don't add index if**:
- ❌ Table has few records (< 1000)
- ❌ Column has low cardinality (few unique values)
- ❌ Column rarely queried
- ❌ High write frequency (indexes slow down writes)

### Query Performance Tips

```sql
-- ✅ Use specific WHERE clauses
SELECT * FROM contact WHERE email = "specific@example.com";

-- ❌ Avoid broad patterns at start of string
SELECT * FROM contact WHERE name CONTAINS "%smith%";

-- ✅ Better: If searching from start
SELECT * FROM contact WHERE name STARTSWITH "Smith";

-- ✅ Use EXPLAIN to analyze queries (when available)
-- EXPLAIN SELECT * FROM contact WHERE company_id = company:xyz789;
```

---

## 🔒 Data Validation & Constraints

### Email Validation

```sql
-- Enforced at schema level
DEFINE FIELD email ON TABLE contact TYPE string
  ASSERT string::is::email($value);

-- Test
CREATE contact CONTENT {
  name: "Test",
  email: "invalid-email"  -- ❌ Will fail
};
```

### Enum Validation

```sql
-- Status must be one of specific values
DEFINE FIELD status ON TABLE invoice TYPE string 
  ASSERT $value IN ["draft", "sent", "paid", "overdue", "cancelled"];

-- Test
UPDATE invoice:inv001 SET status = "unknown";  -- ❌ Will fail
```

### Computed Fields

```sql
-- Tax amount computed from subtotal and rate
DEFINE FIELD tax_amount ON TABLE invoice TYPE decimal 
  VALUE $parent.subtotal * $parent.tax_rate;

-- Total computed from subtotal and tax
DEFINE FIELD total ON TABLE invoice TYPE decimal 
  VALUE $parent.subtotal + $parent.tax_amount;

-- These fields are auto-calculated
CREATE invoice CONTENT {
  invoice_number: "INV-001",
  subtotal: 1000.00,
  tax_rate: 0.08
  -- tax_amount will be 80.00
  -- total will be 1080.00
};
```

### Required vs Optional

```sql
-- Required (no OPTION wrapper)
DEFINE FIELD name ON TABLE contact TYPE string;

-- Optional (with OPTION wrapper)
DEFINE FIELD phone ON TABLE contact TYPE option<string>;

-- Optional with nested type
DEFINE FIELD company_id ON TABLE contact TYPE option<record<company>>;
```

---

## 🔄 Migrations

### Migration Strategy

**Location**: `src-tauri/src/db/migrations/`

```rust
// Example: 001_initial_schema.rs

pub async fn up(db: &Database) -> Result<(), String> {
    // Create tables
    db.execute("
        DEFINE TABLE contact SCHEMAFULL;
        DEFINE FIELD name ON TABLE contact TYPE string;
        DEFINE FIELD email ON TABLE contact TYPE string;
        -- ... more fields
    ").await?;
    
    // Create indexes
    db.execute("
        DEFINE INDEX idx_contact_email ON TABLE contact COLUMNS email UNIQUE;
    ").await?;
    
    Ok(())
}

pub async fn down(db: &Database) -> Result<(), String> {
    db.execute("REMOVE TABLE contact;").await?;
    Ok(())
}
```

### Migration Best Practices

**DO**:
- ✅ Version migrations sequentially (001, 002, 003...)
- ✅ Test migrations on copy of production data
- ✅ Write both up() and down() functions
- ✅ Make migrations idempotent when possible
- ✅ Backup database before migration

**DON'T**:
- ❌ Modify existing migrations after deployed
- ❌ Delete data without backup
- ❌ Change field types without data conversion
- ❌ Skip migration testing

---

## 🔗 Rust Integration

### Database Connection

```rust
// src-tauri/src/db/mod.rs

use surrealdb::Surreal;
use surrealdb::engine::local::RocksDb;

pub struct Database {
    client: Surreal<RocksDb>
}

impl Database {
    pub async fn new(path: &str) -> Result<Self, String> {
        let db = Surreal::new::<RocksDb>(path)
            .await
            .map_err(|e| e.to_string())?;
        
        // Use namespace and database
        db.use_ns("e_fees").use_db("production")
            .await
            .map_err(|e| e.to_string())?;
        
        Ok(Database { client: db })
    }
}
```

### Query Execution

```rust
// src-tauri/src/db/contacts/queries.rs

use surrealdb::sql::Thing;
use crate::models::Contact;

pub async fn get_contact(
    db: &Database, 
    id: &str
) -> Result<Contact, String> {
    let result: Option<Contact> = db.client
        .select(("contact", id))
        .await
        .map_err(|e| e.to_string())?;
    
    result.ok_or_else(|| "Contact not found".to_string())
}

pub async fn create_contact(
    db: &Database,
    contact: Contact
) -> Result<Contact, String> {
    let created: Contact = db.client
        .create("contact")
        .content(contact)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(created)
}

pub async fn search_contacts(
    db: &Database,
    query: &str
) -> Result<Vec<Contact>, String> {
    let sql = format!(
        "SELECT * FROM contact WHERE name CONTAINS '{}' OR email CONTAINS '{}'",
        query, query
    );
    
    let results: Vec<Contact> = db.client
        .query(sql)
        .await
        .map_err(|e| e.to_string())?
        .take(0)
        .map_err(|e| e.to_string())?;
    
    Ok(results)
}
```

---

## 🛠️ Maintenance Tasks

### Database Backup

```rust
// Backup strategy (to be implemented)
pub async fn backup_database(output_path: &str) -> Result<(), String> {
    // Export all tables to JSON
    // Compress and store
    // TODO: Implement
    Ok(())
}
```

### Database Statistics

```sql
-- Count records per table
SELECT 
  (SELECT count() FROM contact) AS contacts,
  (SELECT count() FROM company) AS companies,
  (SELECT count() FROM invoice) AS invoices,
  (SELECT count() FROM project) AS projects;

-- Active vs inactive
SELECT 
  (SELECT count() FROM contact WHERE active = true) AS active_contacts,
  (SELECT count() FROM contact WHERE active = false) AS inactive_contacts;

-- Invoice status breakdown
SELECT status, count() AS count 
FROM invoice 
GROUP BY status;
```

---

## 🔗 Related Documentation

- **MCP Architecture**: `.claude/context/mcp-architecture.md`
- **Testing Strategy**: `.claude/context/testing-strategy.md`
- **Database Patterns Prompt**: `.claude/prompts/database-patterns.md`
- **Database Specialist Sub-Agent**: `.claude/subagents/subagent-database-specialist.md`

---

## 📚 External Resources

- **SurrealDB Documentation**: https://surrealdb.com/docs
- **SurrealQL Reference**: https://surrealdb.com/docs/surrealql
- **Rust SDK**: https://surrealdb.com/docs/sdk/rust

---

**Last Updated**: October 26, 2025  
**Version**: 1.0  
**Schema Version**: 1.0  
**Maintained By**: Martin & Claude Code
