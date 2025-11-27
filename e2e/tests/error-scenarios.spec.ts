import { test, expect } from '@playwright/test'
import { TauriApp, TauriTestHelpers } from '../helpers/tauri-helpers'
import { NavigationPage, ProjectsPage, ProjectModal } from '../helpers/page-objects'
import { DatabaseFixtures } from '../fixtures/database-fixtures'

/**
 * Error scenario and resilience tests
 * Verifies application handles failures gracefully
 */
test.describe('Error Scenarios and Resilience', () => {
  let tauriApp: TauriApp
  let dbFixtures: DatabaseFixtures

  test.beforeAll(async () => {
    tauriApp = new TauriApp(process.env.E2E_DEV_MODE === 'true')
  })

  test.beforeEach(async ({ page }) => {
    dbFixtures = new DatabaseFixtures(page)
    await dbFixtures.clearAllData()
    await dbFixtures.initializeReferenceData()
  })

  test.afterAll(async () => {
    if (tauriApp) {
      await tauriApp.close()
    }
  })

  test('should handle database connection failure gracefully', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const helpers = new TauriTestHelpers(page)
    
    // Verify initial connection
    await navPage.verifyConnectionStatus('connected')
    
    // Simulate connection loss
    await helpers.simulateConnectionLoss()
    
    // Verify error state handling
    // Note: Implementation would depend on how app handles connection loss
    await page.waitForTimeout(2000)
    
    // Try to perform an operation that requires database
    await navPage.navigateToProjects()
    
    // Verify error handling doesn't crash the app
    await expect(page.locator('[data-testid="app-container"]')).toBeVisible()
    
    await page.close()
  })

  test('should validate form inputs and show error messages', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const projectsPage = new ProjectsPage(page)
    const projectModal = new ProjectModal(page)
    
    await navPage.navigateToProjects()
    await projectsPage.clickNewProject()
    
    // Test required field validation
    await page.click('[data-testid="save-project-button"]')
    
    // Verify validation errors appear
    await expect(page.locator('[data-testid="validation-error"]')).toBeVisible()
    
    // Verify modal stays open
    await expect(page.locator('[data-testid="project-modal"]')).toBeVisible()
    
    // Fill required fields and verify errors clear
    await projectModal.fillProjectForm({
      name: 'Valid Project',
      shortName: 'VP',
      area: 'Test Area',
      city: 'Dubai',
      country: 'United Arab Emirates'
    })
    
    // Validation errors should be gone
    await expect(page.locator('[data-testid="validation-error"]')).not.toBeVisible()
    
    await projectModal.saveProject()
    
    await page.close()
  })

  test('should handle invalid email addresses', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    
    // Seed with a company first
    await dbFixtures.seedMinimalData()
    
    await navPage.navigateToContacts()
    await page.click('[data-testid="new-contact-button"]')
    
    // Fill form with invalid email
    await page.fill('[data-testid="contact-first-name-input"]', 'Test')
    await page.fill('[data-testid="contact-last-name-input"]', 'User')
    await page.fill('[data-testid="contact-email-input"]', 'invalid-email-format')
    await page.fill('[data-testid="contact-phone-input"]', '+971-50-123-4567')
    await page.fill('[data-testid="contact-position-input"]', 'Test Position')
    
    await page.click('[data-testid="save-contact-button"]')
    
    // Verify email validation error
    await expect(page.locator('[data-testid="email-validation-error"]')).toBeVisible()
    
    // Fix email and verify error clears
    await page.fill('[data-testid="contact-email-input"]', 'test@example.com')
    await expect(page.locator('[data-testid="email-validation-error"]')).not.toBeVisible()
    
    await page.close()
  })

  test('should handle extremely long input values', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const projectsPage = new ProjectsPage(page)
    const projectModal = new ProjectModal(page)
    
    await navPage.navigateToProjects()
    await projectsPage.clickNewProject()
    
    // Test with extremely long values
    const longName = 'A'.repeat(500)
    
    await page.fill('[data-testid="project-name-input"]', longName)
    await page.fill('[data-testid="project-short-name-input"]', 'LONG')
    await page.fill('[data-testid="project-area-input"]', 'Test Area')
    await page.fill('[data-testid="project-city-input"]', 'Dubai')
    await page.selectOption('[data-testid="project-country-select"]', { label: 'United Arab Emirates' })
    
    // Try to save
    await page.click('[data-testid="save-project-button"]')
    
    // Should either accept the long name or show appropriate validation
    // Verify app doesn't crash
    await expect(page.locator('[data-testid="app-container"]')).toBeVisible()
    
    await page.close()
  })

  test('should handle special characters in input fields', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const projectsPage = new ProjectsPage(page)
    const projectModal = new ProjectModal(page)
    
    await navPage.navigateToProjects()
    await projectsPage.clickNewProject()
    
    // Test with special characters and unicode
    await projectModal.fillProjectForm({
      name: 'Project & Design (Phase 1) — الإمارات',
      shortName: 'P&D1',
      area: 'Al-Wasl — الوصل',
      city: 'Dubai — دبي',
      country: 'United Arab Emirates'
    })
    
    await projectModal.saveProject()
    
    // Verify project was created with special characters
    await projectsPage.verifyProjectExists('Project & Design (Phase 1) — الإمارات')
    
    await page.close()
  })

  test('should handle duplicate data appropriately', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    
    // Seed with existing company
    await dbFixtures.seedMinimalData()
    
    await navPage.navigateToCompanies()
    await page.click('[data-testid="new-company-button"]')
    
    // Try to create company with duplicate abbreviation
    await page.fill('[data-testid="company-name-input"]', 'Duplicate Company')
    await page.fill('[data-testid="company-short-name-input"]', 'DC')
    await page.fill('[data-testid="company-abbreviation-input"]', 'EEC') // Duplicate
    await page.fill('[data-testid="company-city-input"]', 'Abu Dhabi')
    await page.fill('[data-testid="company-country-input"]', 'United Arab Emirates')
    
    await page.click('[data-testid="save-company-button"]')
    
    // Should show appropriate error for duplicate
    await expect(page.locator('[data-testid="duplicate-error"]')).toBeVisible()
    
    await page.close()
  })

  test('should handle rapid form submissions', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const projectsPage = new ProjectsPage(page)
    const projectModal = new ProjectModal(page)
    
    await navPage.navigateToProjects()
    await projectsPage.clickNewProject()
    
    await projectModal.fillProjectForm({
      name: 'Rapid Submit Test',
      shortName: 'RST',
      area: 'Test Area',
      city: 'Dubai',
      country: 'United Arab Emirates'
    })
    
    // Rapidly click save button multiple times
    await Promise.all([
      page.click('[data-testid="save-project-button"]'),
      page.click('[data-testid="save-project-button"]'),
      page.click('[data-testid="save-project-button"]')
    ])
    
    // Should only create one project
    await page.waitForTimeout(2000)
    const count = await projectsPage.getProjectsCount()
    expect(count).toBe(1)
    
    await page.close()
  })

  test('should handle empty search results gracefully', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const projectsPage = new ProjectsPage(page)
    
    await navPage.navigateToProjects()
    
    // Search for non-existent project
    await projectsPage.searchProjects('NonExistentProject12345')
    
    // Verify empty state is shown
    await expect(page.locator('[data-testid="empty-state"]')).toBeVisible()
    await expect(page.locator('[data-testid="no-results-message"]')).toBeVisible()
    
    // Clear search and verify full list returns
    await projectsPage.searchProjects('')
    await expect(page.locator('[data-testid="empty-state"]')).toBeVisible() // Should show generic empty state
    
    await page.close()
  })

  test('should handle window resize gracefully', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const helpers = new TauriTestHelpers(page)
    
    // Test responsive behavior
    await helpers.testResponsiveBehavior()
    
    // Verify app remains functional at all sizes
    await expect(page.locator('[data-testid="app-container"]')).toBeVisible()
    await expect(page.locator('[data-testid="navigation"]')).toBeVisible()
    
    await page.close()
  })

  test('should handle malformed data gracefully', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    
    await navPage.navigateToContacts()
    await page.click('[data-testid="new-contact-button"]')
    
    // Test various malformed inputs
    const malformedData = [
      { field: '[data-testid="contact-email-input"]', value: '<script>alert("xss")</script>' },
      { field: '[data-testid="contact-phone-input"]', value: '+++invalid+++phone+++' },
      { field: '[data-testid="contact-first-name-input"]', value: ''; DROP TABLE contacts; --' }
    ]
    
    for (const data of malformedData) {
      await page.fill(data.field, data.value)
    }
    
    await page.click('[data-testid="save-contact-button"]')
    
    // Should show validation errors or sanitize input
    // App should not crash or execute malicious code
    await expect(page.locator('[data-testid="app-container"]')).toBeVisible()
    
    await page.close()
  })
})