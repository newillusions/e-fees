import { Page, Locator, expect } from '@playwright/test'

/**
 * Page Object Models for E-Fees application
 * Encapsulates page interactions and provides reusable methods for testing
 */

export class BasePage {
  constructor(protected page: Page) {}

  async waitForPageLoad() {
    await this.page.waitForLoadState('domcontentloaded')
    await this.page.waitForSelector('[data-testid="app-container"]', { timeout: 10000 })
  }

  async navigateToRoute(route: string) {
    await this.page.goto(route)
    await this.waitForPageLoad()
  }
}

export class NavigationPage extends BasePage {
  // Navigation elements
  private dashboardLink = '[data-testid="nav-dashboard"]'
  private projectsLink = '[data-testid="nav-projects"]'
  private companiesLink = '[data-testid="nav-companies"]'
  private contactsLink = '[data-testid="nav-contacts"]'
  private proposalsLink = '[data-testid="nav-proposals"]'
  private connectionStatus = '[data-testid="connection-status"]'

  async navigateToDashboard() {
    await this.page.click(this.dashboardLink)
    await this.waitForPageLoad()
  }

  async navigateToProjects() {
    await this.page.click(this.projectsLink)
    await this.waitForPageLoad()
  }

  async navigateToCompanies() {
    await this.page.click(this.companiesLink)
    await this.waitForPageLoad()
  }

  async navigateToContacts() {
    await this.page.click(this.contactsLink)
    await this.waitForPageLoad()
  }

  async navigateToProposals() {
    await this.page.click(this.proposalsLink)
    await this.waitForPageLoad()
  }

  async useKeyboardShortcut(key: string) {
    const isMac = process.platform === 'darwin'
    const modifier = isMac ? 'Meta' : 'Control'
    await this.page.keyboard.press(`${modifier}+${key}`)
    await this.waitForPageLoad()
  }

  async verifyConnectionStatus(status: 'connected' | 'disconnected') {
    const statusElement = this.page.locator(this.connectionStatus)
    await expect(statusElement).toBeVisible()
    
    if (status === 'connected') {
      await expect(statusElement).toHaveClass(/connected/)
    } else {
      await expect(statusElement).toHaveClass(/disconnected/)
    }
  }
}

export class ProjectsPage extends BasePage {
  // Page elements
  private newProjectButton = '[data-testid="new-project-button"]'
  private projectsList = '[data-testid="projects-list"]'
  private searchInput = '[data-testid="search-input"]'
  private projectCard = '[data-testid^="project-card-"]'

  async clickNewProject() {
    await this.page.click(this.newProjectButton)
    await this.page.waitForSelector('[data-testid="project-modal"]')
  }

  async searchProjects(searchTerm: string) {
    await this.page.fill(this.searchInput, searchTerm)
    await this.page.waitForTimeout(500) // Allow for debounced search
  }

  async getProjectsCount(): Promise<number> {
    const projects = await this.page.locator(this.projectCard).count()
    return projects
  }

  async clickProject(projectName: string) {
    await this.page.click(`[data-testid="project-card-${projectName.toLowerCase()}"]`)
  }

  async verifyProjectExists(projectName: string) {
    const projectCard = this.page.locator(`[data-testid="project-card-${projectName.toLowerCase()}"]`)
    await expect(projectCard).toBeVisible()
  }

  async openProjectFolder(projectName: string) {
    const folderButton = this.page.locator(`[data-testid="project-folder-${projectName.toLowerCase()}"]`)
    await folderButton.click()
  }
}

export class ProjectModal extends BasePage {
  // Modal elements
  private modal = '[data-testid="project-modal"]'
  private nameInput = '[data-testid="project-name-input"]'
  private shortNameInput = '[data-testid="project-short-name-input"]'
  private areaInput = '[data-testid="project-area-input"]'
  private cityInput = '[data-testid="project-city-input"]'
  private countrySelect = '[data-testid="project-country-select"]'
  private numberDisplay = '[data-testid="project-number-display"]'
  private saveButton = '[data-testid="save-project-button"]'
  private cancelButton = '[data-testid="cancel-project-button"]'

