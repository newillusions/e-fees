/**
 * Safe Test Data Generation for E2E Testing
 * 
 * ALL test data includes "DELETE ME" prefix for easy identification and cleanup.
 * This ensures test data can never be confused with production records.
 */

export interface SafeTestData {
  projects: SafeProjectData[]
  companies: SafeCompanyData[]
  contacts: SafeContactData[]
  proposals: SafeProposalData[]
}

export interface SafeProjectData {
  name: string
  description: string
  location: string
  country: string
  area: string
  city: string
  client: string
  status: 'active' | 'completed' | 'on_hold' | 'cancelled'
  start_date?: string
  end_date?: string
}

export interface SafeCompanyData {
  name: string
  description: string
  address: string
  city: string
  country: string
  phone?: string
  email?: string
  website?: string
  industry: string
}

export interface SafeContactData {
  first_name: string
  last_name: string
  email: string
  phone?: string
  position?: string
  company_id?: string
  notes?: string
}

export interface SafeProposalData {
  name: string
  description: string
  project_id?: string
  company_id?: string
  contact_id?: string
  status: 'draft' | 'sent' | 'approved' | 'rejected' | 'revised'
  amount?: number
  currency?: string
  submission_date?: string
}

export class SafeTestDataGenerator {
  private testRunId: string
  private timestamp: string

