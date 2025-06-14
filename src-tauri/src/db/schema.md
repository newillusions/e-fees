# SurrealDB Schema Reference

## Database Connection
- **URL**: ws://10.0.1.17:8000
- **Namespace**: emittiv
- **Database**: projects
- **User**: martin
- **Password**: Set via SURREALDB_PASS environment variable

## Tables Overview

### 1. projects
**Purpose**: Project opportunities emittiv is bidding on
**ID Format**: `projects:YY_CCCNN` (auto-generated)
**Type**: SCHEMAFULL

#### Fields:
- `name`: string (required, length > 0)
- `name_short`: string (required)
- `status`: enum (required)
  - 'Draft'
  - 'RFP'
  - 'Active'
  - 'On Hold'
  - 'Completed'
  - 'Cancelled'
- `area`: string (required)
- `city`: string (required)
- `country`: string (required)
- `folder`: string (required)
- `number`: object (required)
  - `year`: number (required, 20-50)
  - `country`: number (required)
  - `seq`: number (required, 1-999)
  - `id`: string (computed)
- `time`: object (auto-managed)
  - `created_at`: datetime (auto-set on create)
  - `updated_at`: datetime (auto-set on update)

#### Indexes:
- `project_number_unique`: UNIQUE on number.id
- `project_status`: on status
- `project_country`: on country
- `project_status_country`: on status, country
- `project_lookup`: on number.id
- `project_created`: on time.created_at
- `project_updated`: on time.updated_at

### 2. rfp
**Purpose**: Fee proposals created by emittiv staff
**ID Format**: `rfp:YY_CCCNN_R` (auto-generated)
**Type**: SCHEMAFULL

#### Fields:
- `name`: string (required, length > 0)
- `number`: string (required)
- `project_id`: record<projects> (required)
- `company_id`: record<company> (required)
- `contact_id`: record<contacts> (required)
- `status`: enum (default: 'Draft')
  - 'Draft'
  - 'Active'
  - 'Sent'
  - 'Awarded'
  - 'Lost'
  - 'Cancelled'
- `stage`: enum (default: 'Draft')
  - 'Draft'
  - 'Prepared'
  - 'Sent'
  - 'Under Review'
  - 'Clarification'
  - 'Negotiation'
  - 'Awarded'
  - 'Lost'
- `issue_date`: string (required, 6 digits YYMMDD format, numeric)
- `activity`: string
- `package`: string
- `strap_line`: string
- `staff_name`: string
- `staff_email`: string
- `staff_phone`: string
- `staff_position`: string
- `rev`: int (auto-computed from revisions array)
- `revisions`: array<object> (default: [])
  - `revision_number`: int (required)
  - `revision_date`: datetime (required)
  - `author_email`: string (required, valid email)
  - `author_name`: string (required)
  - `notes`: string (required)
- `time`: object (auto-managed)
  - `created_at`: datetime
  - `updated_at`: datetime

#### Indexes:
- `rfp_project_rev`: UNIQUE on project_id, rev
- `rfp_status`: on status
- `rfp_stage`: on stage
- `rfp_status_stage`: on status, stage
- `rfp_project`: on project_id
- `rfp_company`: on company_id
- `rfp_contact`: on contact_id
- `rfp_created`: on time.created_at

### 3. company
**Purpose**: Client companies that issue project opportunities
**ID Format**: `company:ABBREVIATION` (manual format, e.g. company:SLG)
**Type**: SCHEMAFULL

#### Fields:
- `name`: string (required, length > 0)
- `name_short`: string (required)
- `abbreviation`: string (required)
- `city`: string (required)
- `country`: string (required)
- `reg_no`: option<string>
- `tax_no`: option<string>
- `time`: object (auto-managed)
  - `created_at`: datetime
  - `updated_at`: datetime

#### Indexes:
- `company_name`: on name
- `company_country`: on country
- `company_abbreviation`: on abbreviation
- `company_country_city`: on country, city
- `company_created`: on time.created_at

### 4. contacts
**Purpose**: Individual contact persons at client companies
**ID Format**: `contacts:random_id` (auto-generated)
**Type**: SCHEMAFULL

#### Fields:
- `first_name`: string (required)
- `last_name`: string (required)
- `email`: string (required, valid email, unique)
- `phone`: string (required, must contain '+', length > 0)
- `position`: string (required)
- `company`: record<company> (required)
- `full_name`: string (auto-computed: first_name + ' ' + last_name)
- `time`: object (auto-managed)
  - `created_at`: datetime
  - `updated_at`: datetime

#### Indexes:
- `contact_email_unique`: UNIQUE on email
- `contact_company`: on company
- `contact_last_name`: on last_name
- `contact_created`: on time.created_at

### 5. country
**Purpose**: Reference table for countries
**ID Format**: `country:CODE` (e.g. country:AE)
**Type**: SCHEMAFULL

#### Fields:
- `name`: string (required)
- `name_formal`: string (required)
- `name_official`: string (required)
- `code`: string (required)
- `code_alt`: string (required)
- `dial_code`: number (required)
- `currency_code`: record<currency> (flexible type)

### 6. currency
**Purpose**: Reference table for currencies
**ID Format**: `currency:CODE` (e.g. currency:USD)
**Type**: SCHEMAFULL

#### Fields:
- `code`: string (required)
- `name`: string (required)

## Business Logic

### Project Numbering System
- Format: YY-CCCNN (e.g. 24-97101)
- YY: 2-digit year
- CCC: Country dial code (971=UAE, 966=Saudi)
- NN: Sequence number
- UAE projects start from 97101
- Saudi projects start from 96601
- Sequence resets annually per country

### RFP Revision Management
- `rev` field is auto-computed from revisions array
- To add a revision: Add object to revisions array
- Required revision data: revision_number, revision_date, author_email, author_name, notes

## Key Relationships
- rfp → projects (many-to-one)
- rfp → company (many-to-one)
- rfp → contacts (many-to-one)
- contacts → company (many-to-one)
- country → currency (many-to-one)
