import { spawn, ChildProcess } from 'child_process'
import { chromium, FullConfig } from '@playwright/test'
import { promises as fs } from 'fs'
import { join } from 'path'

let surrealdbProcess: ChildProcess | null = null

/**
 * Global setup for E2E tests
 * - Starts isolated SurrealDB test instance  
 * - Builds Tauri application in test mode
 * - Sets up test environment variables
 */
async function globalSetup(config: FullConfig) {
  console.log('🚀 Setting up E2E test environment...')
  
  // 1. Start SurrealDB test instance
  console.log('📦 Starting SurrealDB test instance...')
  await startSurrealDBTestInstance()
  
  // 2. Wait for SurrealDB to be ready
  await waitForSurrealDB()
  
  // 3. Set up test environment variables
  process.env.SURREALDB_URL = 'ws://localhost:8001'
  process.env.SURREALDB_NS = 'test'
  process.env.SURREALDB_DB = 'e2e'
  process.env.SURREALDB_USER = 'root'
  process.env.SURREALDB_PASS = 'test'
  process.env.PROJECT_FOLDER_PATH = join(process.cwd(), 'e2e/test-projects')
  process.env.NODE_ENV = 'test'
  
  // 4. Create test project folder
  await fs.mkdir(process.env.PROJECT_FOLDER_PATH, { recursive: true })
  
  // 5. Build Tauri app in test mode (if not in dev mode)
  if (!process.env.E2E_DEV_MODE) {
    console.log('🔨 Building Tauri application for testing...')
    await buildTauriApp()
  }
  
  // 6. Initialize test database with reference data
  console.log('🗃️  Initializing test database...')
  await initializeTestDatabase()
  
  console.log('✅ E2E test environment ready!')
}

async function startSurrealDBTestInstance(): Promise<void> {
  return new Promise((resolve, reject) => {
    // Start SurrealDB in memory mode for testing
    surrealdbProcess = spawn('surreal', [
      'start',
      '--log', 'warn', // Reduce log noise
      '--user', 'root',
      '--pass', 'test', 
      '--bind', '0.0.0.0:8001',
      'memory' // In-memory database for speed
    ], {
      stdio: ['ignore', 'pipe', 'pipe'],
      env: { ...process.env }
    })

    if (surrealdbProcess.stdout) {
      surrealdbProcess.stdout.on('data', (data) => {
        const output = data.toString()
        if (output.includes('Started web server')) {
          resolve()
        }
      })
    }

    if (surrealdbProcess.stderr) {
      surrealdbProcess.stderr.on('data', (data) => {
        console.error('SurrealDB error:', data.toString())
      })
    }

    surrealdbProcess.on('error', (error) => {
      reject(new Error(`Failed to start SurrealDB: ${error.message}`))
    })

    surrealdbProcess.on('exit', (code) => {
      if (code !== 0 && code !== null) {
        reject(new Error(`SurrealDB exited with code ${code}`))
      }
    })

    // Timeout after 10 seconds
    setTimeout(() => {
      reject(new Error('SurrealDB failed to start within 10 seconds'))
    }, 10000)
  })
}

async function waitForSurrealDB(): Promise<void> {
  const maxAttempts = 10
  const delay = 1000

  for (let i = 0; i < maxAttempts; i++) {
    try {
      // Try to connect to SurrealDB
      const response = await fetch('http://localhost:8001/version')
      if (response.ok) {
        console.log('✅ SurrealDB is ready')
        return
      }
    } catch (error) {
      // Connection failed, wait and retry
    }
    
    await new Promise(resolve => setTimeout(resolve, delay))
  }
  
  throw new Error('SurrealDB failed to become ready')
}

async function buildTauriApp(): Promise<void> {
  return new Promise((resolve, reject) => {
    const buildProcess = spawn('npm', ['run', 'tauri:build'], {
      stdio: ['ignore', 'pipe', 'pipe'],
      env: {
        ...process.env,
        TAURI_ENV_SURREALDB_URL: 'ws://localhost:8001',
        TAURI_ENV_SURREALDB_NS: 'test',
        TAURI_ENV_SURREALDB_DB: 'e2e'
      }
    })

    buildProcess.on('close', (code) => {
      if (code === 0) {
        console.log('✅ Tauri build completed')
        resolve()
      } else {
        reject(new Error(`Tauri build failed with code ${code}`))
      }
    })

    buildProcess.on('error', (error) => {
      reject(new Error(`Failed to build Tauri app: ${error.message}`))
    })
  })
}

async function initializeTestDatabase(): Promise<void> {
  // Initialize with reference data that tests depend on
  const browser = await chromium.launch()
  const context = await browser.newContext()
  const page = await context.newPage()
  
  try {
    // Connect to SurrealDB and set up reference data
    await page.evaluate(async () => {
      // This would typically use the SurrealDB client
      // For now, we'll simulate the setup
      const response = await fetch('http://localhost:8001/sql', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Basic ' + btoa('root:test')
        },
        body: JSON.stringify({
          sql: `
            USE NS test DB e2e;
            
            -- Create reference data for countries
            INSERT INTO country [
              { name: "United Arab Emirates", dial_code: 971, code: "AE" },
              { name: "Saudi Arabia", dial_code: 966, code: "SA" },
              { name: "Qatar", dial_code: 974, code: "QA" }
            ];
            
            -- Create reference data for currencies  
            INSERT INTO currency [
              { code: "AED", name: "UAE Dirham", symbol: "د.إ" },
              { code: "SAR", name: "Saudi Riyal", symbol: "ر.س" },
              { code: "USD", name: "US Dollar", symbol: "$" }
            ];
          `
        })
      })
      
      if (!response.ok) {
        throw new Error('Failed to initialize test database')
      }
    })
    
    console.log('✅ Test database initialized with reference data')
  } catch (error) {
    console.error('Failed to initialize test database:', error)
    throw error
  } finally {
    await browser.close()
  }
}

// Store the SurrealDB process globally so it can be cleaned up
global.surrealdbTestProcess = surrealdbProcess

export default globalSetup