  constructor() {
    this.timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19)
    this.testRunId = `e2e-${this.timestamp}`
  }

  /**
   * Generate a safe test name with DELETE ME prefix and timestamp
   */
  private generateSafeName(baseName: string, type: string = ''): string {
    const typePrefix = type ? `${type} ` : ''
    return `DELETE ME - ${typePrefix}${baseName} ${this.timestamp}`
  }

  /**
   * Generate safe project test data
   */
  generateSafeProjects(count: number = 1): SafeProjectData[] {
    const projects: SafeProjectData[] = []
    const locations = [
      { location: 'Dubai', area: 'Downtown Dubai', city: 'Dubai', country: 'UAE' },
      { location: 'Abu Dhabi', area: 'Al Reem Island', city: 'Abu Dhabi', country: 'UAE' },
      { location: 'Riyadh', area: 'King Abdullah Financial District', city: 'Riyadh', country: 'Saudi Arabia' },
      { location: 'Jeddah', area: 'Corniche', city: 'Jeddah', country: 'Saudi Arabia' },
      { location: 'Doha', area: 'West Bay', city: 'Doha', country: 'Qatar' }
    ]

    const projectTypes = [
      'Office Building',
      'Residential Tower', 
      'Shopping Mall',
      'Mixed Use Development',
      'Hospital Complex',
      'School Campus',
      'Hotel Resort'
    ]

    for (let i = 0; i < count; i++) {
      const location = locations[i % locations.length]
      const projectType = projectTypes[i % projectTypes.length]
      
      projects.push({
        name: this.generateSafeName(`${projectType} ${location.location}`, 'Project'),
        description: this.generateSafeName(`E2E testing ${projectType.toLowerCase()} in ${location.location}. This is test data for automated testing.`, 'Description'),
        location: location.location,
        country: location.country,
        area: location.area,
        city: location.city,
        client: this.generateSafeName(`${location.city} Development Authority`, 'Client'),
        status: ['active', 'completed', 'on_hold'][i % 3] as any,
        start_date: new Date(Date.now() + (i * 24 * 60 * 60 * 1000)).toISOString().split('T')[0],
        end_date: new Date(Date.now() + ((i + 90) * 24 * 60 * 60 * 1000)).toISOString().split('T')[0]
      })
    }

    return projects
  }

  /**
   * Generate safe company test data
   */
  generateSafeCompanies(count: number = 1): SafeCompanyData[] {
    const companies: SafeCompanyData[] = []
    const companyTypes = [
      'Construction LLC',
      'Development Group',
      'Engineering Consultants',
      'Architecture Studio',
      'Property Investments',
      'Building Contractors',
      'Infrastructure Solutions'
    ]

    const locations = [
      { city: 'Dubai', country: 'UAE', address: 'Sheikh Zayed Road' },
      { city: 'Abu Dhabi', country: 'UAE', address: 'Corniche Road' },
      { city: 'Riyadh', country: 'Saudi Arabia', address: 'King Fahd Road' },
      { city: 'Jeddah', country: 'Saudi Arabia', address: 'Tahlia Street' },
      { city: 'Doha', country: 'Qatar', address: 'C Ring Road' }
    ]

    const industries = [
      'Construction',
      'Real Estate Development',
      'Engineering Services',
      'Architecture',
      'Project Management',
      'Infrastructure',
      'Property Investment'
    ]

    for (let i = 0; i < count; i++) {
      const location = locations[i % locations.length]
      const companyType = companyTypes[i % companyTypes.length]
      const industry = industries[i % industries.length]
      
      companies.push({
        name: this.generateSafeName(`${location.city} ${companyType}`, 'Company'),
        description: this.generateSafeName(`E2E testing company specializing in ${industry.toLowerCase()}. Test data for automated testing.`, 'Description'),
        address: `${location.address}, ${location.city}`,
        city: location.city,
        country: location.country,
        phone: `+971-${50 + i}-${Math.floor(1000000 + Math.random() * 9000000)}`,
        email: `test.${i}.${this.timestamp}@delete-me-testing.com`,
        website: `https://delete-me-test-${i}-${this.timestamp}.com`,
        industry
      })
    }

    return companies
  }

  /**
   * Generate safe contact test data
   */
  generateSafeContacts(count: number = 1, companyIds?: string[]): SafeContactData[] {
    const contacts: SafeContactData[] = []
    const firstNames = ['John', 'Sarah', 'Ahmed', 'Fatima', 'Mohammed', 'Aisha', 'Abdullah', 'Maryam']
    const positions = [
      'Project Manager',
      'Senior Architect', 
      'Construction Manager',
      'Development Director',
      'Engineering Manager',
      'Design Lead',
      'Operations Manager'
    ]

    for (let i = 0; i < count; i++) {
      const firstName = firstNames[i % firstNames.length]
      const position = positions[i % positions.length]
      
      contacts.push({
        first_name: this.generateSafeName(firstName, 'Contact'),
        last_name: this.generateSafeName('TestUser', ''),
        email: `test.contact.${i}.${this.timestamp}@delete-me-testing.com`,
        phone: `+971-${50 + i}-${Math.floor(1000000 + Math.random() * 9000000)}`,
        position: `${position} (DELETE ME Test)`,
        company_id: companyIds && companyIds[i % companyIds.length] || undefined,
        notes: this.generateSafeName(`E2E testing contact for automated tests. Contact #${i + 1}`, 'Notes')
      })
    }

    return contacts
  }

  /**
   * Generate safe proposal test data
   */
  generateSafeProposals(count: number = 1, linkedIds?: { 
    projectIds?: string[], 
    companyIds?: string[], 
    contactIds?: string[] 
  }): SafeProposalData[] {
    const proposals: SafeProposalData[] = []
    const proposalTypes = [
      'Architectural Design Services',
      'Construction Management',
      'Engineering Consultation', 
      'Project Development',
      'Infrastructure Design',
      'Building Maintenance',
      'Facility Management'
    ]

    const currencies = ['USD', 'AED', 'SAR', 'QAR']
    const statuses: SafeProposalData['status'][] = ['draft', 'sent', 'approved', 'rejected', 'revised']

    for (let i = 0; i < count; i++) {
      const proposalType = proposalTypes[i % proposalTypes.length]
      const currency = currencies[i % currencies.length]
      const status = statuses[i % statuses.length]
      
      proposals.push({
        name: this.generateSafeName(`${proposalType} Proposal`, 'Proposal'),
        description: this.generateSafeName(`E2E testing proposal for ${proposalType.toLowerCase()}. Automated test data.`, 'Description'),
        project_id: linkedIds?.projectIds?.[i % linkedIds.projectIds.length],
        company_id: linkedIds?.companyIds?.[i % linkedIds.companyIds.length],
        contact_id: linkedIds?.contactIds?.[i % linkedIds.contactIds.length],
        status,
        amount: Math.floor(50000 + Math.random() * 500000), // 50k to 550k
        currency,
        submission_date: new Date(Date.now() + (i * 24 * 60 * 60 * 1000)).toISOString().split('T')[0]
      })
    }

    return proposals
  }

  /**
   * Generate complete safe test dataset with relationships
   */
  generateCompleteTestDataset(options: {
    projectCount?: number
    companyCount?: number  
    contactCount?: number
    proposalCount?: number
  } = {}): SafeTestData {
    const {
      projectCount = 3,
      companyCount = 2,
      contactCount = 4,
      proposalCount = 3
    } = options

    const projects = this.generateSafeProjects(projectCount)
    const companies = this.generateSafeCompanies(companyCount)
    const contacts = this.generateSafeContacts(contactCount)
    const proposals = this.generateSafeProposals(proposalCount)

    return {
      projects,
      companies,
      contacts,
      proposals
    }
  }

  /**
   * Generate minimal test dataset (1 of each type)
   */
  generateMinimalTestDataset(): SafeTestData {
    return this.generateCompleteTestDataset({
      projectCount: 1,
      companyCount: 1,
      contactCount: 1,
      proposalCount: 1
    })
  }

  /**
   * Generate edge case test data with special characters, long names, etc.
   */
  generateEdgeCaseTestData(): SafeTestData {
    const edgeProjects: SafeProjectData[] = [
      {
        name: this.generateSafeName('Project with "Quotes" & Special Characters!@#$%', 'EdgeCase'),
        description: this.generateSafeName('Testing special characters: àáâãäåæçèéêëìíîïñòóôõöøùúûüýÿ 中文 العربية', 'Description'),
        location: 'Dubai',
        country: 'UAE',
        area: 'Downtown Dubai',
        city: 'Dubai', 
        client: this.generateSafeName('Client with Very Long Name That Exceeds Normal Character Limits for Testing Purposes', 'Client'),
        status: 'active'
      }
    ]

    const edgeCompanies: SafeCompanyData[] = [
      {
        name: this.generateSafeName('Company名前 with "Mixed" Languages & Symbols!', 'Company'),
        description: this.generateSafeName('Testing unicode: 🏢🏗️🏠 àáâã العربية 中文 русский', 'Description'),
        address: '123 Very Long Address Line That Tests Maximum Field Length Handling',
        city: 'Abu Dhabi',
        country: 'UAE',
        phone: '+971-50-1234567',
        email: 'test.unicode.email+with.special.chars@delete-me-testing.com',
        website: 'https://delete-me-unicode-test-中文.com',
        industry: 'Construction & Engineering Services'
      }
    ]

    const edgeContacts: SafeContactData[] = [
      {
        first_name: this.generateSafeName('João', 'Contact'),
        last_name: this.generateSafeName('François-Pierre', 'Contact'),
        email: 'test.unicode+special.chars@delete-me-testing.com',
        phone: '+971-50-1234567',
        position: 'Senior Manager & Lead Consultant',
        notes: this.generateSafeName('Unicode test: àáâãäåæçèéêë 中文 العربية русский', 'Notes')
      }
    ]

    const edgeProposals: SafeProposalData[] = [
      {
        name: this.generateSafeName('Proposal with "Special Chars" & Very Long Title That Tests Field Limits', 'Proposal'),
        description: this.generateSafeName('Testing maximum description length with special characters: àáâãäåæç 中文 العربية', 'Description'),
        status: 'draft',
        amount: 999999.99,
        currency: 'USD'
      }
    ]

    return {
      projects: edgeProjects,
      companies: edgeCompanies,
      contacts: edgeContacts,
      proposals: edgeProposals
    }
  }

  /**
   * Get the test run identifier
   */
  getTestRunId(): string {
    return this.testRunId
  }

  /**
   * Get the timestamp for this test run
   */
  getTimestamp(): string {
    return this.timestamp
  }
}

/**
 * Singleton instance for consistent test data generation
 */
export const safeTestDataGenerator = new SafeTestDataGenerator()

/**
 * Convenience functions
 */
export const generateSafeProjectData = (count: number = 1) => safeTestDataGenerator.generateSafeProjects(count)
export const generateSafeCompanyData = (count: number = 1) => safeTestDataGenerator.generateSafeCompanies(count)
export const generateSafeContactData = (count: number = 1) => safeTestDataGenerator.generateSafeContacts(count)
export const generateSafeProposalData = (count: number = 1) => safeTestDataGenerator.generateSafeProposals(count)
export const generateCompleteTestDataset = (options?: Parameters<SafeTestDataGenerator['generateCompleteTestDataset']>[0]) => 
  safeTestDataGenerator.generateCompleteTestDataset(options)
export const generateMinimalTestDataset = () => safeTestDataGenerator.generateMinimalTestDataset()
export const generateEdgeCaseTestData = () => safeTestDataGenerator.generateEdgeCaseTestData()