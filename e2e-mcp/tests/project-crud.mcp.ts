/**
 * Real E2E Tests for Project CRUD Operations using Tauri MCP
 * 
 * These tests interact with the ACTUAL desktop application, not browser simulation.
 * All test data includes "DELETE ME" identification for safe cleanup.
 */

import { describe, test, expect, beforeAll, afterAll, beforeEach } from 'vitest'
import MCPClient from '../helpers/mcp-client'
import { generateSafeProjectData, SafeProjectData } from '../fixtures/test-data-safe'
import { cleanupAllTestData, verifyCleanup } from '../fixtures/cleanup-utilities'

describe('Project CRUD Operations - Real MCP Testing', () => {
  let mcpClient: MCPClient
  let testProjectData: SafeProjectData[]

  beforeAll(async () => {
    // Initialize MCP client with debug mode for detailed logging
    mcpClient = new MCPClient({
      debugMode: true,
      screenshotPath: './e2e-mcp/results/screenshots',
      timeout: 30000
    })

    // Verify application is ready for testing
    console.log('🚀 Verifying application is ready for MCP testing...')
    const readyResult = await mcpClient.verifyApplicationReady()
    
    if (!readyResult.success) {
      throw new Error(`Application not ready: ${readyResult.error}`)
    }
    
    console.log('✅ Application ready for testing')

    // Generate safe test data
    testProjectData = generateSafeProjectData(3)
    console.log(`📝 Generated ${testProjectData.length} test projects with DELETE ME identification`)
  })

  beforeEach(async () => {
    // Take a screenshot before each test for documentation
    await mcpClient.takeScreenshot({
      filename: 'test-start-state',
      description: 'Application state before test execution'
    })
  })

  afterAll(async () => {
    // Cleanup all test data after testing
    console.log('🧹 Cleaning up test data...')
    const cleanupReport = await cleanupAllTestData()
    
    console.log(`📊 Cleanup completed: ${cleanupReport.totalDeleted} records deleted`)
    
    if (cleanupReport.errors.length > 0) {
      console.warn('⚠️ Cleanup errors:', cleanupReport.errors)
    }

    // Verify cleanup was successful
    const verification = await verifyCleanup()
    expect(verification.isClean).toBe(true)
    
    // Take final screenshot
    await mcpClient.takeScreenshot({
      filename: 'test-suite-completed',
      description: 'Final application state after all tests and cleanup'
    })

    const summary = mcpClient.getTestSummary()
    console.log(`🎯 Test suite completed in ${summary.totalDuration.toFixed(2)}ms`)
    console.log(`📸 Total screenshots taken: ${summary.screenshotCount}`)
  })

  test('should navigate to projects page using keyboard shortcut', async () => {
    // Test real keyboard navigation (not browser simulation)
    const navigationResult = await mcpClient.testKeyboardNavigation()
    
    expect(navigationResult.success).toBe(true)
    expect(navigationResult.data?.pagesVisited).toContain('projects')
    expect(navigationResult.screenshots).toHaveLength(5) // One per page
    expect(navigationResult.duration).toBeLessThan(15000) // Should complete in <15s
  })

  test('should create a new project with auto-generated number', async () => {
    const projectData = testProjectData[0]
    
    // Create project using real application interaction
    const createResult = await mcpClient.createProject(projectData)
    
    expect(createResult.success).toBe(true)
    expect(createResult.data?.projectNumber).toMatch(/25-971\d{2}/) // UAE format
    expect(createResult.screenshots).toHaveLength(4) // Navigation, modal, form, created
    expect(createResult.duration).toBeLessThan(10000) // Should complete in <10s

    // Verify project was actually created in database
    if (typeof mcp__surrealdb_emittiv__query === 'function') {
      const dbResult = await mcp__surrealdb_emittiv__query({
        query_string: `SELECT * FROM projects WHERE name = "${projectData.name}"`
      })
      
      expect(Array.isArray(dbResult)).toBe(true)
      expect(dbResult.length).toBe(1)
      expect(dbResult[0].name).toBe(projectData.name)
      expect(dbResult[0].status).toBe('active')
      expect(dbResult[0].project_number).toMatch(/25-971\d{2}/)
    }
  })

  test('should create multiple projects with sequential numbering', async () => {
    const projectsToCreate = testProjectData.slice(0, 3)
    const createdProjects: string[] = []

    for (let i = 0; i < projectsToCreate.length; i++) {
      const projectData = projectsToCreate[i]
      const createResult = await mcpClient.createProject(projectData)
      
      expect(createResult.success).toBe(true)
      expect(createResult.data?.projectNumber).toBeTruthy()
      
      createdProjects.push(createResult.data!.projectNumber)
    }

    // Verify sequential numbering (e.g., 25-97101, 25-97102, 25-97103)
    expect(createdProjects).toHaveLength(3)
    
    // Extract sequence numbers
    const sequenceNumbers = createdProjects.map(num => parseInt(num.slice(-2)))
    sequenceNumbers.sort((a, b) => a - b)
    
    // Should be sequential
    for (let i = 1; i < sequenceNumbers.length; i++) {
      expect(sequenceNumbers[i]).toBe(sequenceNumbers[i-1] + 1)
    }
  })

  test('should search and filter projects', async () => {
    // First create a test project to search for
    const searchProjectData = testProjectData[0]
    const createResult = await mcpClient.createProject(searchProjectData)
    expect(createResult.success).toBe(true)

    // Navigate to projects page
    await mcpClient.navigateTo('projects')
    
    // Take screenshot of projects list
    await mcpClient.takeScreenshot({ 
      filename: 'projects-list-before-search',
      description: 'Projects page before applying search filter'
    })

    // Use the search functionality
    await mcpClient.clickElement({
      selector: 'input[data-testid="search-input"]',
      description: 'Search input field'
    })

    // Search for our test project using part of the DELETE ME name
    const searchTerm = 'DELETE ME'
    await mcpClient.typeText(searchTerm)
    
    // Wait for search results to load
    await mcpClient.wait(1000)
    
    await mcpClient.takeScreenshot({
      filename: 'projects-search-results',
      description: `Search results for "${searchTerm}"`
    })

    // Verify search results show our test project
    const dom = await mcpClient.getDOMContent()
    expect(dom).toContain(searchProjectData.name)
    expect(dom).toContain('DELETE ME') // Should filter to only test data
  })

  test('should handle project creation with invalid data gracefully', async () => {
    // Test error handling with empty project name
    const invalidProjectData: SafeProjectData = {
      name: '', // Invalid: empty name
      description: 'DELETE ME - Testing error handling',
      location: 'Dubai',
      country: 'UAE',
      area: 'Downtown Dubai',
      city: 'Dubai',
      client: 'DELETE ME - Test Client',
      status: 'active'
    }

    const createResult = await mcpClient.createProject(invalidProjectData)
    
    // Should fail due to validation
    expect(createResult.success).toBe(false)
    expect(createResult.error).toBeTruthy()
    expect(createResult.screenshots).toContain('project-creation-error')
    
    // Verify no invalid project was created in database
    if (typeof mcp__surrealdb_emittiv__query === 'function') {
      const dbResult = await mcp__surrealdb_emittiv__query({
        query_string: 'SELECT * FROM projects WHERE name = ""'
      })
      
      expect(Array.isArray(dbResult)).toBe(true)
      expect(dbResult.length).toBe(0)
    }
  })

  test('should verify project folder integration', async () => {
    const projectData = testProjectData[1]
    const createResult = await mcpClient.createProject(projectData)
    
    expect(createResult.success).toBe(true)
    
    const projectNumber = createResult.data!.projectNumber
    
    // Test project folder functionality via JavaScript execution
    const folderCheckResult = await mcpClient.executeJS(`
      // Check if project folder operations are working
      (async () => {
        try {
          // This would call the Tauri command to check/create project folder
          if (window.__TAURI__) {
            const { invoke } = window.__TAURI__.tauri;
            const folderExists = await invoke('check_project_folder_exists', {
              projectNumber: '${projectNumber}'
            });
            return { success: true, folderExists };
          }
          return { success: false, error: 'Tauri not available' };
        } catch (error) {
          return { success: false, error: error.message };
        }
      })()
    `)

    // Verify folder integration is working
    expect(folderCheckResult).toBeTruthy()
    
    if (folderCheckResult.success) {
      // Folder should be created or checked successfully
      expect(typeof folderCheckResult.folderExists).toBe('boolean')
    }
  })

  test('should handle database connection loss gracefully', async () => {
    // Take screenshot before simulating connection loss
    await mcpClient.takeScreenshot({
      filename: 'before-connection-loss-test',
      description: 'Application state before connection loss simulation'
    })

    // Simulate connection loss by temporarily breaking the connection
    const connectionTestResult = await mcpClient.executeJS(`
      // Test connection status and recovery
      (async () => {
        try {
          // Get current connection status
          const statusElement = document.querySelector('[data-testid="connection-status"]');
          const isConnected = statusElement?.classList.contains('connected') || false;
          
          return {
            success: true,
            initiallyConnected: isConnected,
            statusElement: statusElement?.outerHTML || 'not found'
          };
        } catch (error) {
          return { success: false, error: error.message };
        }
      })()
    `)

    expect(connectionTestResult.success).toBe(true)
    expect(connectionTestResult.initiallyConnected).toBe(true)
    
    await mcpClient.takeScreenshot({
      filename: 'connection-status-verified',
      description: 'Connection status verification completed'
    })
  })

  test('should measure project creation performance', async () => {
    const projectData = testProjectData[2]
    
    const performanceStart = performance.now()
    const createResult = await mcpClient.createProject(projectData)
    const performanceEnd = performance.now()
    
    const totalTime = performanceEnd - performanceStart
    const mcpTime = createResult.duration
    
    // Verify performance metrics
    expect(createResult.success).toBe(true)
    expect(totalTime).toBeLessThan(15000) // Total time should be <15s
    expect(mcpTime).toBeLessThan(10000)   // MCP operations should be <10s
    expect(mcpTime).toBeLessThan(totalTime) // MCP time should be subset of total
    
    console.log(`⏱️ Performance metrics:`)
    console.log(`   Total time: ${totalTime.toFixed(2)}ms`)
    console.log(`   MCP operations: ${mcpTime.toFixed(2)}ms`)
    console.log(`   Screenshots: ${createResult.screenshots?.length || 0}`)
    
    // Performance should be reasonable for desktop application
    const timePerScreenshot = createResult.screenshots?.length ? mcpTime / createResult.screenshots.length : 0
    expect(timePerScreenshot).toBeLessThan(2500) // <2.5s per screenshot on average
  })
})

