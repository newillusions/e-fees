import '@testing-library/jest-dom'
import { vi, beforeEach, afterEach, beforeAll } from 'vitest'

// Ensure we're in browser environment for Svelte 5
Object.defineProperty(global, 'window', {
  value: global.window || {},
  writable: true
});

Object.defineProperty(global, 'document', {
  value: global.document || {},
  writable: true
});

// Mock Tauri API v2 calls with proper structure
Object.defineProperty(window, '__TAURI_INTERNALS__', {
  value: {
    transformCallback: vi.fn(),
    invoke: vi.fn().mockResolvedValue(null)
  },
  writable: true
});

// Mock Tauri v2 core module
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(null),
  transformCallback: vi.fn()
}));

// Mock Tauri v2 dialog plugin
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn().mockResolvedValue(null),
  save: vi.fn().mockResolvedValue(null),
  message: vi.fn().mockResolvedValue(null),
  ask: vi.fn().mockResolvedValue(false),
  confirm: vi.fn().mockResolvedValue(false)
}));

// Mock SurrealDB v1.3+ with proper named exports
vi.mock('surrealdb', () => ({
  default: vi.fn().mockImplementation(() => ({
    connect: vi.fn().mockResolvedValue(true),
    signin: vi.fn().mockResolvedValue({ token: 'mock-token' }),
    use: vi.fn().mockResolvedValue(true),
    select: vi.fn().mockResolvedValue([]),
    create: vi.fn().mockResolvedValue({ id: 'mock:id' }),
    update: vi.fn().mockResolvedValue({ id: 'mock:id' }),
    delete: vi.fn().mockResolvedValue(true),
    query: vi.fn().mockResolvedValue([[]]),
    close: vi.fn().mockResolvedValue(true)
  })),
  Surreal: vi.fn().mockImplementation(() => ({
    connect: vi.fn().mockResolvedValue(true),
    signin: vi.fn().mockResolvedValue({ token: 'mock-token' }),
    use: vi.fn().mockResolvedValue(true),
    select: vi.fn().mockResolvedValue([]),
    create: vi.fn().mockResolvedValue({ id: 'mock:id' }),
    update: vi.fn().mockResolvedValue({ id: 'mock:id' }),
    delete: vi.fn().mockResolvedValue(true),
    query: vi.fn().mockResolvedValue([[]]),
    close: vi.fn().mockResolvedValue(true)
  }))
}));

// Mock surrealdb.js v1.0+ 
vi.mock('surrealdb.js', () => ({
  default: vi.fn().mockImplementation(() => ({
    connect: vi.fn().mockResolvedValue(true),
    signin: vi.fn().mockResolvedValue({ token: 'mock-token' }),
    use: vi.fn().mockResolvedValue(true),
    select: vi.fn().mockResolvedValue([]),
    create: vi.fn().mockResolvedValue({ id: 'mock:id' }),
    update: vi.fn().mockResolvedValue({ id: 'mock:id' }),
    delete: vi.fn().mockResolvedValue(true),
    query: vi.fn().mockResolvedValue([[]]),
    close: vi.fn().mockResolvedValue(true)
  }))
}));

// Mock WebSocket for SurrealDB
global.WebSocket = class MockWebSocket {
  constructor(url: string) {
    this.url = url;
  }
  url: string;
  readyState = 1;
  send = vi.fn();
  close = vi.fn();
  addEventListener = vi.fn();
  removeEventListener = vi.fn();
} as any;

// Mock localStorage
Object.defineProperty(window, 'localStorage', {
  value: {
    getItem: vi.fn(),
    setItem: vi.fn(),
    removeItem: vi.fn(),
    clear: vi.fn(),
  },
  writable: true,
});

// Mock console to reduce noise in tests
const originalConsole = console;
global.console = {
  ...originalConsole,
  // Keep error and warn for debugging
  error: originalConsole.error,
  warn: originalConsole.warn,
  // Mock info, log, debug to reduce test noise
  info: vi.fn(),
  log: vi.fn(),
  debug: vi.fn(),
};

// Environment setup for Svelte 5 compatibility
beforeEach(() => {
  // Ensure clean DOM state for each test
  if (global.document && global.document.body) {
    global.document.body.innerHTML = '';
  }
  
  // Reset any global state that might interfere
  vi.clearAllMocks();
});

// Clean up after each test
afterEach(() => {
  vi.clearAllMocks();
  
  // Clean up DOM state
  if (global.document && global.document.body) {
    global.document.body.innerHTML = '';
  }
});

// Global error handling for tests
beforeAll(() => {
  // Suppress expected errors during testing
  const originalError = console.error;
  console.error = (...args) => {
    // Filter out known Svelte testing warnings
    const message = args[0]?.toString() || '';
    if (message.includes('lifecycle_function_unavailable') || 
        message.includes('mount(...) is not available on the server')) {
      return;
    }
    originalError.apply(console, args);
  };
});