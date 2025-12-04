import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import sveltePlugin from 'eslint-plugin-svelte';
import svelteParser from 'svelte-eslint-parser';
import tsParser from '@typescript-eslint/parser';
import globals from 'globals';

export default [
  // Base JS rules
  js.configs.recommended,

  // Global ignores
  {
    ignores: [
      '**/node_modules/**',
      '**/dist/**',
      '**/build/**',
      '**/.svelte-kit/**',
      '**/src-tauri/target/**',
      '**/coverage/**',
      '**/performance/**',
      '**/tauri-plugin-mcp/**',
      '**/*.config.js',
      '**/*.config.ts',
      'vite.config.ts',
      'svelte.config.js',
      'tailwind.config.js',
      'postcss.config.js'
    ]
  },

  // TypeScript files
  ...tseslint.configs.recommended.map(config => ({
    ...config,
    files: ['**/*.ts', '**/*.tsx']
  })),

  // TypeScript-specific settings
  {
    files: ['**/*.ts', '**/*.tsx'],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        ecmaVersion: 'latest',
        sourceType: 'module'
      },
      globals: {
        ...globals.browser,
        ...globals.es2021
      }
    },
    rules: {
      '@typescript-eslint/no-unused-vars': ['warn', {
        argsIgnorePattern: '^_',
        varsIgnorePattern: '^_'
      }],
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/no-empty-object-type': 'off',
      '@typescript-eslint/no-empty-interface': 'off',
      'no-empty': ['error', { allowEmptyCatch: true }]
    }
  },

  // Svelte files
  ...sveltePlugin.configs['flat/recommended'].map(config => ({
    ...config,
    files: ['**/*.svelte']
  })),

  // Svelte-specific settings
  {
    files: ['**/*.svelte'],
    languageOptions: {
      parser: svelteParser,
      parserOptions: {
        parser: tsParser
      },
      globals: {
        ...globals.browser,
        ...globals.es2021
      }
    },
    rules: {
      'svelte/no-at-html-tags': 'warn',
      'svelte/valid-compile': ['warn', { ignoreWarnings: true }],
      'svelte/no-unused-svelte-ignore': 'warn',
      'svelte/require-each-key': 'warn',
      'no-unused-vars': 'off',
      'no-undef': 'off', // TypeScript handles this
      'no-empty': ['error', { allowEmptyCatch: true }]
    }
  },

  // Test files - more relaxed rules
  {
    files: ['**/*.test.ts', '**/*.spec.ts', '**/tests/**/*.ts'],
    languageOptions: {
      globals: {
        ...globals.browser,
        ...globals.es2021,
        ...globals.node,
        vi: 'readonly',
        describe: 'readonly',
        it: 'readonly',
        expect: 'readonly',
        beforeEach: 'readonly',
        afterEach: 'readonly',
        beforeAll: 'readonly',
        afterAll: 'readonly'
      }
    },
    rules: {
      '@typescript-eslint/no-explicit-any': 'off',
      '@typescript-eslint/no-unused-vars': 'off',
      'no-unused-vars': 'off',
      'no-useless-escape': 'off'
    }
  }
];
