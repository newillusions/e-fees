# SurrealDB Query Examples

## Connection
```rust
// Connection string
let db = Surreal::new::<Ws>("ws://10.0.1.17:8000").await?;

// Authentication
db.signin(Root {
    username: "root",
    password: &password,
}).await?;

// Or namespace/database level auth
db.use_ns("emittiv").use_db("projects").await?;
```

## Projects

### Create a new project
```sql
CREATE projects:24_97101 CONTENT {
    name: "Museum of the Future",
    name_short: "MOTF",
    status: "RFP",
    area: "Downtown",
    city: "Dubai",
    country: "UAE",
    folder: "24-97101 Museum of the Future",
    number: {
        year: 24,
        country: 971,
        seq: 1,
        id: "24-97101"
    }
};
```

### Get next project number for a country
```sql
-- For UAE (971) in 2025
SELECT max(number.seq) as max_seq FROM projects 
WHERE number.year = 25 AND number.country = 971;

-- If no results, start with seq = 1
-- If results, use max_seq + 1
```

### List active projects
```sql
SELECT * FROM projects 
WHERE status IN ["RFP", "Active"] 
ORDER BY time.created_at DESC;
```

### Search projects
```sql
SELECT * FROM projects 
WHERE name ~ $search_term 
OR name_short ~ $search_term 
OR number.id ~ $search_term;
```

## RFPs

### Create a new RFP
```sql
CREATE rfp CONTENT {
    name: "Museum of the Future - Lighting Design Services",
    number: "24-97101-01",
    project_id: projects:24_97101,
    company_id: company:RTA,
    contact_id: contacts:abc123,
    status: "Draft",
    stage: "Draft",
    issue_date: "250115",
    activity: "Design and Consultancy",
    package: "Lighting Design",
    strap_line: "sensory design studio",
    staff_name: "John Smith",
    staff_email: "john@emittiv.com",
    staff_phone: "+971501234567",
    staff_position: "Senior Designer",
    revisions: []
};
```

### Add a revision to RFP
```sql
UPDATE rfp:xyz789 SET revisions += {
    revision_number: 1,
    revision_date: time::now(),
    author_email: "john@emittiv.com",
    author_name: "John Smith",
    notes: "Updated pricing based on client feedback"
};
-- Note: rev field will auto-update to 1
```

### Get RFPs with related data
```sql
SELECT *,
    project_id.* as project,
    company_id.* as company,
    contact_id.* as contact
FROM rfp
WHERE status = "Active"
ORDER BY time.created_at DESC;
```

### Get latest revision of RFPs for a project
```sql
SELECT * FROM rfp 
WHERE project_id = projects:24_97101 
AND rev = (
    SELECT max(rev) FROM rfp 
    WHERE project_id = projects:24_97101
);
```

## Companies

### Create a company
```sql
CREATE company:MERAAS CONTENT {
    name: "Meraas Holding LLC",
    name_short: "Meraas",
    abbreviation: "MERAAS",
    city: "Dubai",
    country: "UAE",
    reg_no: "REG123456",
    tax_no: "TAX789012"
};
```

### Search companies
```sql
SELECT * FROM company 
WHERE name ~ $search_term 
OR abbreviation ~ $search_term
ORDER BY name;
```

## Contacts

### Create a contact
```sql
CREATE contacts CONTENT {
    first_name: "Ahmed",
    last_name: "Al Rashid",
    email: "ahmed.rashid@meraas.ae",
    phone: "+971501234567",
    position: "Project Manager",
    company: company:MERAAS
};
-- Note: full_name will be auto-computed as "Ahmed Al Rashid"
```

### Get contacts for a company
```sql
SELECT * FROM contacts 
WHERE company = company:MERAAS
ORDER BY last_name, first_name;
```

### Check if email exists
```sql
SELECT count() as total FROM contacts 
WHERE email = "ahmed.rashid@meraas.ae";
```

## Countries & Currencies

### Get country with currency
```sql
SELECT *, currency_code.* as currency 
FROM country 
WHERE code = "AE";
```

### Get countries for dropdown
```sql
SELECT name, dial_code, code 
FROM country 
ORDER BY name;
```

### Get currencies for dropdown
```sql
SELECT code, name 
FROM currency 
ORDER BY code;
```

## InDesign Export Query

### Get all data needed for proposal export
```sql
-- 1. Get RFP with all related data
LET $rfp = (
    SELECT *,
        project_id.* as project,
        company_id.* as company,
        contact_id.* as contact
    FROM rfp:specific_id
)[0];

-- 2. Format for InDesign JSON export
RETURN {
    "01 Document Name": $rfp.name,
    "02 Document Number": $rfp.number,
    "03 Document Issue Date": $rfp.issue_date, -- App needs to format as 'dd MMM yyyy'
    "04 Document Revision": $rfp.rev,
    "06 Project Number": $rfp.project.number.id,
    "07 Project Name": $rfp.project.name,
    "08 Project Name Short": $rfp.project.name_short,
    "09 Project Location": $rfp.project.city,
    "10 Project Country": $rfp.project.country,
    "11 Project Area": $rfp.project.area,
    "21 Client Name": $rfp.company.name,
    "22 Client Name Short": $rfp.company.name_short,
    "23 Client City": $rfp.company.city,
    "24 Client Country": $rfp.company.country,
    "26 Contact Full Name": $rfp.contact.full_name,
    "27 Contact Position": $rfp.contact.position,
    "28 Contact Email": $rfp.contact.email,
    "29 Contact Phone": $rfp.contact.phone,
    "99 Strap Line": $rfp.strap_line
};
```

## Transactions

### Create project with RFP in transaction
```sql
BEGIN TRANSACTION;

-- Create project
CREATE projects:25_97102 CONTENT {
    name: "Dubai Creek Tower",
    -- ... all fields
};

-- Create RFP
CREATE rfp CONTENT {
    name: "Dubai Creek Tower - Lighting Design",
    project_id: projects:25_97102,
    -- ... all fields
};

COMMIT TRANSACTION;
```

## Common Patterns

### Pagination
```sql
SELECT * FROM projects 
ORDER BY time.created_at DESC 
LIMIT 20 
START 0;
```

### Count records
```sql
SELECT count() as total FROM projects WHERE status = "Active";
```

### Check unique constraint
```sql
-- Before creating project
SELECT count() as exists FROM projects WHERE number.id = "25-97102";
```

### Soft delete pattern (if needed)
```sql
-- Add deleted field
UPDATE projects:24_97101 SET deleted = true, deleted_at = time::now();

-- Query non-deleted
SELECT * FROM projects WHERE deleted != true;
```
