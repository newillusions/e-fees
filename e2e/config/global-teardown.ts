import { ChildProcess } from 'child_process'
import { promises as fs } from 'fs'
import { join } from 'path'

/**
 * Global teardown for E2E tests
 * - Stops SurrealDB test instance
 * - Cleans up test artifacts and temporary files
 * - Generates final test reports
 */
async function globalTeardown() {
  console.log('🧹 Cleaning up E2E test environment...')
  
  try {
    // 1. Stop SurrealDB test instance
    const surrealdbProcess = (global as any).surrealdbTestProcess as ChildProcess
    if (surrealdbProcess && !surrealdbProcess.killed) {
      console.log('🛑 Stopping SurrealDB test instance...')
      surrealdbProcess.kill('SIGTERM')
      
      // Wait for graceful shutdown
      await new Promise(resolve => {
        surrealdbProcess.on('exit', resolve)
        setTimeout(resolve, 5000) // Force cleanup after 5s
      })
    }
    
    // 2. Clean up test project folders
    const testProjectsPath = join(process.cwd(), 'e2e/test-projects')
    try {
      await fs.rm(testProjectsPath, { recursive: true, force: true })
      console.log('🗂️  Cleaned up test project folders')
    } catch (error) {
      console.warn('Failed to clean up test project folders:', error)
    }
    
    // 3. Clean up old test artifacts
    await cleanupOldTestArtifacts()
    
    // 4. Generate test summary
    await generateTestSummary()
    
    console.log('✅ E2E test environment cleaned up successfully')
    
  } catch (error) {
    console.error('❌ Error during teardown:', error)
    process.exit(1)
  }
}

async function cleanupOldTestArtifacts() {
  const artifactPaths = [
    './e2e/test-results',
    './e2e/reports/videos',
    './e2e/reports/screenshots'
  ]
  
  for (const path of artifactPaths) {
    try {
      const stats = await fs.stat(path)
      const now = Date.now()
      const maxAge = 7 * 24 * 60 * 60 * 1000 // 7 days in milliseconds
      
      if (stats.isDirectory() && (now - stats.mtime.getTime()) > maxAge) {
        await fs.rm(path, { recursive: true, force: true })
        console.log(`🗑️  Cleaned up old artifacts: ${path}`)
      }
    } catch (error) {
      // Path doesn't exist or other error - ignore
    }
  }
}

async function generateTestSummary() {
  try {
    // Read test results if available
    const resultsPath = './e2e/reports/test-results.json'
    let testResults: any = null
    
    try {
      const resultsData = await fs.readFile(resultsPath, 'utf-8')
      testResults = JSON.parse(resultsData)
    } catch (error) {
      console.log('ℹ️  No test results found to summarize')
      return
    }
    
    // Generate summary report
    const summary = {
      timestamp: new Date().toISOString(),
      environment: {
        nodeVersion: process.version,
        platform: process.platform,
        arch: process.arch
      },
      testSuite: 'E-Fees E2E Tests',
      results: {
        total: testResults?.stats?.total || 0,
        passed: testResults?.stats?.passed || 0,
        failed: testResults?.stats?.failed || 0,
        skipped: testResults?.stats?.skipped || 0,
        duration: testResults?.stats?.duration || 0
      },
      artifacts: {
        htmlReport: './e2e/reports/html/index.html',
        junitReport: './e2e/reports/junit.xml',
        jsonResults: './e2e/reports/test-results.json'
      }
    }
    
    await fs.writeFile(
      './e2e/reports/test-summary.json',
      JSON.stringify(summary, null, 2)
    )
    
    console.log('📊 Test summary generated:')
    console.log(`   Total: ${summary.results.total}`)
    console.log(`   Passed: ${summary.results.passed}`)
    console.log(`   Failed: ${summary.results.failed}`)
    console.log(`   Duration: ${Math.round(summary.results.duration / 1000)}s`)
    
  } catch (error) {
    console.warn('Failed to generate test summary:', error)
  }
}

export default globalTeardown