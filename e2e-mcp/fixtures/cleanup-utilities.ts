/**
 * Safe Test Data Cleanup Utilities for E2E Testing
 * 
 * Provides comprehensive cleanup capabilities for test data with "DELETE ME" identification.
 * Ensures test data can be safely removed without affecting production records.
 */

export interface CleanupReport {
  timestamp: string
  deletedCounts: {
    projects: number
    companies: number
    contacts: number
    proposals: number
  }
  errors: string[]
  totalDeleted: number
  duration: number
}

export interface TestDataQuery {
  table: string
  identificationField: string
  pattern: string
}

export class SafeTestDataCleaner {
  private static readonly DELETE_ME_PATTERN = 'DELETE ME'
  private static readonly TEST_DATA_QUERIES: TestDataQuery[] = [
    { table: 'projects', identificationField: 'name', pattern: 'DELETE ME' },
    { table: 'projects', identificationField: 'description', pattern: 'DELETE ME' },
    { table: 'projects', identificationField: 'client', pattern: 'DELETE ME' },
    { table: 'company', identificationField: 'name', pattern: 'DELETE ME' },
    { table: 'company', identificationField: 'description', pattern: 'DELETE ME' },
    { table: 'contacts', identificationField: 'first_name', pattern: 'DELETE ME' },
    { table: 'contacts', identificationField: 'last_name', pattern: 'DELETE ME' },
    { table: 'contacts', identificationField: 'notes', pattern: 'DELETE ME' },
    { table: 'rfp', identificationField: 'name', pattern: 'DELETE ME' },
    { table: 'rfp', identificationField: 'description', pattern: 'DELETE ME' }
  ]

  /**
   * Clean up all test data across all tables
   */
  static async cleanupAllTestData(): Promise<CleanupReport> {
    const startTime = performance.now()
    const report: CleanupReport = {
      timestamp: new Date().toISOString(),
      deletedCounts: { projects: 0, companies: 0, contacts: 0, proposals: 0 },
      errors: [],
      totalDeleted: 0,
      duration: 0
    }

    try {
      console.log('🧹 Starting comprehensive test data cleanup...')

      // Clean projects
      report.deletedCounts.projects = await this.cleanupProjects()
      console.log(`✅ Cleaned ${report.deletedCounts.projects} test projects`)

      // Clean companies
      report.deletedCounts.companies = await this.cleanupCompanies()
      console.log(`✅ Cleaned ${report.deletedCounts.companies} test companies`)

      // Clean contacts
      report.deletedCounts.contacts = await this.cleanupContacts()
      console.log(`✅ Cleaned ${report.deletedCounts.contacts} test contacts`)

      // Clean proposals/RFPs
      report.deletedCounts.proposals = await this.cleanupProposals()
      console.log(`✅ Cleaned ${report.deletedCounts.proposals} test proposals`)

      report.totalDeleted = Object.values(report.deletedCounts).reduce((sum, count) => sum + count, 0)
      
    } catch (error) {
      const errorMessage = `Cleanup failed: ${error instanceof Error ? error.message : String(error)}`
      report.errors.push(errorMessage)
      console.error('❌', errorMessage)
    }

    report.duration = performance.now() - startTime
    console.log(`🏁 Cleanup completed in ${report.duration.toFixed(2)}ms`)
    console.log(`📊 Total records deleted: ${report.totalDeleted}`)

    return report
  }

  /**
   * Clean up test projects
   */
  static async cleanupProjects(): Promise<number> {
    if (typeof mcp__surrealdb_emittiv__query !== 'function') {
      throw new Error('SurrealDB MCP is not available. Ensure MCP server is running.')
    }

    const queries = [
      `DELETE FROM projects WHERE name CONTAINS "${this.DELETE_ME_PATTERN}"`,
      `DELETE FROM projects WHERE description CONTAINS "${this.DELETE_ME_PATTERN}"`,
      `DELETE FROM projects WHERE client CONTAINS "${this.DELETE_ME_PATTERN}"`
    ]

    let totalDeleted = 0
    for (const query of queries) {
      try {
        const result = await mcp__surrealdb_emittiv__query({ query_string: query })
        if (result && typeof result === 'object' && 'length' in result) {
          totalDeleted += (result as any[]).length
        }
      } catch (error) {
        console.error(`Failed to execute query: ${query}`, error)
        throw error
      }
    }

    return totalDeleted
  }

