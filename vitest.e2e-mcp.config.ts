import { defineConfig } from 'vitest/config'
import { resolve } from 'path'

export default defineConfig({
  test: {
    globals: true,
    environment: 'node', // E2E tests don't need DOM environment
    setupFiles: [],
    pool: 'threads',
    poolOptions: {
      threads: {
        singleThread: true
      }
    },
    // Include E2E MCP test files
    include: ['e2e-mcp/tests/*.mcp.ts', 'e2e-mcp/tests/**/*.test.ts'],
    exclude: ['node_modules/**', 'dist/**'],
    testTimeout: 60000, // 60 seconds for E2E tests
    hookTimeout: 30000  // 30 seconds for setup/teardown
  },
  resolve: {
    alias: {
      '$lib': resolve(__dirname, './src/lib'),
      '$routes': resolve(__dirname, './src/routes'),
      '$types': resolve(__dirname, './src/types')
    }
  }
})