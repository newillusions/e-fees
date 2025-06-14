# SurrealDB Validation Rules Reference

## Field Validation Rules

### projects table
- **name**: 
  - Required
  - Length > 0
  - Type: string
  
- **name_short**: 
  - Required
  - Type: string
  
- **activity**: 
  - Required
  - Must be one of: ['Design and Consultancy', 'Management', 'Construction', 'Maintenance', 'Research', 'Other']
  
- **package**: 
  - Required
  - Type: string
  
- **status**: 
  - Required
  - Must be one of: ['Draft', 'RFP', 'Active', 'On Hold', 'Completed', 'Cancelled']
  
- **area**: 
  - Required
  - Type: string
  
- **city**: 
  - Required
  - Type: string
  
- **country**: 
  - Required
  - Type: string
  
- **folder**: 
  - Required
  - Type: string
  
- **number.year**: 
  - Required
  - Range: 20-50
  - Type: number
  
- **number.country**: 
  - Required
  - Type: number (dial code)
  
- **number.seq**: 
  - Required
  - Range: 1-999
  - Type: number
  
- **number.id**: 
  - Computed field
  - Unique constraint
  - Format: YY-CCCNN

### rfp table
- **name**: 
  - Required
  - Length > 0
  - Type: string
  
- **number**: 
  - Required
  - Type: string
  
- **project_id**: 
  - Required
  - Type: record<projects>
  - Must reference existing project
  
- **company_id**: 
  - Required
  - Type: record<company>
  - Must reference existing company
  
- **contact_id**: 
  - Required
  - Type: record<contacts>
  - Must reference existing contact
  
- **status**: 
  - Default: 'Draft'
  - Must be one of: ['Draft', 'Active', 'Sent', 'Awarded', 'Lost', 'Cancelled']
  
- **stage**: 
  - Default: 'Draft'
  - Must be one of: ['Draft', 'Prepared', 'Sent', 'Under Review', 'Clarification', 'Negotiation', 'Awarded', 'Lost']
  
- **issue_date**: 
  - Required
  - Format: YYMMDD (6 digits, all numeric)
  - Type: string
  
- **rev**: 
  - Computed from max(revisions[*].revision_number)
  - Default: 0 if no revisions
  - Read-only for create/update operations
  
- **revisions**: 
  - Default: []
  - Type: array<object>
  - Each revision must have:
    - revision_number: int
    - revision_date: datetime
    - author_email: valid email
    - author_name: string
    - notes: string
  
- **project_id + rev**: 
  - Unique constraint (only one revision number per project)

### company table
- **name**: 
  - Required
  - Length > 0
  - Type: string
  
- **name_short**: 
  - Required
  - Type: string
  
- **abbreviation**: 
  - Required
  - Type: string
  - Used in ID format
  
- **city**: 
  - Required
  - Type: string
  
- **country**: 
  - Required
  - Type: string
  
- **reg_no**: 
  - Optional
  - Type: option<string>
  
- **tax_no**: 
  - Optional
  - Type: option<string>

### contacts table
- **first_name**: 
  - Required
  - Type: string
  
- **last_name**: 
  - Required
  - Type: string
  
- **email**: 
  - Required
  - Must be valid email format
  - Unique constraint
  - Type: string
  
- **phone**: 
  - Required
  - Must contain '+'
  - Length > 0
  - Type: string
  
- **position**: 
  - Required
  - Type: string
  
- **company**: 
  - Required
  - Type: record<company>
  - Must reference existing company
  
- **full_name**: 
  - Computed: first_name + ' ' + last_name
  - Read-only

### country table
- All fields required
- No specific validation rules
- Pre-populated reference table

### currency table
- All fields required
- No specific validation rules
- Pre-populated reference table

## Unique Constraints
1. **projects.number.id**: Each project number must be unique
2. **rfp.project_id + rfp.rev**: Only one RFP revision per project
3. **contacts.email**: Each email must be unique

## Auto-Computed Fields
1. **projects.number.id**: Generated from year, country, seq
2. **rfp.rev**: Max value from revisions array
3. **contacts.full_name**: Concatenation of first_name and last_name
4. **time.created_at**: Set on record creation
5. **time.updated_at**: Updated on every modification

## ID Formats
1. **projects**: `projects:YY_CCCNN` (e.g., projects:24_97101)
2. **rfp**: `rfp:YY_CCCNN_R` (e.g., rfp:24_97101_1)
3. **company**: `company:ABBREVIATION` (e.g., company:MERAAS)
4. **contacts**: `contacts:random_id` (auto-generated)
5. **country**: `country:CODE` (e.g., country:AE)
6. **currency**: `currency:CODE` (e.g., currency:USD)

## Business Rules
1. **Project Numbering**:
   - UAE projects: Start at 97101 each year
   - Saudi projects: Start at 96601 each year
   - Sequence resets annually
   - Country code embedded in number

2. **RFP Revisions**:
   - Always append to revisions array
   - Never modify existing revisions
   - Rev number auto-increments
   - Each revision requires author info

3. **Email Validation**:
   - Basic format: contains '@'
   - Must be unique across all contacts

4. **Phone Validation**:
   - Must start with '+' (international format)
   - Cannot be empty

## Error Messages
```typescript
// TypeScript validation
const errors = {
  name_required: "Name is required",
  name_empty: "Name cannot be empty",
  email_invalid: "Invalid email format",
  email_duplicate: "Email already exists",
  phone_invalid: "Phone must contain '+' and not be empty",
  year_range: "Year must be between 20 and 50",
  seq_range: "Sequence must be between 1 and 999",
  issue_date_format: "Issue date must be 6 digits in YYMMDD format",
  status_invalid: "Invalid status value",
  reference_not_found: "Referenced record does not exist"
};
```

## Validation Implementation Tips
1. **Frontend**: Validate before sending to backend
2. **Backend**: Re-validate before database operations
3. **Database**: Schema enforces final validation
4. **User Experience**: Show helpful error messages
5. **Batch Operations**: Validate all records before starting transaction
