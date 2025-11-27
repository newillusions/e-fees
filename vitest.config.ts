import { defineConfig } from 'vitest/config'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { resolve } from 'path'

export default defineConfig({
  plugins: [svelte({ 
    hot: !process.env.VITEST,
    // Force client-side compilation during testing
    compilerOptions: {
      hydratable: false,
      dev: true
    },
    // Configure for testing environment  
    emitCss: false,
    onwarn: (warning, handler) => {
      // Suppress warnings during testing
      if (process.env.VITEST) return
      handler(warning)
    }
  })],
  // Force client-side resolution for Svelte during testing
  define: {
    'process.env.VITEST': JSON.stringify(process.env.VITEST || 'true')
  },
  ssr: {
    // Exclude Svelte from SSR during testing
    noExternal: process.env.VITEST ? [] : undefined
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/tests/setup.ts'],
    // Ensure proper DOM environment for Svelte 5
    environmentOptions: {
      jsdom: {
        resources: 'usable',
        pretendToBeVisual: true
      }
    },
    // Force browser condition for Svelte 5
    pool: 'threads',
    poolOptions: {
      threads: {
        singleThread: true
      }
    },
    include: ['src/**/*.{test,spec}.{js,ts}', 'tests/**/*.{test,spec}.{js,ts}'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html', 'lcov'],
      reportsDirectory: './coverage',
      exclude: [
        'node_modules/',
        'src/tests/',
        '**/*.d.ts',
        '**/*.config.{js,ts}',
        '**/*.test.{js,ts}',
        '**/*.spec.{js,ts}',
        'dist/',
        'src-tauri/',
        'archive/',
        'docs/',
        'src/main.ts' // Entry point, minimal logic
      ],
      thresholds: {
        global: {
          branches: 80,
          functions: 80,
          lines: 80,
          statements: 80
        },
        // Per-file thresholds for critical business logic
        'src/lib/api.ts': {
          branches: 90,
          functions: 90,
          lines: 90,
          statements: 90
        },
        'src/lib/stores.ts': {
          branches: 85,
          functions: 85,
          lines: 85,
          statements: 85
        }
      }
    }
  },
  resolve: {
    alias: {
      '$lib': resolve(__dirname, './src/lib'),
      '$routes': resolve(__dirname, './src/routes'),
      '$types': resolve(__dirname, './src/types')
    },
    // Force browser conditions for Svelte 5 testing
    conditions: process.env.VITEST ? ['browser'] : undefined
  }
})