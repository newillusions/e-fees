import { invoke } from '@tauri-apps/api/core';

/**
 * Log levels enum matching Rust log crate levels
 */
export enum LogLevel {
  ERROR = 'error',
  WARN = 'warn',
  INFO = 'info',
  DEBUG = 'debug',
  TRACE = 'trace'
}

/**
 * Structured log context for better debugging
 */
export interface LogContext {
  component?: string;
  action?: string;
  userId?: string;
  requestId?: string;
  [key: string]: unknown;
}

/**
 * Log entry structure
 */
export interface LogEntry {
  level: LogLevel;
  message: string;
  context?: LogContext;
  error?: Error;
  timestamp?: Date;
}

/**
 * Logger configuration
 */
interface LoggerConfig {
  level: LogLevel;
  enableConsole: boolean;
  enableTauri: boolean;
  component?: string;
}

/**
 * Professional logging service for E-Fees application
 * 
 * Features:
 * - Multiple log levels (error, warn, info, debug, trace)
 * - Structured logging with context
 * - Integration with Tauri backend logging
 * - Console fallback for development
 * - Performance optimized
 * - TypeScript support
 * 
 * @example
 * ```typescript
 * import { logger } from '$lib/services/logger';
 * 
 * // Simple logging
 * logger.error('Database connection failed');
 * logger.warn('Retrying operation', { attempt: 3 });
 * logger.info('User logged in', { userId: 'user123' });
 * 
 * // With context
 * logger.error('API call failed', {
 *   component: 'ProjectModal',
 *   action: 'createProject',
 *   error: new Error('Network timeout')
 * });
 * 
 * // Component-specific logger
 * const componentLogger = logger.child({ component: 'ContactModal' });
 * componentLogger.info('Modal opened');
 * ```
 */
class Logger {
  private config: LoggerConfig;
  private static instance: Logger;

  constructor(config: Partial<LoggerConfig> = {}) {
    this.config = {
      level: LogLevel.INFO,
      enableConsole: import.meta.env.DEV,
      enableTauri: true,
      ...config
    };
  }

  /**
   * Get singleton logger instance
   */
  static getInstance(): Logger {
    if (!Logger.instance) {
      Logger.instance = new Logger();
    }
    return Logger.instance;
  }

  /**
   * Create a child logger with additional context
   */
  child(context: LogContext): Logger {
    const childLogger = new Logger(this.config);
    childLogger.config = {
      ...this.config,
      component: context.component || this.config.component
    };
    return childLogger;
  }

  /**
   * Set log level
   */
  setLevel(level: LogLevel): void {
    this.config.level = level;
  }

  /**
   * Check if level is enabled
   */
  private isLevelEnabled(level: LogLevel): boolean {
    const levels = [LogLevel.ERROR, LogLevel.WARN, LogLevel.INFO, LogLevel.DEBUG, LogLevel.TRACE];
    const currentIndex = levels.indexOf(this.config.level);
    const messageIndex = levels.indexOf(level);
    return messageIndex <= currentIndex;
  }

  /**
   * Format log message with context
   */
  private formatMessage(message: string, context?: LogContext, error?: Error): string {
    const parts: string[] = [];
    
    if (this.config.component) {
      parts.push(`[${this.config.component}]`);
    }
    
    if (context?.component && context.component !== this.config.component) {
      parts.push(`[${context.component}]`);
    }
    
    if (context?.action) {
      parts.push(`${context.action}:`);
    }
    
    parts.push(message);
    
    if (error) {
      parts.push(`- ${error.message}`);
    }
    
    return parts.join(' ');
  }

