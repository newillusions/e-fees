# Sample Data for Testing

## Countries (pre-populated)
```json
{
  "countries": [
    { "id": "country:AE", "name": "United Arab Emirates", "dial_code": 971, "code": "AE" },
    { "id": "country:SA", "name": "Saudi Arabia", "dial_code": 966, "code": "SA" },
    { "id": "country:QA", "name": "Qatar", "dial_code": 974, "code": "QA" },
    { "id": "country:KW", "name": "Kuwait", "dial_code": 965, "code": "KW" },
    { "id": "country:OM", "name": "Oman", "dial_code": 968, "code": "OM" }
  ]
}
```

## Currencies (pre-populated)
```json
{
  "currencies": [
    { "id": "currency:AED", "code": "AED", "name": "UAE Dirham" },
    { "id": "currency:SAR", "code": "SAR", "name": "Saudi Riyal" },
    { "id": "currency:QAR", "code": "QAR", "name": "Qatari Riyal" },
    { "id": "currency:USD", "code": "USD", "name": "US Dollar" },
    { "id": "currency:EUR", "code": "EUR", "name": "Euro" }
  ]
}
```

## Companies
```json
{
  "companies": [
    {
      "id": "company:MERAAS",
      "name": "Meraas Holding LLC",
      "name_short": "Meraas",
      "abbreviation": "MERAAS",
      "city": "Dubai",
      "country": "UAE",
      "reg_no": "REG-2007-0123",
      "tax_no": "TRN-100223344100003"
    },
    {
      "id": "company:RTA",
      "name": "Roads and Transport Authority",
      "name_short": "RTA",
      "abbreviation": "RTA",
      "city": "Dubai",
      "country": "UAE",
      "reg_no": "GOV-2005-0001",
      "tax_no": "TRN-100000000000000"
    },
    {
      "id": "company:EMAAR",
      "name": "Emaar Properties PJSC",
      "name_short": "Emaar",
      "abbreviation": "EMAAR",
      "city": "Dubai",
      "country": "UAE",
      "reg_no": "REG-1997-0456",
      "tax_no": "TRN-100111222333444"
    }
  ]
}
```

## Contacts
```json
{
  "contacts": [
    {
      "first_name": "Ahmed",
      "last_name": "Al Rashid",
      "email": "ahmed.rashid@meraas.ae",
      "phone": "+971501234567",
      "position": "Senior Project Manager",
      "company": "company:MERAAS"
    },
    {
      "first_name": "Sarah",
      "last_name": "Johnson",
      "email": "sarah.johnson@rta.ae",
      "phone": "+971502345678",
      "position": "Design Director",
      "company": "company:RTA"
    },
    {
      "first_name": "Mohammed",
      "last_name": "Al Maktoum",
      "email": "m.almaktoum@emaar.ae",
      "phone": "+971503456789",
      "position": "VP Development",
      "company": "company:EMAAR"
    }
  ]
}
```

## Projects
```json
{
  "projects": [
    {
      "id": "projects:24_97101",
      "name": "Museum of the Future - Lighting Enhancement",
      "name_short": "MOTF Lighting",
      "activity": "Design and Consultancy",
      "package": "Lighting Design",
      "status": "Active",
      "stage": "Design Development",
      "area": "Sheikh Zayed Road",
      "city": "Dubai",
      "country": "UAE",
      "folder": "24-97101 Museum of the Future",
      "number": {
        "year": 24,
        "country": 971,
        "seq": 1,
        "id": "24-97101"
      }
    },
    {
      "id": "projects:24_97102",
      "name": "Dubai Creek Tower - Facade Lighting",
      "name_short": "DCT Facade",
      "activity": "Design and Consultancy",
      "package": "Facade Lighting",
      "status": "RFP",
      "stage": "Concept",
      "area": "Dubai Creek Harbour",
      "city": "Dubai",
      "country": "UAE",
      "folder": "24-97102 Dubai Creek Tower",
      "number": {
        "year": 24,
        "country": 971,
        "seq": 2,
        "id": "24-97102"
      }
    },
    {
      "id": "projects:24_96601",
      "name": "King Abdullah Financial District - Plaza Lighting",
      "name_short": "KAFD Plaza",
      "activity": "Design and Consultancy",
      "package": "Landscape Lighting",
      "status": "Active",
      "stage": "Documentation",
      "area": "KAFD",
      "city": "Riyadh",
      "country": "Saudi Arabia",
      "folder": "24-96601 KAFD Plaza",
      "number": {
        "year": 24,
        "country": 966,
        "seq": 1,
        "id": "24-96601"
      }
    }
  ]
}
```

