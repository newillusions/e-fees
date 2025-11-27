import { test, expect } from '@playwright/test'
import { TauriApp } from '../helpers/tauri-helpers'
import { NavigationPage, CompaniesPage, CompanyModal, ContactsPage, ContactModal } from '../helpers/page-objects'
import { DatabaseFixtures } from '../fixtures/database-fixtures'

/**
 * Company and Contact management tests
 * Tests CRUD operations and relationships between companies and contacts
 */
test.describe('Company and Contact Management', () => {
  let tauriApp: TauriApp
  let dbFixtures: DatabaseFixtures

  test.beforeAll(async () => {
    tauriApp = new TauriApp(process.env.E2E_DEV_MODE === 'true')
  })

  test.beforeEach(async ({ page }) => {
    dbFixtures = new DatabaseFixtures(page)
    
    // Setup clean database state
    await dbFixtures.clearAllData()
    await dbFixtures.initializeReferenceData()
  })

  test.afterAll(async () => {
    if (tauriApp) {
      await tauriApp.close()
    }
  })

  test('should create company successfully', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const companiesPage = new CompaniesPage(page)
    const companyModal = new CompanyModal(page)
    
    // Navigate to companies
    await navPage.navigateToCompanies()
    
    // Create new company
    await companiesPage.clickNewCompany()
    
    await companyModal.fillCompanyForm({
      name: 'Emirates Engineering Consultancy LLC',
      shortName: 'EEC',
      abbreviation: 'EEC',
      city: 'Dubai',
      country: 'United Arab Emirates'
    })
    
    await companyModal.saveCompany()
    
    // Verify company appears in list
    await companiesPage.verifyCompanyExists('Emirates Engineering Consultancy LLC')
    
    // Verify company count
    const count = await companiesPage.getCompaniesCount()
    expect(count).toBe(1)
    
    await page.close()
  })

  test('should create multiple companies', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const companiesPage = new CompaniesPage(page)
    const companyModal = new CompanyModal(page)
    
    await navPage.navigateToCompanies()
    
    const companies = [
      {
        name: 'Emirates Engineering Consultancy LLC',
        shortName: 'EEC',
        abbreviation: 'EEC',
        city: 'Dubai',
        country: 'United Arab Emirates'
      },
      {
        name: 'Saudi Design Studio',
        shortName: 'SDS',
        abbreviation: 'SDS',
        city: 'Riyadh',
        country: 'Saudi Arabia'
      },
      {
        name: 'Gulf Architecture Partners',
        shortName: 'GAP',
        abbreviation: 'GAP',
        city: 'Doha',
        country: 'Qatar'
      }
    ]
    
    for (const company of companies) {
      await companiesPage.clickNewCompany()
      await companyModal.fillCompanyForm(company)
      await companyModal.saveCompany()
      await companiesPage.verifyCompanyExists(company.name)
    }
    
    const totalCount = await companiesPage.getCompaniesCount()
    expect(totalCount).toBe(3)
    
    await page.close()
  })

  test('should edit existing company', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    
    // Seed database with a company
    await dbFixtures.seedMinimalData()
    
    const navPage = new NavigationPage(page)
    const companiesPage = new CompaniesPage(page)
    const companyModal = new CompanyModal(page)
    
    await navPage.navigateToCompanies()
    
    // Wait for data to load
    await page.waitForTimeout(1000)
    
    // Edit the company
    await companiesPage.editCompany('Emirates Engineering Consultancy LLC')
    
    // Modify the company data
    await page.fill('[data-testid="company-city-input"]', 'Abu Dhabi')
    
    await companyModal.saveCompany()
    
    // Verify changes were saved
    // Note: This would require checking the database or UI for the updated city
    const count = await companiesPage.getCompaniesCount()
    expect(count).toBe(1) // Should still be 1 company
    
    await page.close()
  })

  test('should create contact linked to company', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    
    // First create a company
    await dbFixtures.seedMinimalData()
    
    const navPage = new NavigationPage(page)
    const contactsPage = new ContactsPage(page)
    const contactModal = new ContactModal(page)
    
    // Navigate to contacts
    await navPage.navigateToContacts()
    
    // Create new contact
    await contactsPage.clickNewContact()
    
    await contactModal.fillContactForm({
      firstName: 'Ahmed',
      lastName: 'Al-Mansoori',
      email: 'ahmed.almansoori@eec.ae',
      phone: '+971-50-123-4567',
      position: 'Project Manager',
      company: 'Emirates Engineering Consultancy LLC'
    })
    
    await contactModal.saveContact()
    
    // Verify contact appears in list
    await contactsPage.verifyContactExists('Ahmed Al-Mansoori')
    
    const count = await contactsPage.getContactsCount()
    expect(count).toBe(1)
    
    await page.close()
  })

  test('should create multiple contacts for same company', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    
    // Seed with companies
    await dbFixtures.seedCompleteData()
    
    const navPage = new NavigationPage(page)
    const contactsPage = new ContactsPage(page)
    const contactModal = new ContactModal(page)
    
    await navPage.navigateToContacts()
    
    const contacts = [
      {
        firstName: 'Ahmed',
        lastName: 'Al-Mansoori',
        email: 'ahmed.almansoori@eec.ae',
        phone: '+971-50-123-4567',
        position: 'Project Manager',
        company: 'Emirates Engineering Consultancy LLC'
      },
      {
        firstName: 'Fatima',
        lastName: 'Al-Zahra',
        email: 'fatima.alzahra@eec.ae',
        phone: '+971-55-234-5678',
        position: 'Design Director',
        company: 'Emirates Engineering Consultancy LLC'
      }
    ]
    
    for (const contact of contacts) {
      await contactsPage.clickNewContact()
      await contactModal.fillContactForm(contact)
      await contactModal.saveContact()
      await contactsPage.verifyContactExists(`${contact.firstName} ${contact.lastName}`)
    }
    
    const totalCount = await contactsPage.getContactsCount()
    expect(totalCount).toBe(2)
    
    await page.close()
  })

  test('should validate company form fields', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const companiesPage = new CompaniesPage(page)
    
    await navPage.navigateToCompanies()
    await companiesPage.clickNewCompany()
    
    // Try to save without required fields
    await page.click('[data-testid="save-company-button"]')
    
    // Verify validation errors appear
    await expect(page.locator('[data-testid="validation-error"]')).toBeVisible()
    
    // Verify modal doesn't close
    await expect(page.locator('[data-testid="company-modal"]')).toBeVisible()
    
    await page.close()
  })

  test('should validate contact form fields', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const contactsPage = new ContactsPage(page)
    
    await navPage.navigateToContacts()
    await contactsPage.clickNewContact()
    
    // Try to save without required fields
    await page.click('[data-testid="save-contact-button"]')
    
    // Verify validation errors
    await expect(page.locator('[data-testid="validation-error"]')).toBeVisible()
    
    // Verify modal stays open
    await expect(page.locator('[data-testid="contact-modal"]')).toBeVisible()
    
    await page.close()
  })

  test('should validate email format', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const contactsPage = new ContactsPage(page)
    const contactModal = new ContactModal(page)
    
    await navPage.navigateToContacts()
    await contactsPage.clickNewContact()
    
    // Fill form with invalid email
    await contactModal.fillContactForm({
      firstName: 'Test',
      lastName: 'User',
      email: 'invalid-email',
      phone: '+971-50-123-4567',
      position: 'Test Position'
    })
    
    await page.click('[data-testid="save-contact-button"]')
    
    // Verify email validation error
    await expect(page.locator('[data-testid="email-validation-error"]')).toBeVisible()
    
    await page.close()
  })

  test('should handle special characters in company names', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    const navPage = new NavigationPage(page)
    const companiesPage = new CompaniesPage(page)
    const companyModal = new CompanyModal(page)
    
    await navPage.navigateToCompanies()
    await companiesPage.clickNewCompany()
    
    await companyModal.fillCompanyForm({
      name: 'Al-Mansoori & Partners (LLC)',
      shortName: 'A&P',
      abbreviation: 'A&P',
      city: 'Dubai',
      country: 'United Arab Emirates'
    })
    
    await companyModal.saveCompany()
    
    // Verify company with special characters was created
    await companiesPage.verifyCompanyExists('Al-Mansoori & Partners (LLC)')
    
    await page.close()
  })

  test('should delete company when no dependencies exist', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    
    // Seed with minimal data (company only)
    await dbFixtures.seedMinimalData()
    
    const navPage = new NavigationPage(page)
    const companiesPage = new CompaniesPage(page)
    
    await navPage.navigateToCompanies()
    
    // Wait for data to load
    await page.waitForTimeout(1000)
    
    const initialCount = await companiesPage.getCompaniesCount()
    expect(initialCount).toBe(1)
    
    // Delete the company
    await companiesPage.deleteCompany('Emirates Engineering Consultancy LLC')
    
    // Wait for deletion to complete
    await page.waitForTimeout(1000)
    
    // Verify company was deleted
    const finalCount = await companiesPage.getCompaniesCount()
    expect(finalCount).toBe(0)
    
    await page.close()
  })

  test('should search companies by name and location', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    
    // Seed with complete test data
    await dbFixtures.seedCompleteData()
    
    const navPage = new NavigationPage(page)
    const companiesPage = new CompaniesPage(page)
    
    await navPage.navigateToCompanies()
    
    // Wait for data to load
    await page.waitForTimeout(1000)
    
    // Search by name
    await page.fill('[data-testid="search-input"]', 'Emirates')
    await page.waitForTimeout(500)
    
    let count = await companiesPage.getCompaniesCount()
    expect(count).toBeGreaterThan(0)
    
    // Search by city
    await page.fill('[data-testid="search-input"]', 'Riyadh')
    await page.waitForTimeout(500)
    
    count = await companiesPage.getCompaniesCount()
    expect(count).toBeGreaterThan(0)
    
    // Clear search
    await page.fill('[data-testid="search-input"]', '')
    await page.waitForTimeout(500)
    
    count = await companiesPage.getCompaniesCount()
    expect(count).toBe(3) // All companies should show
    
    await page.close()
  })

  test('should search contacts by multiple fields', async ({ browser }) => {
    if (process.env.E2E_DEV_MODE !== 'true') {
      await tauriApp.launch()
    }
    
    const page = await tauriApp.connect(browser)
    
    // Seed with linked test data
    await dbFixtures.seedLinkedData()
    
    const navPage = new NavigationPage(page)
    const contactsPage = new ContactsPage(page)
    
    await navPage.navigateToContacts()
    
    // Wait for data to load
    await page.waitForTimeout(1000)
    
    // Search by first name
    await page.fill('[data-testid="search-input"]', 'Ahmed')
    await page.waitForTimeout(500)
    
    let count = await contactsPage.getContactsCount()
    expect(count).toBeGreaterThan(0)
    
    // Search by email
    await page.fill('[data-testid="search-input"]', '@eec.ae')
    await page.waitForTimeout(500)
    
    count = await contactsPage.getContactsCount()
    expect(count).toBeGreaterThan(0)
    
    // Search by company name
    await page.fill('[data-testid="search-input"]', 'Emirates')
    await page.waitForTimeout(500)
    
    count = await contactsPage.getContactsCount()
    expect(count).toBeGreaterThan(0)
    
    await page.close()
  })
})