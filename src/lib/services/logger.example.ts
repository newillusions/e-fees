/**
 * Logger Usage Examples for E-Fees Application
 * 
 * This file demonstrates best practices for using the logging service
 * across different components and scenarios.
 */

import { logger, createComponentLogger, logApiError, logUserAction, enableDebugLogging } from './logger';
import { LogLevel } from './logger';

// Example 1: Basic logging in a Svelte component
export function basicLoggingExample() {
  // Simple error logging
  logger.error('Database connection failed');
  
  // Warning with context
  logger.warn('API rate limit approaching', { 
    remainingRequests: 5,
    resetTime: new Date().toISOString() 
  });
  
  // Info logging for user actions
  logger.info('User created new project', { 
    userId: 'user123',
    projectName: 'Office Building Design'
  });
  
  // Debug logging (only shown in debug mode)
  logger.debug('State updated', { newState: 'loading' });
}

// Example 2: Component-specific logger
export function componentLoggerExample() {
  const componentLogger = createComponentLogger('ProjectModal');
  
  // All logs from this logger will include [ProjectModal] prefix
  componentLogger.info('Modal opened');
  componentLogger.warn('Validation warning', { field: 'projectName' });
  componentLogger.error('Save failed', new Error('Network timeout'));
}

// Example 3: API error logging
export async function apiErrorExample() {
  try {
    // Some API call that might fail
    await fetch('/api/projects');
  } catch (error) {
    // Use helper function for consistent API error logging
    await logApiError('createProject', error as Error, {
      component: 'ProjectModal',
      userId: 'user123'
    });
  }
}

// Example 4: User action logging
export async function userActionExample() {
  // Log important user actions for analytics/debugging
  await logUserAction('projectCreated', {
    projectId: 'proj123',
    projectType: 'office',
    source: 'dashboard'
  });
  
  await logUserAction('proposalExported', {
    proposalId: 'rfp456',
    format: 'PDF',
    timestamp: Date.now()
  });
}

// Example 5: Performance timing
export async function performanceExample() {
  const timer = logger.timer('DatabaseQuery', { 
    query: 'SELECT * FROM projects',
    component: 'Dashboard' 
  });
  
  try {
    // Some long-running operation
    await new Promise(resolve => setTimeout(resolve, 1000));
  } finally {
    timer(); // Logs execution time
  }
}

// Example 6: Error handling with context
export async function errorHandlingExample() {
  try {
    const response = await fetch('/api/projects');
    
    if (!response.ok) {
      logger.error('API request failed', {
        component: 'ProjectList',
        action: 'fetchProjects',
        status: response.status,
        statusText: response.statusText,
        url: response.url
      });
      return;
    }
    
    const projects = await response.json();
    logger.info('Projects loaded successfully', {
      component: 'ProjectList',
      count: projects.length
    });
    
  } catch (error) {
    logger.error('Network error occurred', {
      component: 'ProjectList',
      action: 'fetchProjects'
    }, error as Error);
  }
}

// Example 7: Structured logging for business events
export async function businessEventExample() {
  // Project lifecycle events
  await logger.info('Project status changed', {
    component: 'StatusChangeModal',
    action: 'statusUpdate',
    projectId: 'proj123',
    oldStatus: 'draft',
    newStatus: 'active',
    userId: 'user123',
    timestamp: Date.now()
  });
  
  // Fee proposal events
  await logger.info('Fee proposal submitted', {
    component: 'ProposalModal',
    action: 'submitProposal',
    proposalId: 'rfp456',
    projectId: 'proj123',
    amount: 50000,
    currency: 'USD',
    submittedBy: 'user123',
    timestamp: Date.now()
  });
}

// Example 8: Development debugging
export function developmentDebuggingExample() {
  // Enable debug logging in development
  if (import.meta.env.DEV) {
    enableDebugLogging();
    
    logger.debug('Component props updated', {
      component: 'ProjectCard',
      props: { projectId: 'proj123', expanded: true }
    });
    
    logger.trace('Reactive statement executed', {
      component: 'Dashboard',
      variable: 'filteredProjects',
      length: 25
    });
  }
}

// Example 9: Child logger with persistent context
export function childLoggerExample() {
  const userLogger = logger.child({ 
    userId: 'user123',
    sessionId: 'sess456' 
  });
  
  // All logs from userLogger will include the user context
  userLogger.info('Started session');
  userLogger.warn('Approaching session timeout');
  userLogger.error('Session expired');
}

// Example 10: Integration with Svelte components
export class ComponentLoggingMixin {
  private logger = createComponentLogger('UnknownComponent');
  
  constructor(componentName: string) {
    this.logger = createComponentLogger(componentName);
  }
  
  async logMount() {
    await this.logger.info('Component mounted');
  }
  
  async logUnmount() {
    await this.logger.info('Component unmounted');
  }
  
  async logError(error: Error, context?: any) {
    await this.logger.error('Component error', context, error);
  }
  
  async logUserInteraction(action: string, data?: any) {
    await this.logger.info(`User interaction: ${action}`, {
      action,
      ...data
    });
  }
}

// Usage in Svelte component:
// ```svelte
// <script lang="ts">
//   import { ComponentLoggingMixin } from '$lib/services/logger.example';
//   
//   const logging = new ComponentLoggingMixin('ProjectCard');
//   
//   onMount(() => {
//     logging.logMount();
//   });
//   
//   onDestroy(() => {
//     logging.logUnmount();
//   });
//   
//   function handleClick() {
//     logging.logUserInteraction('cardClicked', { projectId });
//   }
// </script>
// ```