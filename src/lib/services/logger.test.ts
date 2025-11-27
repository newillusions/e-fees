import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { logger, LogLevel, createComponentLogger, logApiError, logUserAction } from './logger';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

const mockInvoke = vi.mocked((await import('@tauri-apps/api/core')).invoke);

// Mock console methods
const originalError = console.error;
const originalWarn = console.warn;
const originalLog = console.log;

beforeEach(() => {
  vi.clearAllMocks();
  console.error = vi.fn();
  console.warn = vi.fn();
  console.log = vi.fn();
  
  // Reset logger to default state
  logger.setLevel(LogLevel.INFO);
});

afterEach(() => {
  console.error = originalError;
  console.warn = originalWarn;
  console.log = originalLog;
});

describe('Logger Service', () => {
  describe('Basic Logging', () => {
    it('should log error messages', async () => {
      await logger.error('Test error message');
      
      expect(mockInvoke).toHaveBeenCalledWith('log_message', {
        level: 'error',
        target: 'frontend',
        message: 'Test error message',
        context: null
      });
    });

    it('should log with context', async () => {
      await logger.error('Test error', { component: 'TestComponent', userId: '123' });
      
      expect(mockInvoke).toHaveBeenCalledWith('log_message', {
        level: 'error',
        target: 'TestComponent',
        message: '[TestComponent] Test error',
        context: JSON.stringify({ component: 'TestComponent', userId: '123' })
      });
    });

    it('should log with error object', async () => {
      const error = new Error('Test error');
      await logger.error('Operation failed', { action: 'test' }, error);
      
      expect(mockInvoke).toHaveBeenCalledWith('log_message', {
        level: 'error',
        target: 'frontend', 
        message: 'test: Operation failed - Test error',
        context: JSON.stringify({ action: 'test' })
      });
    });

    it('should support all log levels', async () => {
      logger.setLevel(LogLevel.TRACE);
      
      await logger.error('Error message');
      await logger.warn('Warning message');  
      await logger.info('Info message');
      await logger.debug('Debug message');
      await logger.trace('Trace message');
      
      expect(mockInvoke).toHaveBeenCalledTimes(5);
      expect(mockInvoke).toHaveBeenNthCalledWith(1, 'log_message', expect.objectContaining({ level: 'error' }));
      expect(mockInvoke).toHaveBeenNthCalledWith(2, 'log_message', expect.objectContaining({ level: 'warn' }));
      expect(mockInvoke).toHaveBeenNthCalledWith(3, 'log_message', expect.objectContaining({ level: 'info' }));
      expect(mockInvoke).toHaveBeenNthCalledWith(4, 'log_message', expect.objectContaining({ level: 'debug' }));
      expect(mockInvoke).toHaveBeenNthCalledWith(5, 'log_message', expect.objectContaining({ level: 'trace' }));
    });
  });

  describe('Log Level Filtering', () => {
    it('should respect log level settings', async () => {
      logger.setLevel(LogLevel.WARN);
      
      await logger.error('Should log');
      await logger.warn('Should log');
      await logger.info('Should not log');
      await logger.debug('Should not log');
      
      expect(mockInvoke).toHaveBeenCalledTimes(2);
    });
  });

  describe('Component Logger', () => {
    it('should create component logger with context', async () => {
      const componentLogger = createComponentLogger('TestModal');
      await componentLogger.info('Test message');
      
      expect(mockInvoke).toHaveBeenCalledWith('log_message', {
        level: 'info',
        target: 'frontend',
        message: '[TestModal] Test message',
        context: null
      });
    });

    it('should support child logger context', async () => {
      const childLogger = logger.child({ component: 'ParentComponent' });
      await childLogger.info('Child message');
      
      expect(mockInvoke).toHaveBeenCalledWith('log_message', {
        level: 'info',
        target: 'frontend', 
        message: '[ParentComponent] Child message',
        context: null
      });
    });
  });

  describe('Helper Functions', () => {
    it('should log API errors correctly', async () => {
      const error = new Error('Network timeout');
      await logApiError('createProject', error, { userId: '123' });
      
      expect(mockInvoke).toHaveBeenCalledWith('log_message', {
        level: 'error',
        target: 'frontend',
        message: 'createProject: API operation failed: createProject - Network timeout',
        context: JSON.stringify({ userId: '123', action: 'createProject' })
      });
    });

    it('should log user actions correctly', async () => {
      await logUserAction('buttonClicked', { button: 'save', screen: 'project' });
      
      expect(mockInvoke).toHaveBeenCalledWith('log_message', {
        level: 'info',
        target: 'frontend',
        message: 'buttonClicked: User action: buttonClicked',
        context: JSON.stringify({ button: 'save', screen: 'project', action: 'buttonClicked' })
      });
    });
  });

  describe('Performance Timer', () => {
    it('should measure and log execution time', async () => {
      vi.useFakeTimers();
      
      const timer = logger.timer('TestOperation', { component: 'TestComponent' });
      
      // Simulate 100ms delay
      vi.advanceTimersByTime(100);
      
      timer();
      
      // Should only call once for the completion log (optimized implementation)
      expect(mockInvoke).toHaveBeenCalledWith('log_message', expect.objectContaining({
        level: 'info',
        target: 'TestComponent',
        message: expect.stringContaining('Timer TestOperation:'),
        context: expect.stringContaining('duration')
      }));
      
      vi.useRealTimers();
    });
  });

  describe('Fallback Behavior', () => {
    it('should fallback to console when Tauri invoke fails', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Tauri not available'));
      
      await logger.error('Test error message');
      
      expect(console.error).toHaveBeenCalledWith(
        'Failed to send log to Tauri:',
        expect.any(Error)
      );
      expect(console.error).toHaveBeenCalledWith('Test error message');
    });
  });
});

describe('Message Formatting', () => {
  it('should format messages with component and action', async () => {
    const componentLogger = createComponentLogger('TestModal');
    await componentLogger.info('Operation completed', { action: 'save' });
    
    expect(mockInvoke).toHaveBeenCalledWith('log_message', {
      level: 'info',
      target: 'frontend',
      message: '[TestModal] save: Operation completed',
      context: JSON.stringify({ action: 'save' })
    });
  });

  it('should handle nested component context', async () => {
    const parentLogger = logger.child({ component: 'ParentModal' });
    await parentLogger.info('Child action', { component: 'ChildModal', action: 'click' });
    
    expect(mockInvoke).toHaveBeenCalledWith('log_message', {
      level: 'info', 
      target: 'ChildModal',
      message: '[ParentModal] [ChildModal] click: Child action',
      context: JSON.stringify({ component: 'ChildModal', action: 'click' })
    });
  });
});