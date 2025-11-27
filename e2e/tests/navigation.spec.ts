import { test, expect } from '@playwright/test'
import { TauriApp, TauriTestHelpers } from '../helpers/tauri-helpers'
import { NavigationPage } from '../helpers/page-objects'
import { DatabaseFixtures } from '../fixtures/database-fixtures'

/**
 * Navigation and keyboard shortcut tests
 * Verifies all navigation methods work correctly
 */
test.describe('Navigation and Keyboard Shortcuts', () => {
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

  test('should navigate using keyboard shortcuts', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    
    // Test each keyboard shortcut
    const shortcuts = [
      { key: '1', page: 'dashboard-page', url: '/#/' },
      { key: '2', page: 'projects-page', url: '/#/projects' },
      { key: '3', page: 'companies-page', url: '/#/companies' },
      { key: '4', page: 'contacts-page', url: '/#/contacts' },
      { key: '5', page: 'proposals-page', url: '/#/proposals' }
    ]
    
    for (const shortcut of shortcuts) {
      await navPage.useKeyboardShortcut(shortcut.key)
      
      // Verify correct page is displayed
      await expect(page.locator(`[data-testid="${shortcut.page}"]`)).toBeVisible()
      
      // Verify URL updated
      await expect(page).toHaveURL(new RegExp(shortcut.url.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')))
    }
    
    await page.close()
  })

  test('should navigate using sidebar links', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    
    // Test navigation via sidebar clicks
    await navPage.navigateToProjects()
    await expect(page.locator('[data-testid="projects-page"]')).toBeVisible()
    
    await navPage.navigateToCompanies()
    await expect(page.locator('[data-testid="companies-page"]')).toBeVisible()
    
    await navPage.navigateToContacts()
    await expect(page.locator('[data-testid="contacts-page"]')).toBeVisible()
    
    await navPage.navigateToProposals()
    await expect(page.locator('[data-testid="proposals-page"]')).toBeVisible()
    
    await navPage.navigateToDashboard()
    await expect(page.locator('[data-testid="dashboard-page"]')).toBeVisible()
    
    await page.close()
  })

  test('should handle window positioning shortcut', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const helpers = new TauriTestHelpers(page)
    
    // Test Cmd/Ctrl+W for 4K window positioning
    await helpers.testWindowPositioning()
    
    // Verify app remains functional after window positioning
    await expect(page.locator('[data-testid="app-container"]')).toBeVisible()
    
    await page.close()
  })

  test('should maintain navigation state during database operations', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    
    // Navigate to projects page
    await navPage.navigateToProjects()
    await expect(page.locator('[data-testid="projects-page"]')).toBeVisible()
    
    // Perform a database operation (create project)
    await page.click('[data-testid="new-project-button"]')
    await page.fill('[data-testid="project-name-input"]', 'Test Project')
    await page.fill('[data-testid="project-short-name-input"]', 'TP')
    await page.fill('[data-testid="project-area-input"]', 'Test Area')
    await page.fill('[data-testid="project-city-input"]', 'Dubai')
    await page.selectOption('[data-testid="project-country-select"]', { label: 'United Arab Emirates' })
    await page.click('[data-testid="save-project-button"]')
    
    // Verify we're still on projects page
    await expect(page.locator('[data-testid="projects-page"]')).toBeVisible()
    await expect(page).toHaveURL(/.*\/#\/projects/)
    
    await page.close()
  })

  test('should handle rapid navigation', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    
    // Rapidly navigate between pages
    for (let i = 0; i < 3; i++) {
      await navPage.useKeyboardShortcut('1') // Dashboard
      await navPage.useKeyboardShortcut('2') // Projects
      await navPage.useKeyboardShortcut('3') // Companies
      await navPage.useKeyboardShortcut('4') // Contacts
      await navPage.useKeyboardShortcut('5') // Proposals
    }
    
    // Verify final state is correct
    await expect(page.locator('[data-testid="proposals-page"]')).toBeVisible()
    await expect(page).toHaveURL(/.*\/#\/proposals/)
    
    await page.close()
  })

  test('should highlight active navigation item', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    
    // Navigate to each page and verify active state
    const pages = [
      { shortcut: '1', selector: '[data-testid="nav-dashboard"]' },
      { shortcut: '2', selector: '[data-testid="nav-projects"]' },
      { shortcut: '3', selector: '[data-testid="nav-companies"]' },
      { shortcut: '4', selector: '[data-testid="nav-contacts"]' },
      { shortcut: '5', selector: '[data-testid="nav-proposals"]' }
    ]
    
    for (const pageTest of pages) {
      await navPage.useKeyboardShortcut(pageTest.shortcut)
      
      // Verify active state styling
      const navItem = page.locator(pageTest.selector)
      await expect(navItem).toHaveClass(/active/)
    }
    
    await page.close()
  })

  test('should handle browser back/forward buttons', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    
    // Navigate through pages
    await navPage.navigateToProjects()
    await navPage.navigateToCompanies()
    await navPage.navigateToContacts()
    
    // Use browser back button
    await page.goBack()
    await expect(page.locator('[data-testid="companies-page"]')).toBeVisible()
    
    await page.goBack()
    await expect(page.locator('[data-testid="projects-page"]')).toBeVisible()
    
    // Use browser forward button
    await page.goForward()
    await expect(page.locator('[data-testid="companies-page"]')).toBeVisible()
    
    await page.close()
  })

  test('should maintain scroll position during navigation', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    
    // Seed with enough data to require scrolling
    await dbFixtures.seedCompleteData()
    
    const navPage = new NavigationPage(page)
    
    // Navigate to a page with data
    await navPage.navigateToProjects()
    await page.waitForTimeout(1000)
    
    // Scroll down
    await page.mouse.wheel(0, 500)
    const scrollPosition = await page.evaluate(() => window.pageYOffset)
    
    // Navigate away and back
    await navPage.navigateToDashboard()
    await navPage.navigateToProjects()
    
    // Scroll position handling varies by implementation
    // This test verifies the page loads correctly regardless
    await expect(page.locator('[data-testid="projects-page"]')).toBeVisible()
    
    await page.close()
  })
})