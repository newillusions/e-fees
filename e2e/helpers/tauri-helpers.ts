import { Page, BrowserContext, Browser } from '@playwright/test'
import { spawn, ChildProcess } from 'child_process'
import { join, resolve } from 'path'
import { promises as fs } from 'fs'

/**
 * Tauri-specific helper functions for E2E testing
 * Handles app launching, window management, and native functionality
 */

export class TauriApp {
  private appProcess: ChildProcess | null = null
  private appPath: string
  private isDevMode: boolean

  constructor(isDevMode = false) {
    this.isDevMode = isDevMode
    this.appPath = this.getAppPath()
  }

  /**
   * Launch the Tauri application
   */
  async launch(): Promise<void> {
    if (this.isDevMode) {
      return this.launchDevMode()
    } else {
      return this.launchBuiltApp()
    }
  }

  /**
   * Close the Tauri application
   */
  async close(): Promise<void> {
    if (this.appProcess && !this.appProcess.killed) {
      this.appProcess.kill('SIGTERM')
      
      // Wait for graceful shutdown
      await new Promise(resolve => {
        if (this.appProcess) {
          this.appProcess.on('exit', resolve)
          setTimeout(resolve, 5000) // Force close after 5s
        }
      })
    }
  }

  /**
   * Wait for app to be ready for testing
   */
  async waitForReady(page: Page, timeout = 30000): Promise<void> {
    // Wait for the application to fully load
    await page.waitForURL('http://localhost:1420/**', { timeout })
    
    // Wait for main app container
    await page.waitForSelector('[data-testid="app-container"]', { timeout })
    
    // Wait for database connection
    await page.waitForFunction(() => {
      const connectionStatus = document.querySelector('[data-testid="connection-status"]')
      return connectionStatus && connectionStatus.classList.contains('connected')
    }, { timeout })
    
    // Wait for initial data load
    await page.waitForTimeout(2000)
  }

  /**
   * Connect to the running Tauri application
   */
  async connect(browser: Browser): Promise<Page> {
    const context = await browser.newContext({
      // Tauri app specific context options
      ignoreHTTPSErrors: true,
      bypassCSP: true,
      viewport: { width: 1280, height: 1200 }
    })

    const page = await context.newPage()
    
    // Navigate to the Tauri app
    await page.goto('http://localhost:1420')
    
    // Wait for app to be ready
    await this.waitForReady(page)
    
    return page
  }

  /**
   * Launch in development mode
   */
  private async launchDevMode(): Promise<void> {
    console.log('🚀 Launching Tauri app in development mode...')
    
    return new Promise((resolve, reject) => {
      this.appProcess = spawn('npm', ['run', 'tauri:dev'], {
        cwd: resolve(process.cwd()),
        env: {
          ...process.env,
          SURREALDB_URL: 'ws://localhost:8001',
          SURREALDB_NS: 'test',
          SURREALDB_DB: 'e2e',
          SURREALDB_USER: 'root',
          SURREALDB_PASS: 'test'
        },
        stdio: ['ignore', 'pipe', 'pipe']
      })

      let startupComplete = false

      if (this.appProcess.stdout) {
        this.appProcess.stdout.on('data', (data) => {
          const output = data.toString()
          console.log('App output:', output)
          
          // Look for indicators that the app is ready
          if (output.includes('Local:') && output.includes('1420') && !startupComplete) {
            startupComplete = true
            setTimeout(resolve, 3000) // Give app time to fully initialize
          }
        })
      }

      if (this.appProcess.stderr) {
        this.appProcess.stderr.on('data', (data) => {
          const error = data.toString()
          console.error('App error:', error)
        })
      }

      this.appProcess.on('error', (error) => {
        reject(new Error(`Failed to start Tauri app: ${error.message}`))
      })

      // Timeout after 60 seconds
      setTimeout(() => {
        if (!startupComplete) {
          reject(new Error('Tauri app failed to start within 60 seconds'))
        }
      }, 60000)
    })
  }

  /**
   * Launch built application
   */
  private async launchBuiltApp(): Promise<void> {
    console.log('🚀 Launching built Tauri app...')
    
    // Check if built app exists
    const appExists = await fs.access(this.appPath).then(() => true).catch(() => false)
    if (!appExists) {
      throw new Error(`Built app not found at: ${this.appPath}`)
    }

    return new Promise((resolve, reject) => {
      this.appProcess = spawn(this.appPath, [], {
        env: {
          ...process.env,
          SURREALDB_URL: 'ws://localhost:8001',
          SURREALDB_NS: 'test',
          SURREALDB_DB: 'e2e',
          SURREALDB_USER: 'root',
          SURREALDB_PASS: 'test'
        },
        stdio: ['ignore', 'pipe', 'pipe']
      })

      // For built apps, we don't get the same startup output
      // so we'll just wait a bit and assume it's ready
      setTimeout(resolve, 5000)

      this.appProcess.on('error', (error) => {
        reject(new Error(`Failed to start built Tauri app: ${error.message}`))
      })
    })
  }

