import { Page } from '@playwright/test'
import { 
  testCompanies, 
  testContacts, 
  testProjects, 
  testProposals,
  referenceCountries,
  referenceCurrencies,
  getMinimalTestData,
  getCompleteTestData,
  getLinkedTestData,
  getEmptyTestData
} from './test-data'

/**
 * Database fixture management for E2E tests
 * Handles seeding, cleanup, and state management for test database
 */
export class DatabaseFixtures {
  constructor(private page: Page) {}

  /**
   * Initialize database with reference data (countries, currencies)
   */
  async initializeReferenceData() {
    await this.page.evaluate(async (data) => {
      const { referenceCountries, referenceCurrencies } = data
      
      const response = await fetch('http://localhost:8001/sql', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Basic ' + btoa('root:test')
        },
        body: JSON.stringify({
          sql: `
            USE NS test DB e2e;
            
            -- Clear existing reference data
            DELETE FROM country;
            DELETE FROM currency;
            
            -- Insert countries
            ${referenceCountries.map(country => 
              `INSERT INTO country { name: "${country.name}", dial_code: ${country.dial_code}, code: "${country.code}" };`
            ).join('\n')}
            
            -- Insert currencies
            ${referenceCurrencies.map(currency =>
              `INSERT INTO currency { code: "${currency.code}", name: "${currency.name}", symbol: "${currency.symbol}" };`
            ).join('\n')}
          `
        })
      })
      
      if (!response.ok) {
        throw new Error('Failed to initialize reference data')
      }
    }, { referenceCountries, referenceCurrencies })
  }

  /**
   * Seed database with minimal test data (1 record per entity type)
   */
  async seedMinimalData() {
    const data = getMinimalTestData()
    await this.clearAllData()
    await this.initializeReferenceData()
    await this.seedData(data)
  }

  /**
   * Seed database with complete test dataset
   */
  async seedCompleteData() {
    const data = getCompleteTestData()
    await this.clearAllData()
    await this.initializeReferenceData()
    await this.seedData(data)
  }

  /**
   * Seed database with linked test data (relationships established)
   */
  async seedLinkedData() {
    const data = getLinkedTestData()
    await this.clearAllData()
    await this.initializeReferenceData()
    await this.seedLinkedData(data)
  }

  /**
   * Clear all test data but preserve reference data
   */
  async clearTestData() {
    await this.page.evaluate(async () => {
      const response = await fetch('http://localhost:8001/sql', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Basic ' + btoa('root:test')
        },
        body: JSON.stringify({
          sql: `
            USE NS test DB e2e;
            DELETE FROM rfp;
            DELETE FROM contacts;
            DELETE FROM projects;
            DELETE FROM company;
          `
        })
      })
      
      if (!response.ok) {
        throw new Error('Failed to clear test data')
      }
    })
  }

  /**
   * Clear all data including reference data
   */
  async clearAllData() {
    await this.page.evaluate(async () => {
      const response = await fetch('http://localhost:8001/sql', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Basic ' + btoa('root:test')
        },
        body: JSON.stringify({
          sql: `
            USE NS test DB e2e;
            DELETE FROM rfp;
            DELETE FROM contacts;
            DELETE FROM projects;
            DELETE FROM company;
            DELETE FROM currency;
            DELETE FROM country;
          `
        })
      })
      
      if (!response.ok) {
        throw new Error('Failed to clear all data')
      }
    })
  }

  /**
   * Generic data seeding function
   */
  private async seedData(data: any) {
    await this.page.evaluate(async (testData) => {
      const { companies, contacts, projects, proposals } = testData
      
      let sql = 'USE NS test DB e2e;\n'
      
      // Insert companies
      if (companies.length > 0) {
        companies.forEach((company: any, index: number) => {
          const id = `company:${company.abbreviation.toLowerCase()}`
          sql += `INSERT INTO company:${company.abbreviation.toLowerCase()} {
            name: "${company.name}",
            name_short: "${company.name_short}",
            abbreviation: "${company.abbreviation}",
            city: "${company.city}",
            country: "${company.country}"
            ${company.registration_no ? `, registration_no: "${company.registration_no}"` : ''}
            ${company.tax_no ? `, tax_no: "${company.tax_no}"` : ''}
          };\n`
        })
      }
      
      // Insert projects
      if (projects.length > 0) {
        projects.forEach((project: any, index: number) => {
          const country = project.country === "United Arab Emirates" ? 971 :
                         project.country === "Saudi Arabia" ? 966 : 974
          const projectId = `project:${project.name_short.toLowerCase()}`
          
          sql += `INSERT INTO projects:${project.name_short.toLowerCase()} {
            name: "${project.name}",
            name_short: "${project.name_short}",
            area: "${project.area}",
            city: "${project.city}",
            country: "${project.country}",
            status: "${project.status}",
            number: {
              year: 25,
              country: ${country},
              seq: ${index + 1},
              id: "25-${country}${String(index + 1).padStart(2, '0')}"
            }
            ${project.folder ? `, folder: "${project.folder}"` : ''}
          };\n`
        })
      }
      
      // Insert contacts
      if (contacts.length > 0) {
        contacts.forEach((contact: any, index: number) => {
          const contactId = `contact:${contact.first_name.toLowerCase()}`
          sql += `INSERT INTO contacts:${contact.first_name.toLowerCase()} {
            first_name: "${contact.first_name}",
            last_name: "${contact.last_name}",
            full_name: "${contact.full_name}",
            email: "${contact.email}",
            phone: "${contact.phone}",
            position: "${contact.position}"
          };\n`
        })
      }
      
      // Insert proposals
      if (proposals.length > 0) {
        proposals.forEach((proposal: any, index: number) => {
          const proposalId = `rfp:${proposal.number.toLowerCase().replace(/-/g, '_')}`
          sql += `INSERT INTO rfp:${proposal.number.toLowerCase().replace(/-/g, '_')} {
            name: "${proposal.name}",
            number: "${proposal.number}",
            issue_date: "${proposal.issue_date}",
            activity: "${proposal.activity}",
            status: "${proposal.status}"
          };\n`
        })
      }
      
      const response = await fetch('http://localhost:8001/sql', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Basic ' + btoa('root:test')
        },
        body: JSON.stringify({ sql })
      })
      
      if (!response.ok) {
        throw new Error('Failed to seed test data')
      }
    }, data)
  }

  /**
   * Seed data with relationships established
   */
  private async seedLinkedData(data: any) {
    // First seed basic data
    await this.seedData(data)
    
    // Then establish relationships
    await this.page.evaluate(async (testData) => {
      const { contacts, proposals } = testData
      
      let sql = 'USE NS test DB e2e;\n'
      
      // Link contacts to companies
      contacts.forEach((contact: any) => {
        if (contact.company_id) {
          const contactId = `contacts:${contact.first_name.toLowerCase()}`
          sql += `UPDATE ${contactId} SET company = ${contact.company_id};\n`
        }
      })
      
      // Link proposals to projects, companies, and contacts
      proposals.forEach((proposal: any) => {
        const proposalId = `rfp:${proposal.number.toLowerCase().replace(/-/g, '_')}`
        if (proposal.project_id) {
          sql += `UPDATE ${proposalId} SET project = ${proposal.project_id};\n`
        }
        if (proposal.company_id) {
          sql += `UPDATE ${proposalId} SET company = ${proposal.company_id};\n`
        }
        if (proposal.contact_id) {
          sql += `UPDATE ${proposalId} SET contact = ${proposal.contact_id};\n`
        }
      })
      
      const response = await fetch('http://localhost:8001/sql', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Basic ' + btoa('root:test')
        },
        body: JSON.stringify({ sql })
      })
      
      if (!response.ok) {
        throw new Error('Failed to establish relationships')
      }
    }, data)
  }

  /**
   * Verify database state - useful for debugging
   */
  async verifyDatabaseState() {
    return await this.page.evaluate(async () => {
      const response = await fetch('http://localhost:8001/sql', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Basic ' + btoa('root:test')
        },
        body: JSON.stringify({
          sql: `
            USE NS test DB e2e;
            SELECT count() FROM company GROUP ALL;
            SELECT count() FROM contacts GROUP ALL;
            SELECT count() FROM projects GROUP ALL;
            SELECT count() FROM rfp GROUP ALL;
            SELECT count() FROM country GROUP ALL;
            SELECT count() FROM currency GROUP ALL;
          `
        })
      })
      
      if (!response.ok) {
        throw new Error('Failed to verify database state')
      }
      
      const result = await response.json()
      return result
    })
  }

  /**
   * Get current database counts for assertions
   */
  async getDatabaseCounts() {
    const state = await this.verifyDatabaseState()
    return {
      companies: state[0]?.result?.[0]?.count || 0,
      contacts: state[1]?.result?.[0]?.count || 0,
      projects: state[2]?.result?.[0]?.count || 0,
      proposals: state[3]?.result?.[0]?.count || 0,
      countries: state[4]?.result?.[0]?.count || 0,
      currencies: state[5]?.result?.[0]?.count || 0
    }
  }
}