  /**
   * Send log to Tauri backend
   */
  private async sendToTauri(entry: LogEntry): Promise<void> {
    if (!this.config.enableTauri) return;

    try {
      await invoke('log_message', {
        level: entry.level,
        target: entry.context?.component || 'frontend',
        message: this.formatMessage(entry.message, entry.context, entry.error),
        context: entry.context ? JSON.stringify(entry.context) : null
      });
    } catch (err) {
      // Fallback to console if Tauri logging fails
      if (this.config.enableConsole) {
        console.error('Failed to send log to Tauri:', err);
        console[entry.level === LogLevel.ERROR ? 'error' : 
               entry.level === LogLevel.WARN ? 'warn' : 
               'log'](this.formatMessage(entry.message, entry.context, entry.error));
      }
    }
  }

  /**
   * Send log to console (development only)
   */
  private logToConsole(entry: LogEntry): void {
    if (!this.config.enableConsole) return;

    const message = this.formatMessage(entry.message, entry.context, entry.error);
    
    switch (entry.level) {
      case LogLevel.ERROR:
        console.error(message, entry.context, entry.error);
        break;
      case LogLevel.WARN:
        console.warn(message, entry.context);
        break;
      case LogLevel.DEBUG:
      case LogLevel.TRACE:
        console.debug(message, entry.context);
        break;
      default:
        console.log(message, entry.context);
    }
  }

  /**
   * Core logging method
   */
  private async log(entry: LogEntry): Promise<void> {
    if (!this.isLevelEnabled(entry.level)) return;

    entry.timestamp = new Date();
    
    // Always log to console in development
    this.logToConsole(entry);
    
    // Send to Tauri backend for persistent logging
    await this.sendToTauri(entry);
  }

  /**
   * Log error message
   */
  async error(message: string, contextOrError?: LogContext | Error, error?: Error): Promise<void> {
    let context: LogContext | undefined;
    let err: Error | undefined;

    if (contextOrError instanceof Error) {
      err = contextOrError;
      context = undefined;
    } else {
      context = contextOrError;
      err = error;
    }

    await this.log({
      level: LogLevel.ERROR,
      message,
      context,
      error: err
    });
  }

  /**
   * Log warning message
   */
  async warn(message: string, context?: LogContext): Promise<void> {
    await this.log({
      level: LogLevel.WARN,
      message,
      context
    });
  }

  /**
   * Log info message
   */
  async info(message: string, context?: LogContext): Promise<void> {
    await this.log({
      level: LogLevel.INFO,
      message,
      context
    });
  }

  /**
   * Log debug message
   */
  async debug(message: string, context?: LogContext): Promise<void> {
    await this.log({
      level: LogLevel.DEBUG,
      message,
      context
    });
  }

  /**
   * Log trace message (most verbose)
   */
  async trace(message: string, context?: LogContext): Promise<void> {
    await this.log({
      level: LogLevel.TRACE,
      message,
      context
    });
  }

  /**
   * Performance timer utility
   */
  timer(label: string, context?: LogContext): () => void {
    const start = performance.now();
    
    return () => {
      const duration = performance.now() - start;
      this.info(`Timer ${label}: ${duration.toFixed(2)}ms`, {
        ...context,
        duration: duration.toFixed(2)
      });
    };
  }
}

// Export singleton instance
export const logger = Logger.getInstance();

// Export convenience functions for common patterns
export const createComponentLogger = (component: string): Logger => 
  logger.child({ component });

export const logApiError = async (operation: string, error: Error, context?: LogContext): Promise<void> => 
  logger.error(`API operation failed: ${operation}`, { ...context, action: operation }, error);

export const logUserAction = async (action: string, context?: LogContext): Promise<void> => 
  logger.info(`User action: ${action}`, { ...context, action });

// Development helpers
export const enableDebugLogging = (): void => logger.setLevel(LogLevel.DEBUG);
export const enableTraceLogging = (): void => logger.setLevel(LogLevel.TRACE);

/**
 * Utility function to safely stringify objects for logging
 */
export const safeStringify = (obj: unknown): string => {
  try {
    return JSON.stringify(obj, null, 2);
  } catch (error) {
    return `[Unable to stringify: ${error instanceof Error ? error.message : 'Unknown error'}]`;
  }
};