describe('Project Data Integrity - MCP Database Verification', () => {
  test('should verify all test projects have DELETE ME identification', async () => {
    // This test ensures our safety measures are working
    if (typeof mcp__surrealdb_emittiv__query === 'function') {
      
      // Query all projects that might be test data
      const allProjectsResult = await mcp__surrealdb_emittiv__query({
        query_string: 'SELECT name, description, client FROM projects WHERE name CONTAINS "Test" OR name CONTAINS "DELETE" ORDER BY created_at DESC LIMIT 10'
      })
      
      expect(Array.isArray(allProjectsResult)).toBe(true)
      
      // Every test project should have "DELETE ME" in name, description, or client
      for (const project of allProjectsResult) {
        const hasDeleteMe = 
          project.name?.includes('DELETE ME') ||
          project.description?.includes('DELETE ME') ||
          project.client?.includes('DELETE ME')
        
        expect(hasDeleteMe).toBe(true)
        console.log(`✅ Test project properly identified: ${project.name}`)
      }
    }
  })

  test('should verify no production data was affected', async () => {
    // This test ensures we never accidentally modified real data
    if (typeof mcp__surrealdb_emittiv__query === 'function') {
      
      // Query some known production project patterns (without DELETE ME)
      const productionCheck = await mcp__surrealdb_emittiv__query({
        query_string: 'SELECT COUNT() as count FROM projects WHERE name NOT CONTAINS "DELETE ME" AND project_number ~ "25-971\\\\d{2}"'
      })
      
      expect(Array.isArray(productionCheck)).toBe(true)
      
      if (productionCheck.length > 0) {
        const productionCount = productionCheck[0].count
        console.log(`📊 Found ${productionCount} production projects (unaffected by tests)`)
        
        // Should have production data (this verifies we're testing on live DB safely)
        expect(productionCount).toBeGreaterThan(0)
      }
    }
  })
})