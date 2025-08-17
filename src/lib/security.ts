/**
 * Security utilities for CSP violation reporting and monitoring
 */

export interface CSPViolation {
  blockedURI: string;
  columnNumber: number;
  disposition: 'enforce' | 'report';
  documentURI: string;
  effectiveDirective: string;
  lineNumber: number;
  originalPolicy: string;
  referrer: string;
  sample: string;
  sourceFile: string;
  statusCode: number;
  violatedDirective: string;
}

export interface SecurityEvent {
  type: 'csp-violation' | 'unsafe-eval' | 'inline-script' | 'external-resource';
  timestamp: Date;
  details: CSPViolation | Record<string, any>;
  userAgent: string;
  url: string;
}

class SecurityMonitor {
  private violations: SecurityEvent[] = [];
  private maxViolations = 100; // Keep last 100 violations
  
  constructor() {
    this.initCSPReporting();
    this.initSecurityHeaders();
  }

  private initCSPReporting(): void {
    // Listen for CSP violations
    document.addEventListener('securitypolicyviolation', (event) => {
      const violation: CSPViolation = {
        blockedURI: event.blockedURI,
        columnNumber: event.columnNumber,
        disposition: event.disposition,
        documentURI: event.documentURI,
        effectiveDirective: event.effectiveDirective,
        lineNumber: event.lineNumber,
        originalPolicy: event.originalPolicy,
        referrer: event.referrer,
        sample: event.sample,
        sourceFile: event.sourceFile,
        statusCode: event.statusCode,
        violatedDirective: event.violatedDirective
      };

      this.reportViolation(violation);
    });
  }

  private initSecurityHeaders(): void {
    // Monitor for security-related headers and configurations
    if (window.location.protocol === 'http:' && window.location.hostname !== 'localhost') {
      console.warn('Security Warning: Application is running over HTTP instead of HTTPS');
    }

    // Check for potentially unsafe practices
    this.checkForUnsafePractices();
  }

  private checkForUnsafePractices(): void {
    // Check for eval usage
    const originalEval = window.eval;
    window.eval = function(code: string) {
      securityMonitor.reportSecurityEvent({
        type: 'unsafe-eval',
        timestamp: new Date(),
        details: { code: code.substring(0, 100) },
        userAgent: navigator.userAgent,
        url: window.location.href
      });
      console.warn('Security Warning: eval() usage detected');
      return originalEval.call(this, code);
    };

    // Check for inline script injection attempts
    const observer = new MutationObserver((mutations) => {
      mutations.forEach((mutation) => {
        mutation.addedNodes.forEach((node) => {
          if (node.nodeType === Node.ELEMENT_NODE) {
            const element = node as Element;
            if (element.tagName === 'SCRIPT' && element.textContent) {
              securityMonitor.reportSecurityEvent({
                type: 'inline-script',
                timestamp: new Date(),
                details: { 
                  script: element.textContent.substring(0, 100),
                  src: element.getAttribute('src')
                },
                userAgent: navigator.userAgent,
                url: window.location.href
              });
            }
          }
        });
      });
    });

    observer.observe(document.body, {
      childList: true,
      subtree: true
    });
  }

  private reportViolation(violation: CSPViolation): void {
    const securityEvent: SecurityEvent = {
      type: 'csp-violation',
      timestamp: new Date(),
      details: violation,
      userAgent: navigator.userAgent,
      url: window.location.href
    };

    this.reportSecurityEvent(securityEvent);
  }

  public reportSecurityEvent(event: SecurityEvent): void {
    // Add to violations array
    this.violations.push(event);
    
    // Keep only the last maxViolations
    if (this.violations.length > this.maxViolations) {
      this.violations.shift();
    }

    // Log to console in development
    if (import.meta.env.DEV) {
      console.warn('Security Event:', event);
    }

    // In a real application, you might want to send this to a security monitoring service
    // this.sendToSecurityService(event);
  }

  public getViolations(): SecurityEvent[] {
    return [...this.violations];
  }

  public getViolationsSummary(): Record<string, number> {
    const summary: Record<string, number> = {};
    
    this.violations.forEach(violation => {
      if (violation.type === 'csp-violation') {
        const directive = (violation.details as CSPViolation).violatedDirective;
        summary[directive] = (summary[directive] || 0) + 1;
      } else {
        summary[violation.type] = (summary[violation.type] || 0) + 1;
      }
    });

    return summary;
  }

  public clearViolations(): void {
    this.violations = [];
  }

  // Security best practices validation
  public validateSecurityConfiguration(): {
    passed: boolean;
    warnings: string[];
    errors: string[];
  } {
    const warnings: string[] = [];
    const errors: string[] = [];

    // Check CSP configuration
    const metaCSP = document.querySelector('meta[http-equiv="Content-Security-Policy"]');
    if (!metaCSP) {
      warnings.push('No CSP meta tag found - relying on Tauri CSP configuration');
    }

    // Check for mixed content
    if (window.location.protocol === 'https:') {
      // Check for HTTP resources in HTTPS context
      const httpResources = document.querySelectorAll('img[src^="http:"], script[src^="http:"], link[href^="http:"]');
      if (httpResources.length > 0) {
        errors.push(`Found ${httpResources.length} HTTP resources in HTTPS context (mixed content)`);
      }
    }

    // Check for inline event handlers
    const inlineHandlers = document.querySelectorAll('[onclick], [onload], [onerror]');
    if (inlineHandlers.length > 0) {
      warnings.push(`Found ${inlineHandlers.length} inline event handlers (potential CSP violations)`);
    }

    // Check for external domains
    const externalLinks = document.querySelectorAll('a[href^="http"]:not([href*="localhost"]):not([href*="127.0.0.1"])');
    if (externalLinks.length > 0) {
      warnings.push(`Found ${externalLinks.length} external links - ensure they are trusted`);
    }

    return {
      passed: errors.length === 0,
      warnings,
      errors
    };
  }
}

// Create singleton instance
export const securityMonitor = new SecurityMonitor();

// Export utility functions
export const getSecurityReport = () => ({
  violations: securityMonitor.getViolations(),
  summary: securityMonitor.getViolationsSummary(),
  validation: securityMonitor.validateSecurityConfiguration()
});

export const clearSecurityLog = () => securityMonitor.clearViolations();