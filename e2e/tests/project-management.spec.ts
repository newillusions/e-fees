import { test, expect } from '@playwright/test'
import { TauriApp, TauriTestHelpers } from '../helpers/tauri-helpers'
import { NavigationPage, ProjectsPage, ProjectModal } from '../helpers/page-objects'
import { DatabaseFixtures } from '../fixtures/database-fixtures'

/**
 * Project management workflow tests
 * Covers project CRUD operations, auto-number generation, and file system integration
 */
test.describe('Project Management Workflows', () => {
  let tauriApp: TauriApp
  let navigationPage: NavigationPage
  let projectsPage: ProjectsPage
  let projectModal: ProjectModal
  let dbFixtures: DatabaseFixtures

  test.beforeAll(async () => {
    tauriApp = new TauriApp(process.env.E2E_DEV_MODE === 'true')
  })

  test.beforeEach(async ({ page }) => {
    navigationPage = new NavigationPage(page)
    projectsPage = new ProjectsPage(page)
    projectModal = new ProjectModal(page)
    dbFixtures = new DatabaseFixtures(page)
    
    // Setup clean database state with reference data
    await dbFixtures.clearAllData()
    await dbFixtures.initializeReferenceData()
  })

  test.afterAll(async () => {
    if (tauriApp) {
      await tauriApp.close()
    }
  })

  test('should create project with auto-generated number', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const projectsPage = new ProjectsPage(page)
    const projectModal = new ProjectModal(page)
    
    // Navigate to projects page
    await navPage.navigateToProjects()
    
    // Click new project button
    await projectsPage.clickNewProject()
    
    // Verify modal opens
    await projectModal.verifyModalVisible()
    
    // Fill project form
    await projectModal.fillProjectForm({
      name: 'Dubai Museum of Future Heritage',
      shortName: 'DMFH',
      area: 'Business Bay',
      city: 'Dubai',
      country: 'United Arab Emirates'
    })
    
    // Verify project number is auto-generated with correct format
    await projectModal.verifyProjectNumberFormat(/^25-971\d{2}$/)
    
    const projectNumber = await projectModal.getGeneratedProjectNumber()
    expect(projectNumber).toMatch(/^25-971\d{2}$/)
    
    // Save project
    await projectModal.saveProject()
    
    // Verify project appears in list
    await projectsPage.verifyProjectExists('Dubai Museum of Future Heritage')
    
    // Verify project count increased
    const count = await projectsPage.getProjectsCount()
    expect(count).toBe(1)
    
    await page.close()
  })

  test('should create multiple projects with sequential numbering', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const projectsPage = new ProjectsPage(page)
    const projectModal = new ProjectModal(page)
    
    await navPage.navigateToProjects()
    
    const projects = [
      {
        name: 'Dubai Museum Project',
        shortName: 'DMP',
        area: 'Business Bay',
        city: 'Dubai',
        country: 'United Arab Emirates',
        expectedPattern: /^25-97101$/
      },
      {
        name: 'Dubai Cultural Center',
        shortName: 'DCC',
        area: 'DIFC',
        city: 'Dubai',
        country: 'United Arab Emirates',
        expectedPattern: /^25-97102$/
      },
      {
        name: 'Riyadh Innovation Hub',
        shortName: 'RIH',
        area: 'King Fahd District',
        city: 'Riyadh',
        country: 'Saudi Arabia',
        expectedPattern: /^25-96601$/
      }
    ]
    
    const createdNumbers: string[] = []
    
    for (const project of projects) {
      await projectsPage.clickNewProject()
      await projectModal.fillProjectForm(project)
      
      const projectNumber = await projectModal.getGeneratedProjectNumber()
      expect(projectNumber).toMatch(project.expectedPattern)
      createdNumbers.push(projectNumber)
      
      await projectModal.saveProject()
      await projectsPage.verifyProjectExists(project.name)
    }
    
    // Verify all numbers are unique
    const uniqueNumbers = new Set(createdNumbers)
    expect(uniqueNumbers.size).toBe(createdNumbers.length)
    
    // Verify total count
    const totalCount = await projectsPage.getProjectsCount()
    expect(totalCount).toBe(3)
    
    await page.close()
  })

  test('should handle different countries correctly', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const projectsPage = new ProjectsPage(page)
    const projectModal = new ProjectModal(page)
    
    await navPage.navigateToProjects()
    
    const countryTests = [
      { country: 'United Arab Emirates', expectedCode: '971' },
      { country: 'Saudi Arabia', expectedCode: '966' },
      { country: 'Qatar', expectedCode: '974' }
    ]
    
    for (const test of countryTests) {
      await projectsPage.clickNewProject()
      await projectModal.fillProjectForm({
        name: `Test Project ${test.country}`,
        shortName: 'TP',
        area: 'Test Area',
        city: 'Test City',
        country: test.country
      })
      
      const projectNumber = await projectModal.getGeneratedProjectNumber()
      expect(projectNumber).toContain(test.expectedCode)
      
      await projectModal.saveProject()
    }
    
    await page.close()
  })

  test('should search and filter projects', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const dbFixtures = new DatabaseFixtures(page)
    
    // Seed database with test projects
    await dbFixtures.seedCompleteData()
    
    const navPage = new NavigationPage(page)
    const projectsPage = new ProjectsPage(page)
    
    await navPage.navigateToProjects()
    
    // Wait for projects to load
    await page.waitForTimeout(1000)
    
    // Test search by name
    await projectsPage.searchProjects('Dubai Museum')
    let count = await projectsPage.getProjectsCount()
    expect(count).toBeGreaterThan(0)
    
    // Test search by location
    await projectsPage.searchProjects('Riyadh')
    count = await projectsPage.getProjectsCount()
    expect(count).toBeGreaterThan(0)
    
    // Test search by project number
    await projectsPage.searchProjects('25-971')
    count = await projectsPage.getProjectsCount()
    expect(count).toBeGreaterThan(0)
    
    // Clear search
    await projectsPage.searchProjects('')
    count = await projectsPage.getProjectsCount()
    expect(count).toBeGreaterThan(2) // Should show all projects
    
    await page.close()
  })

  test('should handle project folder creation', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const projectsPage = new ProjectsPage(page)
    const projectModal = new ProjectModal(page)
    const helpers = new TauriTestHelpers(page)
    
    await navPage.navigateToProjects()
    
    // Create project
    await projectsPage.clickNewProject()
    await projectModal.fillProjectForm({
      name: 'Test Folder Project',
      shortName: 'TFP',
      area: 'Test Area',
      city: 'Dubai',
      country: 'United Arab Emirates'
    })
    
    const projectNumber = await projectModal.getGeneratedProjectNumber()
    await projectModal.saveProject()
    
    // Test file system integration
    const folderExists = await helpers.testFileOperations('Test Folder Project')
    expect(folderExists).toBe(true)
    
    await page.close()
  })

  test('should validate required fields', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const projectsPage = new ProjectsPage(page)
    const projectModal = new ProjectModal(page)
    
    await navPage.navigateToProjects()
    await projectsPage.clickNewProject()
    
    // Try to save without filling required fields
    await page.click('[data-testid="save-project-button"]')
    
    // Verify validation messages appear
    await expect(page.locator('[data-testid="validation-error"]')).toBeVisible()
    
    // Verify modal doesn't close
    await projectModal.verifyModalVisible()
    
    await page.close()
  })

  test('should cancel project creation', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const projectsPage = new ProjectsPage(page)
    const projectModal = new ProjectModal(page)
    
    await navPage.navigateToProjects()
    
    const initialCount = await projectsPage.getProjectsCount()
    
    await projectsPage.clickNewProject()
    await projectModal.fillProjectForm({
      name: 'Cancelled Project',
      shortName: 'CP',
      area: 'Test Area',
      city: 'Dubai',
      country: 'United Arab Emirates'
    })
    
    // Cancel instead of save
    await projectModal.cancelProject()
    
    // Verify project count didn't change
    const finalCount = await projectsPage.getProjectsCount()
    expect(finalCount).toBe(initialCount)
    
    await page.close()
  })

  test('should handle edge cases in project data', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const projectsPage = new ProjectsPage(page)
    const projectModal = new ProjectModal(page)
    
    await navPage.navigateToProjects()
    
    // Test with very long project name
    await projectsPage.clickNewProject()
    await projectModal.fillProjectForm({
      name: 'A'.repeat(100), // Very long name
      shortName: 'LONG',
      area: 'Test Area',
      city: 'Dubai',
      country: 'United Arab Emirates'
    })
    
    const projectNumber = await projectModal.getGeneratedProjectNumber()
    expect(projectNumber).toMatch(/^25-971\d{2}$/)
    
    await projectModal.saveProject()
    
    // Test with special characters
    await projectsPage.clickNewProject()
    await projectModal.fillProjectForm({
      name: 'Project & Design (Phase 1)',
      shortName: 'P&D1',
      area: 'Al-Wasl',
      city: 'Dubai',
      country: 'United Arab Emirates'
    })
    
    await projectModal.saveProject()
    
    await page.close()
  })
})