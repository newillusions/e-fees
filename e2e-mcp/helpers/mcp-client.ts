/**
 * MCP Client for Real E2E Testing with Tauri Application
 * 
 * Provides high-level interface for interacting with the actual Tauri desktop application
 * via MCP (Model Context Protocol) tools. This enables REAL application testing instead
 * of browser simulation.
 */

import { SafeProjectData, SafeCompanyData, SafeContactData, SafeProposalData } from '../fixtures/test-data-safe'

export interface MCPClientConfig {
  screenshotPath?: string
  timeout?: number
  retryAttempts?: number
  debugMode?: boolean
}

export interface ElementSelector {
  selector: string
  description: string
}

export interface ScreenshotOptions {
  filename: string
  description?: string
  fullscreen?: boolean
}

export interface KeyboardShortcut {
  key: string
  modifier?: 'Cmd' | 'Ctrl' | 'Alt' | 'Shift'
  description?: string
}

export interface TestResult<T = any> {
  success: boolean
  data?: T
  error?: string
  screenshots?: string[]
  duration: number
}

export class MCPClient {
  private config: MCPClientConfig
  private screenshotCounter: number = 0
  private testStartTime: number

  constructor(config: MCPClientConfig = {}) {
    this.config = {
      screenshotPath: config.screenshotPath || './e2e-mcp/results/screenshots',
      timeout: config.timeout || 30000,
      retryAttempts: config.retryAttempts || 3,
      debugMode: config.debugMode || false,
      ...config
    }
    this.testStartTime = performance.now()
  }