  /**
   * Clean up test companies
   */
  static async cleanupCompanies(): Promise<number> {
    if (typeof mcp__surrealdb_emittiv__query !== 'function') {
      throw new Error('SurrealDB MCP is not available. Ensure MCP server is running.')
    }

    const queries = [
      `DELETE FROM company WHERE name CONTAINS "${this.DELETE_ME_PATTERN}"`,
      `DELETE FROM company WHERE description CONTAINS "${this.DELETE_ME_PATTERN}"`,
      `DELETE FROM company WHERE email CONTAINS "delete-me-testing.com"`
    ]

    let totalDeleted = 0
    for (const query of queries) {
      try {
        const result = await mcp__surrealdb_emittiv__query({ query_string: query })
        if (result && typeof result === 'object' && 'length' in result) {
          totalDeleted += (result as any[]).length
        }
      } catch (error) {
        console.error(`Failed to execute query: ${query}`, error)
        throw error
      }
    }

    return totalDeleted
  }

  /**
   * Clean up test contacts
   */
  static async cleanupContacts(): Promise<number> {
    if (typeof mcp__surrealdb_emittiv__query !== 'function') {
      throw new Error('SurrealDB MCP is not available. Ensure MCP server is running.')
    }

    const queries = [
      `DELETE FROM contacts WHERE first_name CONTAINS "${this.DELETE_ME_PATTERN}"`,
      `DELETE FROM contacts WHERE last_name CONTAINS "${this.DELETE_ME_PATTERN}"`,
      `DELETE FROM contacts WHERE notes CONTAINS "${this.DELETE_ME_PATTERN}"`,
      `DELETE FROM contacts WHERE email CONTAINS "delete-me-testing.com"`
    ]

    let totalDeleted = 0
    for (const query of queries) {
      try {
        const result = await mcp__surrealdb_emittiv__query({ query_string: query })
        if (result && typeof result === 'object' && 'length' in result) {
          totalDeleted += (result as any[]).length
        }
      } catch (error) {
        console.error(`Failed to execute query: ${query}`, error)
        throw error
      }
    }

    return totalDeleted
  }

  /**
   * Clean up test proposals/RFPs
   */
  static async cleanupProposals(): Promise<number> {
    if (typeof mcp__surrealdb_emittiv__query !== 'function') {
      throw new Error('SurrealDB MCP is not available. Ensure MCP server is running.')
    }

    const queries = [
      `DELETE FROM rfp WHERE name CONTAINS "${this.DELETE_ME_PATTERN}"`,
      `DELETE FROM rfp WHERE description CONTAINS "${this.DELETE_ME_PATTERN}"`
    ]

    let totalDeleted = 0
    for (const query of queries) {
      try {
        const result = await mcp__surrealdb_emittiv__query({ query_string: query })
        if (result && typeof result === 'object' && 'length' in result) {
          totalDeleted += (result as any[]).length
        }
      } catch (error) {
        console.error(`Failed to execute query: ${query}`, error)
        throw error
      }
    }

    return totalDeleted
  }

