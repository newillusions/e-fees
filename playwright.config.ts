// 🚨 CRITICAL WARNING: THIS FILE IS DISABLED 🚨
// 
// Browser-based testing (Playwright) DOES NOT WORK for Tauri desktop applications.
// This configuration file is kept for reference only but should NEVER be used.
//
// ❌ WRONG: Browser testing cannot connect to Tauri backend
// ✅ CORRECT: Use Tauri MCP server testing (see e2e-mcp/ directory)
//
// For E2E testing, use: npm run test:e2e:mcp
// See CRITICAL_DIRECTIVE_TAURI_MCP_ONLY.md for mandatory requirements.

throw new Error(`
🚨 CRITICAL ERROR: Browser testing attempted for Tauri application!

This configuration file exists for reference only.
Browser-based testing DOES NOT WORK for Tauri desktop applications.

Use the correct approach:
- npm run test:e2e:mcp (Tauri MCP server testing)
- See CRITICAL_DIRECTIVE_TAURI_MCP_ONLY.md
- Use e2e-mcp/ directory for all E2E testing

NEVER use Playwright/Puppeteer/Selenium for Tauri E2E testing.
`);
  globalTimeout: 10 * 60 * 1000, // 10 minutes for entire test suite
  expect: {
    timeout: 10000 // 10s for expect assertions
  },
  
  // Test execution settings
  fullyParallel: false, // Desktop apps need sequential execution
  workers: 1, // Single worker to avoid conflicts with app instances
  retries: process.env.CI ? 2 : 1, // Retry failed tests in CI
  reporter: [
    ['html', { outputFolder: './e2e/reports/html' }],
    ['json', { outputFile: './e2e/reports/test-results.json' }],
    ['junit', { outputFile: './e2e/reports/junit.xml' }],
    ...(process.env.CI ? [['github']] : [['list']])
  ],
  
  // Global setup and teardown
  globalSetup: './e2e/config/global-setup.ts',
  globalTeardown: './e2e/config/global-teardown.ts',
  
  use: {
    // Base URL for the Tauri app (webview content)
    baseURL: 'http://localhost:1420',
    
    // Tracing and debugging
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    
    // Tauri-specific settings
    ignoreHTTPSErrors: true,
    bypassCSP: true,
    
    // Viewport settings for consistent testing
    viewport: { width: 1280, height: 1200 },
    
    // Test context settings
    contextOptions: {
      recordVideo: {
        dir: './e2e/test-results/videos',
        size: { width: 1280, height: 1200 }
      }
    }
  },

  projects: [
    {
      name: 'e2e-setup',
      testMatch: '**/setup.spec.ts',
      use: {
        ...devices['Desktop Chrome']
      }
    },
    {
      name: 'app-startup',
      testMatch: '**/app-startup.spec.ts',
      dependencies: ['e2e-setup'],
      use: {
        ...devices['Desktop Chrome']
      }
    },
    {
      name: 'project-management',
      testMatch: '**/project-management.spec.ts', 
      dependencies: ['e2e-setup'],
      use: {
        ...devices['Desktop Chrome']
      }
    },
    {
      name: 'company-contact',
      testMatch: '**/company-contact.spec.ts',
      dependencies: ['e2e-setup'],
      use: {
        ...devices['Desktop Chrome']
      }
    },
    {
      name: 'proposal-workflows',
      testMatch: '**/proposal-workflows.spec.ts',
      dependencies: ['e2e-setup'],
      use: {
        ...devices['Desktop Chrome']
      }
    },
    {
      name: 'navigation',
      testMatch: '**/navigation.spec.ts',
      dependencies: ['e2e-setup'],
      use: {
        ...devices['Desktop Chrome']
      }
    },
    {
      name: 'error-scenarios',
      testMatch: '**/error-scenarios.spec.ts',
      dependencies: ['e2e-setup'],
      use: {
        ...devices['Desktop Chrome']
      }
    }
  ],

  // Web server configuration for development mode
  webServer: process.env.E2E_DEV_MODE ? {
    command: 'npm run tauri:dev',
    port: 1420,
    reuseExistingServer: false,
    timeout: 60000, // Allow time for Tauri to build and start
    stdout: 'pipe',
    stderr: 'pipe'
  } : undefined,

  // Test metadata
  metadata: {
    'test-suite': 'E-Fees E2E Tests',
    'application': 'E-Fees v0.9.0',
    'framework': 'Tauri v2 + Svelte 5',
    'database': 'SurrealDB'
  }
})