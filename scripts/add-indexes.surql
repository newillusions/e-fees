-- E-Fees Database Index Additions
-- Run with: surreal sql --endpoint ws://10.0.1.17:8000 --namespace emittiv --database projects --username martin --password <pass> < scripts/add-indexes.surql
-- Or execute each statement individually in SurrealDB console

-- ============================================================
-- Performance Indexes for E-Fees Application
-- ============================================================

-- 1. Project lookups by year/country (used for sequence generation)
-- Query: SELECT number.seq FROM projects WHERE number.year = 25 AND number.country = 971
DEFINE INDEX IF NOT EXISTS idx_project_year_country ON projects FIELDS number.year, number.country;

-- 2. Project filtering by status (dashboard, lists)
-- Query: SELECT * FROM projects WHERE status = 'Active'
DEFINE INDEX IF NOT EXISTS idx_project_status ON projects FIELDS status;

-- 3. Project filtering by country (for area/city dropdowns)
-- Query: SELECT area FROM projects WHERE country = 'U.A.E.'
DEFINE INDEX IF NOT EXISTS idx_project_country ON projects FIELDS country;

-- 4. Fee/RFP status filtering (dashboard, proposal lists)
-- Query: SELECT * FROM fee WHERE status IN ['Sent', 'Negotiation']
DEFINE INDEX IF NOT EXISTS idx_fee_status ON fee FIELDS status;

-- 5. Company name search (lookups, autocomplete)
-- Query: SELECT * FROM company WHERE name CONTAINS 'Hilton'
DEFINE INDEX IF NOT EXISTS idx_company_name ON company FIELDS name;

-- 6. Contact company lookup (for joins, foreign key lookups)
-- Query: SELECT * FROM contacts WHERE company = company:CHE
DEFINE INDEX IF NOT EXISTS idx_contact_company ON contacts FIELDS company;

-- 7. Fee project lookup (for finding all proposals for a project)
-- Query: SELECT * FROM fee WHERE project_id = projects:⟨25_97105⟩
DEFINE INDEX IF NOT EXISTS idx_fee_project ON fee FIELDS project_id;

-- ============================================================
-- Verify indexes were created
-- ============================================================
-- Run: INFO FOR DB;
-- Or for specific table: INFO FOR TABLE projects;