  /**
   * List all test data without deleting it
   */
  static async listAllTestData(): Promise<{
    projects: any[]
    companies: any[]  
    contacts: any[]
    proposals: any[]
  }> {
    if (typeof mcp__surrealdb_emittiv__query !== 'function') {
      throw new Error('SurrealDB MCP is not available. Ensure MCP server is running.')
    }

    const results = {
      projects: [],
      companies: [],
      contacts: [],
      proposals: []
    }

    try {
      // List test projects
      const projectQueries = [
        `SELECT * FROM projects WHERE name CONTAINS "${this.DELETE_ME_PATTERN}"`,
        `SELECT * FROM projects WHERE description CONTAINS "${this.DELETE_ME_PATTERN}"`,
        `SELECT * FROM projects WHERE client CONTAINS "${this.DELETE_ME_PATTERN}"`
      ]
      
      for (const query of projectQueries) {
        const result = await mcp__surrealdb_emittiv__query({ query_string: query })
        if (Array.isArray(result)) {
          results.projects.push(...result)
        }
      }

      // List test companies
      const companyQueries = [
        `SELECT * FROM company WHERE name CONTAINS "${this.DELETE_ME_PATTERN}"`,
        `SELECT * FROM company WHERE description CONTAINS "${this.DELETE_ME_PATTERN}"`
      ]
      
      for (const query of companyQueries) {
        const result = await mcp__surrealdb_emittiv__query({ query_string: query })
        if (Array.isArray(result)) {
          results.companies.push(...result)
        }
      }

      // List test contacts
      const contactQueries = [
        `SELECT * FROM contacts WHERE first_name CONTAINS "${this.DELETE_ME_PATTERN}"`,
        `SELECT * FROM contacts WHERE last_name CONTAINS "${this.DELETE_ME_PATTERN}"`,
        `SELECT * FROM contacts WHERE notes CONTAINS "${this.DELETE_ME_PATTERN}"`
      ]
      
      for (const query of contactQueries) {
        const result = await mcp__surrealdb_emittiv__query({ query_string: query })
        if (Array.isArray(result)) {
          results.contacts.push(...result)
        }
      }

      // List test proposals
      const proposalQueries = [
        `SELECT * FROM rfp WHERE name CONTAINS "${this.DELETE_ME_PATTERN}"`,
        `SELECT * FROM rfp WHERE description CONTAINS "${this.DELETE_ME_PATTERN}"`
      ]
      
      for (const query of proposalQueries) {
        const result = await mcp__surrealdb_emittiv__query({ query_string: query })
        if (Array.isArray(result)) {
          results.proposals.push(...result)
        }
      }

    } catch (error) {
      console.error('Failed to list test data:', error)
      throw error
    }

    // Remove duplicates based on ID
    results.projects = this.removeDuplicatesById(results.projects)
    results.companies = this.removeDuplicatesById(results.companies)
    results.contacts = this.removeDuplicatesById(results.contacts)
    results.proposals = this.removeDuplicatesById(results.proposals)

    return results
  }

  /**
   * Verify that all test data has been cleaned up
   */
  static async verifyCleanup(): Promise<{ isClean: boolean; remainingTestData: any }> {
    const remainingTestData = await this.listAllTestData()
    
    const totalRemaining = 
      remainingTestData.projects.length +
      remainingTestData.companies.length +
      remainingTestData.contacts.length +
      remainingTestData.proposals.length

    const isClean = totalRemaining === 0

    if (isClean) {
      console.log('✅ Database is clean - no test data found')
    } else {
      console.log(`⚠️ Found ${totalRemaining} remaining test records:`)
      console.log(`  Projects: ${remainingTestData.projects.length}`)
      console.log(`  Companies: ${remainingTestData.companies.length}`)
      console.log(`  Contacts: ${remainingTestData.contacts.length}`)
      console.log(`  Proposals: ${remainingTestData.proposals.length}`)
    }

    return { isClean, remainingTestData }
  }

  /**
   * Clean up test data by specific timestamp/test run
   */
  static async cleanupByTimestamp(timestamp: string): Promise<CleanupReport> {
    const startTime = performance.now()
    const report: CleanupReport = {
      timestamp: new Date().toISOString(),
      deletedCounts: { projects: 0, companies: 0, contacts: 0, proposals: 0 },
      errors: [],
      totalDeleted: 0,
      duration: 0
    }

    if (typeof mcp__surrealdb_emittiv__query !== 'function') {
      report.errors.push('SurrealDB MCP is not available')
      return report
    }

    try {
      console.log(`🧹 Cleaning up test data for timestamp: ${timestamp}`)

      const queries = [
        `DELETE FROM projects WHERE name CONTAINS "${timestamp}" AND name CONTAINS "${this.DELETE_ME_PATTERN}"`,
        `DELETE FROM company WHERE name CONTAINS "${timestamp}" AND name CONTAINS "${this.DELETE_ME_PATTERN}"`,
        `DELETE FROM contacts WHERE first_name CONTAINS "${timestamp}" AND first_name CONTAINS "${this.DELETE_ME_PATTERN}"`,
        `DELETE FROM rfp WHERE name CONTAINS "${timestamp}" AND name CONTAINS "${this.DELETE_ME_PATTERN}"`
      ]

      for (const query of queries) {
        try {
          const result = await mcp__surrealdb_emittiv__query({ query_string: query })
          if (result && typeof result === 'object' && 'length' in result) {
            report.totalDeleted += (result as any[]).length
          }
        } catch (error) {
          const errorMessage = `Failed to execute query: ${query} - ${error}`
          report.errors.push(errorMessage)
          console.error('❌', errorMessage)
        }
      }

    } catch (error) {
      const errorMessage = `Timestamp cleanup failed: ${error instanceof Error ? error.message : String(error)}`
      report.errors.push(errorMessage)
      console.error('❌', errorMessage)
    }

    report.duration = performance.now() - startTime
    console.log(`🏁 Timestamp cleanup completed in ${report.duration.toFixed(2)}ms`)
    console.log(`📊 Total records deleted: ${report.totalDeleted}`)

    return report
  }