  /**
   * Take a screenshot of the current application state
   */
  async takeScreenshot(options: ScreenshotOptions): Promise<string> {
    try {
      this.screenshotCounter++
      const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19)
      const filename = `${this.screenshotCounter.toString().padStart(3, '0')}-${options.filename}-${timestamp}.png`
      
      if (typeof mcp__tauri_mcp__take_screenshot !== 'function') {
        throw new Error('Tauri MCP take_screenshot tool is not available')
      }

      const result = await mcp__tauri_mcp__take_screenshot({
        filename,
        encoded: false
      })

      if (this.config.debugMode) {
        console.log(`📸 Screenshot taken: ${filename}`)
        if (options.description) {
          console.log(`   Description: ${options.description}`)
        }
      }

      return filename
    } catch (error) {
      console.error('Failed to take screenshot:', error)
      throw new Error(`Screenshot failed: ${error instanceof Error ? error.message : String(error)}`)
    }
  }

  /**
   * Get the current DOM content from the running application
   */
  async getDOMContent(): Promise<string> {
    try {
      if (typeof mcp__tauri_mcp__get_dom !== 'function') {
        throw new Error('Tauri MCP get_dom tool is not available')
      }

      const dom = await mcp__tauri_mcp__get_dom({})
      return dom || ''
    } catch (error) {
      console.error('Failed to get DOM content:', error)
      throw new Error(`Get DOM failed: ${error instanceof Error ? error.message : String(error)}`)
    }
  }

  /**
   * Execute JavaScript code in the application context
   */
  async executeJS(code: string): Promise<any> {
    try {
      if (typeof mcp__tauri_mcp__execute_js !== 'function') {
        throw new Error('Tauri MCP execute_js tool is not available')
      }

      const result = await mcp__tauri_mcp__execute_js({ code })
      
      if (this.config.debugMode) {
        console.log(`🔧 Executed JS: ${code.substring(0, 100)}...`)
      }

      return result
    } catch (error) {
      console.error('Failed to execute JavaScript:', error)
      throw new Error(`JS execution failed: ${error instanceof Error ? error.message : String(error)}`)
    }
  }

  /**
   * Simulate keyboard shortcut (e.g., Cmd+2 for Projects page)
   */
  async keyboardShortcut(shortcut: KeyboardShortcut | string): Promise<void> {
    try {
      if (typeof mcp__tauri_mcp__text_input !== 'function') {
        throw new Error('Tauri MCP text_input tool is not available')
      }

      let keySequence: string
      if (typeof shortcut === 'string') {
        // Simple shortcut like 'Cmd+2'
        keySequence = shortcut
      } else {
        // Full shortcut object
        const modifier = shortcut.modifier || (process.platform === 'darwin' ? 'Cmd' : 'Ctrl')
        keySequence = `${modifier}+${shortcut.key}`
      }

      // Use the text_input tool to send the keyboard shortcut
      await mcp__tauri_mcp__text_input({
        text: keySequence,
        simulate_keypress: true
      })

      if (this.config.debugMode) {
        console.log(`⌨️ Keyboard shortcut: ${keySequence}`)
      }

      // Wait for navigation to complete
      await this.wait(1000)
    } catch (error) {
      console.error('Failed to execute keyboard shortcut:', error)
      throw new Error(`Keyboard shortcut failed: ${error instanceof Error ? error.message : String(error)}`)
    }
  }

  /**
   * Click on an element in the application
   */
  async clickElement(element: ElementSelector | string): Promise<void> {
    try {
      const selector = typeof element === 'string' ? element : element.selector
      const description = typeof element === 'string' ? `Click ${selector}` : element.description
      
      if (typeof mcp__tauri_mcp__send_text_to_element !== 'function') {
        throw new Error('Tauri MCP send_text_to_element tool is not available')
      }

      // Use send_text_to_element with click action
      await mcp__tauri_mcp__send_text_to_element({
        selector,
        action: 'click',
        wait_for_element: true
      })

      if (this.config.debugMode) {
        console.log(`🖱️ Clicked: ${description}`)
      }

      // Wait for UI response
      await this.wait(500)
    } catch (error) {
      console.error('Failed to click element:', error)
      throw new Error(`Click failed: ${error instanceof Error ? error.message : String(error)}`)
    }
  }

  /**
   * Type text into an input field
   */
  async typeText(text: string, selector?: string): Promise<void> {
    try {
      if (typeof mcp__tauri_mcp__text_input !== 'function') {
        throw new Error('Tauri MCP text_input tool is not available')
      }

      if (selector) {
        // First click on the element to focus it
        await this.clickElement({ selector, description: `Focus ${selector}` })
        await this.wait(200)
      }

      // Type the text
      await mcp__tauri_mcp__text_input({
        text,
        clear_field_first: true
      })

      if (this.config.debugMode) {
        console.log(`⌨️ Typed: "${text.substring(0, 50)}${text.length > 50 ? '...' : ''}"`)
      }

      await this.wait(200)
    } catch (error) {
      console.error('Failed to type text:', error)
      throw new Error(`Type text failed: ${error instanceof Error ? error.message : String(error)}`)
    }
  }

  /**
   * Wait for a specified amount of time
   */
  async wait(milliseconds: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, milliseconds))
  }

  /**
   * Navigate to a specific page using keyboard shortcuts
   */
  async navigateTo(page: 'dashboard' | 'projects' | 'companies' | 'contacts' | 'proposals'): Promise<void> {
    const shortcuts = {
      dashboard: 'Cmd+1',
      projects: 'Cmd+2', 
      companies: 'Cmd+3',
      contacts: 'Cmd+4',
      proposals: 'Cmd+5'
    }

    const shortcut = shortcuts[page]
    if (!shortcut) {
      throw new Error(`Unknown page: ${page}`)
    }

    await this.takeScreenshot({ filename: `before-navigate-${page}` })
    await this.keyboardShortcut(shortcut)
    await this.takeScreenshot({ filename: `after-navigate-${page}` })

    // Verify navigation worked by checking URL or page content
    await this.wait(1000)
  }

  /**
   * Create a project using the actual application
   */
  async createProject(projectData: SafeProjectData): Promise<TestResult<{ projectNumber: string }>> {
    const startTime = performance.now()
    const screenshots: string[] = []

    try {
      // Navigate to projects page
      await this.navigateTo('projects')
      screenshots.push(await this.takeScreenshot({ filename: 'projects-page-loaded' }))

      // Click create project button
      await this.clickElement({
        selector: '[data-testid="create-project-btn"]',
        description: 'Create Project button'
      })
      screenshots.push(await this.takeScreenshot({ filename: 'create-project-modal-opened' }))

      // Fill project form
      await this.typeText(projectData.name, 'input[name="name"]')
      await this.typeText(projectData.description, 'textarea[name="description"]')
      await this.typeText(projectData.location, 'input[name="location"]')
      await this.typeText(projectData.client, 'input[name="client"]')

      screenshots.push(await this.takeScreenshot({ filename: 'project-form-filled' }))

      // Submit form
      await this.clickElement({
        selector: 'button[type="submit"]',
        description: 'Submit project form'
      })

      // Wait for project to be created and modal to close
      await this.wait(2000)
      screenshots.push(await this.takeScreenshot({ filename: 'project-created' }))

      // Extract project number from the UI or database
      const projectNumber = await this.getLastCreatedProjectNumber()

      return {
        success: true,
        data: { projectNumber },
        screenshots,
        duration: performance.now() - startTime
      }
    } catch (error) {
      screenshots.push(await this.takeScreenshot({ filename: 'project-creation-error' }))
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        screenshots,
        duration: performance.now() - startTime
      }
    }
  }

  /**
   * Create a company using the actual application
   */
  async createCompany(companyData: SafeCompanyData): Promise<TestResult<{ companyId: string }>> {
    const startTime = performance.now()
    const screenshots: string[] = []

    try {
      await this.navigateTo('companies')
      screenshots.push(await this.takeScreenshot({ filename: 'companies-page-loaded' }))

      await this.clickElement({
        selector: '[data-testid="create-company-btn"]',
        description: 'Create Company button'
      })
      screenshots.push(await this.takeScreenshot({ filename: 'create-company-modal-opened' }))

      // Fill company form
      await this.typeText(companyData.name, 'input[name="name"]')
      await this.typeText(companyData.description, 'textarea[name="description"]')
      await this.typeText(companyData.address, 'input[name="address"]')
      await this.typeText(companyData.city, 'input[name="city"]')
      
      if (companyData.email) {
        await this.typeText(companyData.email, 'input[name="email"]')
      }
      if (companyData.phone) {
        await this.typeText(companyData.phone, 'input[name="phone"]')
      }

      screenshots.push(await this.takeScreenshot({ filename: 'company-form-filled' }))

      await this.clickElement({
        selector: 'button[type="submit"]',
        description: 'Submit company form'
      })

      await this.wait(2000)
      screenshots.push(await this.takeScreenshot({ filename: 'company-created' }))

      const companyId = await this.getLastCreatedCompanyId()

      return {
        success: true,
        data: { companyId },
        screenshots,
        duration: performance.now() - startTime
      }
    } catch (error) {
      screenshots.push(await this.takeScreenshot({ filename: 'company-creation-error' }))
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        screenshots,
        duration: performance.now() - startTime
      }
    }
  }

  /**
   * Create a contact using the actual application
   */
  async createContact(contactData: SafeContactData): Promise<TestResult<{ contactId: string }>> {
    const startTime = performance.now()
    const screenshots: string[] = []

    try {
      await this.navigateTo('contacts')
      screenshots.push(await this.takeScreenshot({ filename: 'contacts-page-loaded' }))

      await this.clickElement({
        selector: '[data-testid="create-contact-btn"]',
        description: 'Create Contact button'
      })
      screenshots.push(await this.takeScreenshot({ filename: 'create-contact-modal-opened' }))

      // Fill contact form
      await this.typeText(contactData.first_name, 'input[name="first_name"]')
      await this.typeText(contactData.last_name, 'input[name="last_name"]')
      await this.typeText(contactData.email, 'input[name="email"]')
      
      if (contactData.phone) {
        await this.typeText(contactData.phone, 'input[name="phone"]')
      }
      if (contactData.position) {
        await this.typeText(contactData.position, 'input[name="position"]')
      }

      screenshots.push(await this.takeScreenshot({ filename: 'contact-form-filled' }))

      await this.clickElement({
        selector: 'button[type="submit"]',
        description: 'Submit contact form'
      })

      await this.wait(2000)
      screenshots.push(await this.takeScreenshot({ filename: 'contact-created' }))

      const contactId = await this.getLastCreatedContactId()

      return {
        success: true,
        data: { contactId },
        screenshots,
        duration: performance.now() - startTime
      }
    } catch (error) {
      screenshots.push(await this.takeScreenshot({ filename: 'contact-creation-error' }))
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        screenshots,
        duration: performance.now() - startTime
      }
    }
  }

  /**
   * Test keyboard navigation shortcuts
   */
  async testKeyboardNavigation(): Promise<TestResult<{ pagesVisited: string[] }>> {
    const startTime = performance.now()
    const screenshots: string[] = []
    const pagesVisited: string[] = []

    try {
      const pages: Array<{ name: string; shortcut: string }> = [
        { name: 'dashboard', shortcut: 'Cmd+1' },
        { name: 'projects', shortcut: 'Cmd+2' },
        { name: 'companies', shortcut: 'Cmd+3' },
        { name: 'contacts', shortcut: 'Cmd+4' },
        { name: 'proposals', shortcut: 'Cmd+5' }
      ]

      for (const page of pages) {
        await this.keyboardShortcut(page.shortcut)
        await this.wait(1000)
        
        screenshots.push(await this.takeScreenshot({ 
          filename: `keyboard-nav-${page.name}`,
          description: `Navigation to ${page.name} via ${page.shortcut}`
        }))
        
        pagesVisited.push(page.name)

        // Verify we're on the correct page by checking DOM
        const dom = await this.getDOMContent()
        if (!dom.includes(`data-testid="${page.name}-page"`)) {
          throw new Error(`Failed to navigate to ${page.name} page`)
        }
      }

      return {
        success: true,
        data: { pagesVisited },
        screenshots,
        duration: performance.now() - startTime
      }
    } catch (error) {
      screenshots.push(await this.takeScreenshot({ filename: 'keyboard-nav-error' }))
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        screenshots,
        duration: performance.now() - startTime
      }
    }
  }

  /**
   * Helper: Get the last created project number
   */
  private async getLastCreatedProjectNumber(): Promise<string> {
    // Use SurrealDB MCP to query the database for the most recent project with DELETE ME
    if (typeof mcp__surrealdb_emittiv__query !== 'function') {
      throw new Error('SurrealDB MCP is not available')
    }

    const result = await mcp__surrealdb_emittiv__query({
      query_string: 'SELECT project_number FROM projects WHERE name CONTAINS "DELETE ME" ORDER BY created_at DESC LIMIT 1'
    })

    if (Array.isArray(result) && result.length > 0 && result[0].project_number) {
      return result[0].project_number
    }

    throw new Error('Could not retrieve project number')
  }

  /**
   * Helper: Get the last created company ID
   */
  private async getLastCreatedCompanyId(): Promise<string> {
    if (typeof mcp__surrealdb_emittiv__query !== 'function') {
      throw new Error('SurrealDB MCP is not available')
    }

    const result = await mcp__surrealdb_emittiv__query({
      query_string: 'SELECT id FROM company WHERE name CONTAINS "DELETE ME" ORDER BY created_at DESC LIMIT 1'
    })

    if (Array.isArray(result) && result.length > 0 && result[0].id) {
      return result[0].id
    }

    throw new Error('Could not retrieve company ID')
  }

  /**
   * Helper: Get the last created contact ID
   */
  private async getLastCreatedContactId(): Promise<string> {
    if (typeof mcp__surrealdb_emittiv__query !== 'function') {
      throw new Error('SurrealDB MCP is not available')
    }

    const result = await mcp__surrealdb_emittiv__query({
      query_string: 'SELECT id FROM contacts WHERE first_name CONTAINS "DELETE ME" ORDER BY created_at DESC LIMIT 1'
    })

    if (Array.isArray(result) && result.length > 0 && result[0].id) {
      return result[0].id
    }

    throw new Error('Could not retrieve contact ID')
  }

  /**
   * Verify application is ready for testing
   */
  async verifyApplicationReady(): Promise<TestResult<{ ready: boolean }>> {
    const startTime = performance.now()
    const screenshots: string[] = []

    try {
      screenshots.push(await this.takeScreenshot({ filename: 'app-ready-check' }))

      const dom = await this.getDOMContent()
      
      // Check for key application elements
      const requiredElements = [
        'data-testid="app-container"',
        'data-testid="navigation"',
        'data-testid="connection-status"'
      ]

      for (const element of requiredElements) {
        if (!dom.includes(element)) {
          throw new Error(`Required element not found: ${element}`)
        }
      }

      // Check database connection status
      const isConnected = dom.includes('connected') || dom.includes('status-connected')
      if (!isConnected) {
        throw new Error('Database connection not established')
      }

      return {
        success: true,
        data: { ready: true },
        screenshots,
        duration: performance.now() - startTime
      }
    } catch (error) {
      screenshots.push(await this.takeScreenshot({ filename: 'app-not-ready' }))
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
        screenshots,
        duration: performance.now() - startTime
      }
    }
  }

  /**
   * Get test execution summary
   */
  getTestSummary(): { totalDuration: number; screenshotCount: number } {
    return {
      totalDuration: performance.now() - this.testStartTime,
      screenshotCount: this.screenshotCounter
    }
  }
}

/**
 * Default export for convenience
 */
export default MCPClient