## RFPs
```json
{
  "rfps": [
    {
      "name": "Museum of the Future - Lighting Enhancement Services",
      "number": "24-97101-01",
      "project_id": "projects:24_97101",
      "company_id": "company:RTA",
      "contact_id": "contacts:abc123", // Reference to Sarah Johnson
      "status": "Sent",
      "stage": "Under Review",
      "issue_date": "240915",
      "activity": "Design and Consultancy",
      "package": "Lighting Design",
      "strap_line": "sensory design studio",
      "staff_name": "John Smith",
      "staff_email": "john@emittiv.com",
      "staff_phone": "+971504567890",
      "staff_position": "Senior Lighting Designer",
      "revisions": [
        {
          "revision_number": 0,
          "revision_date": "2024-09-15T10:00:00Z",
          "author_email": "john@emittiv.com",
          "author_name": "John Smith",
          "notes": "Initial proposal"
        },
        {
          "revision_number": 1,
          "revision_date": "2024-09-20T14:30:00Z",
          "author_email": "john@emittiv.com",
          "author_name": "John Smith",
          "notes": "Updated pricing based on client feedback"
        }
      ]
    },
    {
      "name": "Dubai Creek Tower - Facade Lighting Design",
      "number": "24-97102-01",
      "project_id": "projects:24_97102",
      "company_id": "company:EMAAR",
      "contact_id": "contacts:def456", // Reference to Mohammed Al Maktoum
      "status": "Draft",
      "stage": "Prepared",
      "issue_date": "241020",
      "activity": "Design and Consultancy",
      "package": "Facade Lighting",
      "strap_line": "sensory design studio",
      "staff_name": "Emma Wilson",
      "staff_email": "emma@emittiv.com",
      "staff_phone": "+971505678901",
      "staff_position": "Design Director",
      "revisions": []
    }
  ]
}
```

## Test Scenarios

### 1. Create New Project (UAE)
```typescript
// Get next sequence number for UAE 2025
const maxSeq = await db.query("SELECT max(number.seq) as max_seq FROM projects WHERE number.year = 25 AND number.country = 971");
const nextSeq = (maxSeq[0]?.max_seq || 0) + 1;

const newProject = {
  name: "Expo City Dubai - Interactive Lighting",
  name_short: "Expo City",
  status: "RFP",
  area: "Expo City",
  city: "Dubai",
  country: "UAE",
  folder: "25-97101 Expo City Dubai",
  number: {
    year: 25,
    country: 971,
    seq: nextSeq,
    id: `25-971${nextSeq.toString().padStart(2, '0')}`
  }
};
```

### 2. Add Revision to RFP
```typescript
const revision = {
  revision_number: 2,
  revision_date: new Date().toISOString(),
  author_email: "john@emittiv.com",
  author_name: "John Smith",
  notes: "Adjusted timeline per client request"
};

await db.query(`UPDATE rfp:${rfpId} SET revisions += $revision`, { revision });
```

### 3. InDesign Export Data
```typescript
// Expected output format
{
  "01 Document Name": "Museum of the Future - Lighting Enhancement Services",
  "02 Document Number": "24-97101-01",
  "03 Document Issue Date": "15 Sep 2024", // Formatted from "240915"
  "04 Document Revision": 1,
  "06 Project Number": "24-97101",
  "07 Project Name": "Museum of the Future - Lighting Enhancement",
  "08 Project Name Short": "MOTF Lighting",
  "09 Project Location": "Dubai",
  "10 Project Country": "UAE",
  "11 Project Activity": "Design and Consultancy",
  "12 Project Package": "Lighting Design",
  "13 Project Stage": "Design Development",
  "21 Client Name": "Roads and Transport Authority",
  "22 Client Name Short": "RTA",
  "23 Client City": "Dubai",
  "24 Client Country": "UAE",
  "26 Contact Full Name": "Sarah Johnson",
  "27 Contact Position": "Design Director",
  "28 Contact Email": "sarah.johnson@rta.ae",
  "29 Contact Phone": "+971502345678",
  "99 Strap Line": "sensory design studio"
}
```

## Common Test Cases
1. **Duplicate email**: Try creating contact with existing email - should fail
2. **Invalid project number**: Try using seq > 999 - should fail
3. **Missing required field**: Try creating project without name - should fail
4. **Invalid enum value**: Try using status "Pending" - should fail
5. **Phone without +**: Try creating contact with "971501234567" - should fail
6. **Issue date format**: Try "24-09-15" instead of "240915" - should fail
7. **Project number uniqueness**: Try creating duplicate project ID - should fail
8. **Revision without required fields**: Try adding revision without author_email - should fail