  /**
   * Emergency cleanup - removes ALL test data immediately
   */
  static async emergencyCleanup(): Promise<CleanupReport> {
    console.log('🚨 EMERGENCY CLEANUP INITIATED')
    console.log('⚠️  This will remove ALL test data with "DELETE ME" pattern')
    
    const report = await this.cleanupAllTestData()
    
    // Double-check with verification
    const verification = await this.verifyCleanup()
    if (!verification.isClean) {
      console.log('🔄 Some test data remains, performing second cleanup pass...')
      const secondPass = await this.cleanupAllTestData()
      report.totalDeleted += secondPass.totalDeleted
      report.deletedCounts.projects += secondPass.deletedCounts.projects
      report.deletedCounts.companies += secondPass.deletedCounts.companies
      report.deletedCounts.contacts += secondPass.deletedCounts.contacts
      report.deletedCounts.proposals += secondPass.deletedCounts.proposals
    }

    console.log('🚨 EMERGENCY CLEANUP COMPLETED')
    return report
  }

  /**
   * Remove duplicate records based on ID field
   */
  private static removeDuplicatesById(records: any[]): any[] {
    const seen = new Set()
    return records.filter(record => {
      const id = record.id || record.Id
      if (seen.has(id)) {
        return false
      }
      seen.add(id)
      return true
    })
  }

  /**
   * Generate cleanup report for documentation
   */
  static generateCleanupReport(report: CleanupReport): string {
    const lines = [
      '# Test Data Cleanup Report',
      '',
      `**Timestamp**: ${report.timestamp}`,
      `**Duration**: ${report.duration.toFixed(2)}ms`,
      `**Total Deleted**: ${report.totalDeleted} records`,
      '',
      '## Deleted Records by Type',
      '',
      `- **Projects**: ${report.deletedCounts.projects}`,
      `- **Companies**: ${report.deletedCounts.companies}`, 
      `- **Contacts**: ${report.deletedCounts.contacts}`,
      `- **Proposals**: ${report.deletedCounts.proposals}`,
      ''
    ]

    if (report.errors.length > 0) {
      lines.push('## Errors Encountered', '')
      report.errors.forEach(error => {
        lines.push(`- ${error}`)
      })
      lines.push('')
    }

    lines.push('## Status')
    if (report.errors.length === 0) {
      lines.push('✅ **Cleanup completed successfully**')
    } else {
      lines.push('⚠️ **Cleanup completed with errors**')
    }

    return lines.join('\n')
  }
}

/**
 * Convenience functions for common cleanup operations
 */
export const cleanupAllTestData = () => SafeTestDataCleaner.cleanupAllTestData()
export const cleanupProjects = () => SafeTestDataCleaner.cleanupProjects()
export const cleanupCompanies = () => SafeTestDataCleaner.cleanupCompanies()
export const cleanupContacts = () => SafeTestDataCleaner.cleanupContacts()
export const cleanupProposals = () => SafeTestDataCleaner.cleanupProposals()
export const listAllTestData = () => SafeTestDataCleaner.listAllTestData()
export const verifyCleanup = () => SafeTestDataCleaner.verifyCleanup()
export const cleanupByTimestamp = (timestamp: string) => SafeTestDataCleaner.cleanupByTimestamp(timestamp)
export const emergencyCleanup = () => SafeTestDataCleaner.emergencyCleanup()

/**
 * Default export for direct usage
 */
export default SafeTestDataCleaner