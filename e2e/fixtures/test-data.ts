/**
 * Test data fixtures for E2E tests
 * Provides consistent test data and helper functions for database seeding
 */

export interface TestCompany {
  name: string
  name_short: string
  abbreviation: string
  city: string
  country: string
  registration_no?: string
  tax_no?: string
}

export interface TestContact {
  first_name: string
  last_name: string
  full_name: string
  email: string
  phone: string
  position: string
  company_id?: string
}

export interface TestProject {
  name: string
  name_short: string
  area: string
  city: string
  country: string
  status: string
  folder?: string
}

export interface TestProposal {
  name: string
  number: string
  project_id?: string
  company_id?: string
  contact_id?: string
  issue_date: string
  activity: string
  status: string
}

// Test companies data
export const testCompanies: TestCompany[] = [
  {
    name: "Emirates Engineering Consultancy LLC",
    name_short: "EEC",
    abbreviation: "EEC",
    city: "Dubai",
    country: "United Arab Emirates",
    registration_no: "REG-EEC-001",
    tax_no: "TRN-100234567890"
  },
  {
    name: "Saudi Design Studio",
    name_short: "SDS",
    abbreviation: "SDS", 
    city: "Riyadh",
    country: "Saudi Arabia",
    registration_no: "REG-SDS-002",
    tax_no: "TRN-200345678901"
  },
  {
    name: "Gulf Architecture Partners",
    name_short: "GAP",
    abbreviation: "GAP",
    city: "Doha",
    country: "Qatar",
    registration_no: "REG-GAP-003",
    tax_no: "TRN-300456789012"
  }
]

// Test contacts data
export const testContacts: TestContact[] = [
  {
    first_name: "Ahmed",
    last_name: "Al-Mansoori",
    full_name: "Ahmed Al-Mansoori",
    email: "ahmed.almansoori@eec.ae",
    phone: "+971-50-123-4567",
    position: "Project Manager"
  },
  {
    first_name: "Sarah",
    last_name: "Al-Rashid", 
    full_name: "Sarah Al-Rashid",
    email: "sarah.alrashid@sds.sa",
    phone: "+966-55-234-5678",
    position: "Design Director"
  },
  {
    first_name: "Omar",
    last_name: "Al-Thani",
    full_name: "Omar Al-Thani", 
    email: "omar.althani@gap.qa",
    phone: "+974-66-345-6789",
    position: "Lead Architect"
  }
]

// Test projects data
export const testProjects: TestProject[] = [
  {
    name: "Dubai Museum of Future Heritage",
    name_short: "DMFH",
    area: "Business Bay",
    city: "Dubai",
    country: "United Arab Emirates",
    status: "Concept",
    folder: "/test-projects/25-97101-dubai-museum-future-heritage"
  },
  {
    name: "Riyadh Cultural Center",
    name_short: "RCC",
    area: "King Fahd District", 
    city: "Riyadh",
    country: "Saudi Arabia",
    status: "Draft",
    folder: "/test-projects/25-96601-riyadh-cultural-center"
  },
  {
    name: "Doha Innovation Hub",
    name_short: "DIH",
    area: "West Bay",
    city: "Doha", 
    country: "Qatar",
    status: "Planning",
    folder: "/test-projects/25-97401-doha-innovation-hub"
  }
]

// Test proposals data
export const testProposals: TestProposal[] = [
  {
    name: "Dubai Museum Design Proposal",
    number: "RFP-25-001",
    issue_date: "250815",
    activity: "Design and Consultancy",
    status: "Draft"
  },
  {
    name: "Riyadh Cultural Center Development",
    number: "RFP-25-002", 
    issue_date: "250820",
    activity: "Master Planning and Design",
    status: "Prepared"
  },
  {
    name: "Doha Innovation Hub Architecture",
    number: "RFP-25-003",
    issue_date: "250825", 
    activity: "Architectural Design Services",
    status: "Sent"
  }
]

// Reference data for countries and currencies
export const referenceCountries = [
  { name: "United Arab Emirates", dial_code: 971, code: "AE" },
  { name: "Saudi Arabia", dial_code: 966, code: "SA" },
  { name: "Qatar", dial_code: 974, code: "QA" },
  { name: "Kuwait", dial_code: 965, code: "KW" },
  { name: "Bahrain", dial_code: 973, code: "BH" },
  { name: "Oman", dial_code: 968, code: "OM" }
]

export const referenceCurrencies = [
  { code: "AED", name: "UAE Dirham", symbol: "د.إ" },
  { code: "SAR", name: "Saudi Riyal", symbol: "ر.س" },
  { code: "QAR", name: "Qatari Riyal", symbol: "ر.ق" },
  { code: "USD", name: "US Dollar", symbol: "$" },
  { code: "EUR", name: "Euro", symbol: "€" }
]

/**
 * Generate minimal test dataset for quick tests
 */
export function getMinimalTestData() {
  return {
    companies: testCompanies.slice(0, 1),
    contacts: testContacts.slice(0, 1),
    projects: testProjects.slice(0, 1),
    proposals: testProposals.slice(0, 1)
  }
}

/**
 * Generate complete test dataset for full workflow tests
 */
export function getCompleteTestData() {
  return {
    companies: testCompanies,
    contacts: testContacts,
    projects: testProjects,
    proposals: testProposals
  }
}

/**
 * Generate test data with relationships established
 */
export function getLinkedTestData() {
  const data = getCompleteTestData()
  
  // Link contacts to companies
  data.contacts[0].company_id = "company:eec"
  data.contacts[1].company_id = "company:sds"  
  data.contacts[2].company_id = "company:gap"
  
  // Link proposals to projects, companies, and contacts
  data.proposals[0].project_id = "project:dmfh"
  data.proposals[0].company_id = "company:eec"
  data.proposals[0].contact_id = "contact:ahmed"
  
  data.proposals[1].project_id = "project:rcc"
  data.proposals[1].company_id = "company:sds"
  data.proposals[1].contact_id = "contact:sarah"
  
  data.proposals[2].project_id = "project:dih"
  data.proposals[2].company_id = "company:gap"
  data.proposals[2].contact_id = "contact:omar"
  
  return data
}

/**
 * Generate edge case test data for validation testing
 */
export function getEdgeCaseTestData() {
  return {
    companies: [
      {
        name: "A".repeat(100), // Very long name
        name_short: "Long Co",
        abbreviation: "LC",
        city: "Test City",
        country: "United Arab Emirates"
      },
      {
        name: "Special-Chars & Co. (LLC)",
        name_short: "SC&Co",
        abbreviation: "SC&",
        city: "Test-City",
        country: "Saudi Arabia"
      }
    ],
    contacts: [
      {
        first_name: "Test",
        last_name: "User",
        full_name: "Test User",
        email: "test@example-domain-with-very-long-name.co.ae", 
        phone: "+971-50-000-0000",
        position: "Test Position with Very Long Title"
      }
    ],
    projects: [
      {
        name: "Test Project with Extremely Long Name That Tests Input Limits",
        name_short: "TPWELNT",
        area: "Test Area",
        city: "Test City",
        country: "United Arab Emirates",
        status: "Draft"
      }
    ]
  }
}

/**
 * Generate empty state test data
 */
export function getEmptyTestData() {
  return {
    companies: [],
    contacts: [],
    projects: [],
    proposals: []
  }
}