  async fillProjectForm(data: {
    name: string
    shortName: string
    area: string
    city: string
    country: string
  }) {
    await this.page.fill(this.nameInput, data.name)
    await this.page.fill(this.shortNameInput, data.shortName)
    await this.page.fill(this.areaInput, data.area)
    await this.page.fill(this.cityInput, data.city)
    await this.page.selectOption(this.countrySelect, { label: data.country })
    
    // Wait for project number to be generated
    await this.page.waitForFunction(() => {
      const numberElement = document.querySelector('[data-testid="project-number-display"]')
      return numberElement && numberElement.textContent && numberElement.textContent.trim() !== ''
    })
  }

  async getGeneratedProjectNumber(): Promise<string> {
    const numberElement = this.page.locator(this.numberDisplay)
    return await numberElement.textContent() || ''
  }

  async saveProject() {
    await this.page.click(this.saveButton)
    await this.page.waitForSelector(this.modal, { state: 'hidden' })
  }

  async cancelProject() {
    await this.page.click(this.cancelButton)
    await this.page.waitForSelector(this.modal, { state: 'hidden' })
  }

  async verifyModalVisible() {
    await expect(this.page.locator(this.modal)).toBeVisible()
  }

  async verifyProjectNumberFormat(expectedPattern: RegExp) {
    const projectNumber = await this.getGeneratedProjectNumber()
    expect(projectNumber).toMatch(expectedPattern)
  }
}

export class CompaniesPage extends BasePage {
  private newCompanyButton = '[data-testid="new-company-button"]'
  private companiesList = '[data-testid="companies-list"]'
  private searchInput = '[data-testid="search-input"]'
  private companyCard = '[data-testid^="company-card-"]'

  async clickNewCompany() {
    await this.page.click(this.newCompanyButton)
    await this.page.waitForSelector('[data-testid="company-modal"]')
  }

  async getCompaniesCount(): Promise<number> {
    const companies = await this.page.locator(this.companyCard).count()
    return companies
  }

  async verifyCompanyExists(companyName: string) {
    const companyCard = this.page.locator(`[data-testid*="${companyName.toLowerCase()}"]`)
    await expect(companyCard).toBeVisible()
  }

  async editCompany(companyName: string) {
    await this.page.click(`[data-testid="edit-company-${companyName.toLowerCase()}"]`)
    await this.page.waitForSelector('[data-testid="company-modal"]')
  }

  async deleteCompany(companyName: string) {
    await this.page.click(`[data-testid="delete-company-${companyName.toLowerCase()}"]`)
    await this.page.waitForSelector('[data-testid="confirm-delete-modal"]')
    await this.page.click('[data-testid="confirm-delete-button"]')
  }
}

export class CompanyModal extends BasePage {
  private modal = '[data-testid="company-modal"]'
  private nameInput = '[data-testid="company-name-input"]'
  private shortNameInput = '[data-testid="company-short-name-input"]'
  private abbreviationInput = '[data-testid="company-abbreviation-input"]'
  private cityInput = '[data-testid="company-city-input"]'
  private countryInput = '[data-testid="company-country-input"]'
  private saveButton = '[data-testid="save-company-button"]'
  private cancelButton = '[data-testid="cancel-company-button"]'

  async fillCompanyForm(data: {
    name: string
    shortName: string
    abbreviation: string
    city: string
    country: string
  }) {
    await this.page.fill(this.nameInput, data.name)
    await this.page.fill(this.shortNameInput, data.shortName)
    await this.page.fill(this.abbreviationInput, data.abbreviation)
    await this.page.fill(this.cityInput, data.city)
    await this.page.fill(this.countryInput, data.country)
  }

  async saveCompany() {
    await this.page.click(this.saveButton)
    await this.page.waitForSelector(this.modal, { state: 'hidden' })
  }

  async cancelCompany() {
    await this.page.click(this.cancelButton)
    await this.page.waitForSelector(this.modal, { state: 'hidden' })
  }
}

export class ContactsPage extends BasePage {
  private newContactButton = '[data-testid="new-contact-button"]'
  private contactsList = '[data-testid="contacts-list"]'
  private searchInput = '[data-testid="search-input"]'
  private contactCard = '[data-testid^="contact-card-"]'

