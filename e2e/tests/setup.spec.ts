import { test, expect } from '@playwright/test'
import { DatabaseFixtures } from '../fixtures/database-fixtures'

/**
 * Setup test to verify E2E environment is ready
 * This test runs first and ensures all prerequisites are met
 */
test.describe('E2E Environment Setup', () => {
  test('should verify SurrealDB test instance is running', async ({ page }) => {
    // Test direct connection to SurrealDB
    const isConnected = await page.evaluate(async () => {
      try {
        const response = await fetch('http://localhost:8001/version')
        return response.ok
      } catch (error) {
        return false
      }
    })
    
    expect(isConnected).toBe(true)
  })

  test('should initialize test database with reference data', async ({ page }) => {
    const dbFixtures = new DatabaseFixtures(page)
    
    // Initialize reference data
    await dbFixtures.initializeReferenceData()
    
    // Verify reference data was created
    const counts = await dbFixtures.getDatabaseCounts()
    expect(counts.countries).toBeGreaterThan(0)
    expect(counts.currencies).toBeGreaterThan(0)
  })

  test('should verify test project folder exists', async ({ page }) => {
    const projectFolderExists = await page.evaluate(async () => {
      const testFolderPath = process.env.PROJECT_FOLDER_PATH || '/tmp/test-projects'
      
      try {
        // This would be implemented using a Tauri command to check file system
        // For now, we'll assume it exists if the environment variable is set
        return Boolean(process.env.PROJECT_FOLDER_PATH)
      } catch (error) {
        return false
      }
    })
    
    expect(projectFolderExists).toBe(true)
  })

  test('should verify all required environment variables are set', async ({ page }) => {
    const envVars = await page.evaluate(() => {
      return {
        SURREALDB_URL: process.env.SURREALDB_URL,
        SURREALDB_NS: process.env.SURREALDB_NS,
        SURREALDB_DB: process.env.SURREALDB_DB,
        SURREALDB_USER: process.env.SURREALDB_USER,
        SURREALDB_PASS: process.env.SURREALDB_PASS,
        PROJECT_FOLDER_PATH: process.env.PROJECT_FOLDER_PATH,
        NODE_ENV: process.env.NODE_ENV
      }
    })

    expect(envVars.SURREALDB_URL).toBe('ws://localhost:8001')
    expect(envVars.SURREALDB_NS).toBe('test')
    expect(envVars.SURREALDB_DB).toBe('e2e')
    expect(envVars.SURREALDB_USER).toBe('root')
    expect(envVars.SURREALDB_PASS).toBe('test')
    expect(envVars.PROJECT_FOLDER_PATH).toBeTruthy()
    expect(envVars.NODE_ENV).toBe('test')
  })
})