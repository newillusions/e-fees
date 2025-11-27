import { test, expect } from '@playwright/test'
import { TauriApp, TauriTestHelpers } from '../helpers/tauri-helpers'
import { NavigationPage } from '../helpers/page-objects'
import { DatabaseFixtures } from '../fixtures/database-fixtures'

/**
 * App startup and initialization flow tests
 * Verifies the application launches correctly and establishes database connection
 */
test.describe('App Startup and Initialization', () => {
  let tauriApp: TauriApp
  let navigationPage: NavigationPage
  let dbFixtures: DatabaseFixtures

  test.beforeAll(async () => {
    // Initialize Tauri app (use dev mode if E2E_DEV_MODE is set)
    tauriApp = new TauriApp(process.env.E2E_DEV_MODE === 'true')
  })

  test.beforeEach(async ({ page }) => {
    navigationPage = new NavigationPage(page)
    dbFixtures = new DatabaseFixtures(page)
    
    // Ensure clean database state
    await dbFixtures.clearAllData()
    await dbFixtures.initializeReferenceData()
  })

  test.afterAll(async () => {
    if (tauriApp) {
      await tauriApp.close()
    }
  })

  test('should launch application successfully', async ({ browser }) => {
    // Launch the Tauri application
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    // Connect to the running app
    const page = await tauriApp.connect(browser)
    
    // Verify app container is loaded
    await expect(page.locator('[data-testid="app-container"]')).toBeVisible()
    
    // Verify title
    await expect(page).toHaveTitle(/E-Fees/)
    
    await page.close()
  })

  test('should establish database connection on startup', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    
    // Wait for connection to be established
    await navPage.verifyConnectionStatus('connected')
    
    // Verify connection status indicator shows green
    const connectionStatus = page.locator('[data-testid="connection-status"]')
    await expect(connectionStatus).toHaveClass(/connected/)
    
    await page.close()
  })

  test('should load initial navigation structure', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    
    // Verify all navigation links are present
    await expect(page.locator('[data-testid="nav-dashboard"]')).toBeVisible()
    await expect(page.locator('[data-testid="nav-projects"]')).toBeVisible()
    await expect(page.locator('[data-testid="nav-companies"]')).toBeVisible()
    await expect(page.locator('[data-testid="nav-contacts"]')).toBeVisible()
    await expect(page.locator('[data-testid="nav-proposals"]')).toBeVisible()
    
    // Verify navigation labels
    await expect(page.locator('[data-testid="nav-dashboard"]')).toContainText('Dashboard')
    await expect(page.locator('[data-testid="nav-projects"]')).toContainText('Projects')
    await expect(page.locator('[data-testid="nav-companies"]')).toContainText('Companies')
    await expect(page.locator('[data-testid="nav-contacts"]')).toContainText('Contacts')
    await expect(page.locator('[data-testid="nav-proposals"]')).toContainText('Proposals')
    
    await page.close()
  })

  test('should display dashboard by default', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    
    // Verify we're on the dashboard by default
    await expect(page.locator('[data-testid="dashboard-page"]')).toBeVisible()
    await expect(page).toHaveURL(/.*\/#\//)
    
    // Verify dashboard components are loaded
    await expect(page.locator('[data-testid="quick-actions"]')).toBeVisible()
    
    await page.close()
  })

  test('should handle database connection failure gracefully', async ({ browser }) => {
    // This test would require stopping the SurrealDB instance temporarily
    // For now, we'll test the error handling by simulating a connection failure
    
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const helpers = new TauriTestHelpers(page)
    
    // Simulate connection loss
    await helpers.simulateConnectionLoss()
    
    // Verify error state is handled
    await helpers.waitForAppState('ready') // Should recover or show appropriate error
    
    await page.close()
  })

  test('should initialize with empty data state', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    
    // Navigate to each page and verify empty state
    await navPage.navigateToProjects()
    await expect(page.locator('[data-testid="empty-state"]')).toBeVisible()
    
    await navPage.navigateToCompanies()
    await expect(page.locator('[data-testid="empty-state"]')).toBeVisible()
    
    await navPage.navigateToContacts()
    await expect(page.locator('[data-testid="empty-state"]')).toBeVisible()
    
    await navPage.navigateToProposals()
    await expect(page.locator('[data-testid="empty-state"]')).toBeVisible()
    
    await page.close()
  })

  test('should respond to keyboard shortcuts', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    
    // Test keyboard navigation shortcuts
    await navPage.useKeyboardShortcut('1')
    await expect(page.locator('[data-testid="dashboard-page"]')).toBeVisible()
    
    await navPage.useKeyboardShortcut('2')
    await expect(page.locator('[data-testid="projects-page"]')).toBeVisible()
    
    await navPage.useKeyboardShortcut('3')
    await expect(page.locator('[data-testid="companies-page"]')).toBeVisible()
    
    await navPage.useKeyboardShortcut('4')
    await expect(page.locator('[data-testid="contacts-page"]')).toBeVisible()
    
    await navPage.useKeyboardShortcut('5')
    await expect(page.locator('[data-testid="proposals-page"]')).toBeVisible()
    
    await page.close()
  })

  test('should handle window positioning shortcut', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const helpers = new TauriTestHelpers(page)
    
    // Test 4K window positioning shortcut
    await helpers.testWindowPositioning()
    
    // The actual window positioning can't be easily tested in headless mode
    // but we can verify the shortcut doesn't cause errors
    await expect(page.locator('[data-testid="app-container"]')).toBeVisible()
    
    await page.close()
  })

  test('should load reference data correctly', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const dbFixtures = new DatabaseFixtures(page)
    
    // Verify reference data is available
    const counts = await dbFixtures.getDatabaseCounts()
    expect(counts.countries).toBeGreaterThan(0)
    expect(counts.currencies).toBeGreaterThan(0)
    
    await page.close()
  })
})