  async clickNewContact() {
    await this.page.click(this.newContactButton)
    await this.page.waitForSelector('[data-testid="contact-modal"]')
  }

  async getContactsCount(): Promise<number> {
    const contacts = await this.page.locator(this.contactCard).count()
    return contacts
  }

  async verifyContactExists(contactName: string) {
    const contactCard = this.page.locator(`[data-testid*="${contactName.toLowerCase()}"]`)
    await expect(contactCard).toBeVisible()
  }
}

export class ContactModal extends BasePage {
  private modal = '[data-testid="contact-modal"]'
  private firstNameInput = '[data-testid="contact-first-name-input"]'
  private lastNameInput = '[data-testid="contact-last-name-input"]'
  private emailInput = '[data-testid="contact-email-input"]'
  private phoneInput = '[data-testid="contact-phone-input"]'
  private positionInput = '[data-testid="contact-position-input"]'
  private companySelect = '[data-testid="contact-company-select"]'
  private saveButton = '[data-testid="save-contact-button"]'
  private cancelButton = '[data-testid="cancel-contact-button"]'

  async fillContactForm(data: {
    firstName: string
    lastName: string
    email: string
    phone: string
    position: string
    company?: string
  }) {
    await this.page.fill(this.firstNameInput, data.firstName)
    await this.page.fill(this.lastNameInput, data.lastName)
    await this.page.fill(this.emailInput, data.email)
    await this.page.fill(this.phoneInput, data.phone)
    await this.page.fill(this.positionInput, data.position)
    
    if (data.company) {
      await this.page.selectOption(this.companySelect, { label: data.company })
    }
  }

  async saveContact() {
    await this.page.click(this.saveButton)
    await this.page.waitForSelector(this.modal, { state: 'hidden' })
  }
}

export class ProposalsPage extends BasePage {
  private newProposalButton = '[data-testid="new-proposal-button"]'
  private proposalsList = '[data-testid="proposals-list"]'
  private searchInput = '[data-testid="search-input"]'
  private proposalCard = '[data-testid^="proposal-card-"]'

  async clickNewProposal() {
    await this.page.click(this.newProposalButton)
    await this.page.waitForSelector('[data-testid="proposal-modal"]')
  }

  async getProposalsCount(): Promise<number> {
    const proposals = await this.page.locator(this.proposalCard).count()
    return proposals
  }

  async verifyProposalExists(proposalName: string) {
    const proposalCard = this.page.locator(`[data-testid*="${proposalName.toLowerCase()}"]`)
    await expect(proposalCard).toBeVisible()
  }
}

export class ProposalModal extends BasePage {
  private modal = '[data-testid="proposal-modal"]'
  private nameInput = '[data-testid="proposal-name-input"]'
  private numberInput = '[data-testid="proposal-number-input"]'
  private projectSelect = '[data-testid="proposal-project-select"]'
  private companySelect = '[data-testid="proposal-company-select"]'
  private contactSelect = '[data-testid="proposal-contact-select"]'
  private issueDateInput = '[data-testid="proposal-issue-date-input"]'
  private activityInput = '[data-testid="proposal-activity-input"]'
  private saveButton = '[data-testid="save-proposal-button"]'
  private cancelButton = '[data-testid="cancel-proposal-button"]'

  async fillProposalForm(data: {
    name: string
    number: string
    project: string
    company: string
    contact: string
    issueDate: string
    activity: string
  }) {
    await this.page.fill(this.nameInput, data.name)
    await this.page.fill(this.numberInput, data.number)
    await this.page.selectOption(this.projectSelect, { label: data.project })
    await this.page.selectOption(this.companySelect, { label: data.company })
    
    // Wait for contact dropdown to update based on company selection
    await this.page.waitForTimeout(500)
    await this.page.selectOption(this.contactSelect, { label: data.contact })
    
    await this.page.fill(this.issueDateInput, data.issueDate)
    await this.page.fill(this.activityInput, data.activity)
  }

  async saveProposal() {
    await this.page.click(this.saveButton)
    await this.page.waitForSelector(this.modal, { state: 'hidden' })
  }
}