  /**
   * Get the path to the built application
   */
  private getAppPath(): string {
    const platform = process.platform
    const basePath = resolve(process.cwd(), 'src-tauri/target/release')
    
    switch (platform) {
      case 'darwin':
        return join(basePath, 'bundle/macos/E-Fees v0.9.0.app/Contents/MacOS/e-fees')
      case 'win32':
        return join(basePath, 'e-fees.exe')
      case 'linux':
        return join(basePath, 'e-fees')
      default:
        throw new Error(`Unsupported platform: ${platform}`)
    }
  }
}

/**
 * Helper functions for Tauri-specific testing
 */
export class TauriTestHelpers {
  constructor(private page: Page) {}

  /**
   * Test keyboard shortcuts
   */
  async testKeyboardShortcut(shortcut: string): Promise<void> {
    const isMac = process.platform === 'darwin'
    const modifier = isMac ? 'Meta' : 'Control'
    await this.page.keyboard.press(`${modifier}+${shortcut}`)
  }

  /**
   * Test window positioning (4K optimization)
   */
  async testWindowPositioning(): Promise<void> {
    await this.testKeyboardShortcut('w')
    // Wait for window positioning to complete
    await this.page.waitForTimeout(1000)
  }

  /**
   * Test file system integration
   */
  async testFileOperations(projectName: string): Promise<boolean> {
    try {
      // This would test the actual file system operations
      // For now, we'll simulate the test
      const projectFolderPath = join(process.env.PROJECT_FOLDER_PATH || '/tmp', 
        `25-97101-${projectName.toLowerCase().replace(/\s+/g, '-')}`)
      
      // Check if project folder exists
      await fs.access(projectFolderPath)
      return true
    } catch (error) {
      return false
    }
  }

  /**
   * Simulate database connection loss
   */
  async simulateConnectionLoss(): Promise<void> {
    await this.page.evaluate(() => {
      // This would close the WebSocket connection
      // Implementation would depend on how the app handles connections
      console.log('Simulating connection loss')
    })
  }

  /**
   * Test native dialog interactions
   */
  async handleNativeDialog(accept = true, promptText?: string): Promise<void> {
    this.page.on('dialog', async dialog => {
      console.log(`Dialog appeared: ${dialog.message()}`)
      if (accept) {
        await dialog.accept(promptText)
      } else {
        await dialog.dismiss()
      }
    })
  }

  /**
   * Wait for specific app state
   */
  async waitForAppState(state: 'loading' | 'ready' | 'error', timeout = 10000): Promise<void> {
    await this.page.waitForFunction((expectedState) => {
      const appContainer = document.querySelector('[data-testid="app-container"]')
      if (!appContainer) return false
      
      switch (expectedState) {
        case 'loading':
          return appContainer.classList.contains('loading')
        case 'ready':
          return !appContainer.classList.contains('loading') && 
                 !appContainer.classList.contains('error')
        case 'error':
          return appContainer.classList.contains('error')
        default:
          return false
      }
    }, state, { timeout })
  }

  /**
   * Capture app metrics for performance testing
   */
  async capturePerformanceMetrics(): Promise<any> {
    const metrics = await this.page.evaluate(() => {
      return {
        timestamp: Date.now(),
        memory: (performance as any).memory ? {
          usedJSHeapSize: (performance as any).memory.usedJSHeapSize,
          totalJSHeapSize: (performance as any).memory.totalJSHeapSize,
          jsHeapSizeLimit: (performance as any).memory.jsHeapSizeLimit
        } : null,
        timing: performance.timing ? {
          navigationStart: performance.timing.navigationStart,
          loadEventEnd: performance.timing.loadEventEnd,
          domContentLoadedEventEnd: performance.timing.domContentLoadedEventEnd
        } : null
      }
    })
    
    return metrics
  }

  /**
   * Test responsive behavior
   */
  async testResponsiveBehavior(): Promise<void> {
    // Test different viewport sizes
    const viewports = [
      { width: 1024, height: 600 },  // Minimum
      { width: 1280, height: 1200 }, // Default
      { width: 1920, height: 1080 }, // Standard
      { width: 3840, height: 2160 }  // 4K
    ]

    for (const viewport of viewports) {
      await this.page.setViewportSize(viewport)
      await this.page.waitForTimeout(500) // Allow for layout adjustment
      
      // Verify core elements are still visible
      await this.page.waitForSelector('[data-testid="app-container"]')
      await this.page.waitForSelector('[data-testid="navigation"]')
    }
    
    // Reset to default
    await this.page.setViewportSize({ width: 1280, height: 1200 })
  }
}