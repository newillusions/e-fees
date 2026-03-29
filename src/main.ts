import './styles/app.css';
import { mount } from 'svelte';
import App from './App.svelte';
import { securityMonitor, getSecurityReport } from './lib/security';

// Extend window for dev debugging tools
declare global {
  interface Window {
    getSecurityReport?: typeof getSecurityReport;
  }
}

const appDiv = document.getElementById('app');
if (appDiv) {
  try {
    const app = mount(App, {
      target: appDiv
    });

    // Initialize security monitoring in development
    if (import.meta.env.DEV) {
      // Make security report available globally for debugging
      window.getSecurityReport = getSecurityReport;

      // Log initial security validation
      setTimeout(() => {
        const report = getSecurityReport();
        if (report.validation.errors.length > 0) {
          console.error('Security Errors:', report.validation.errors);
        }
        if (report.validation.warnings.length > 0) {
          console.warn('Security Warnings:', report.validation.warnings);
        }
      }, 1000);
    }
  } catch (error) {
    console.error('Failed to create Svelte app:', error);
    // SEC-M2: Use textContent instead of innerHTML to prevent XSS
    const errorContainer = document.createElement('div');
    errorContainer.style.cssText =
      'color: white; padding: 20px; background: linear-gradient(135deg, #000, #333); min-height: 100vh;';

    const heading = document.createElement('h1');
    heading.textContent = 'Svelte Error';

    const message = document.createElement('p');
    // Safely display error message using textContent (prevents XSS)
    message.textContent = `Failed to load Svelte app: ${error instanceof Error ? error.message : String(error)}`;

    errorContainer.appendChild(heading);
    errorContainer.appendChild(message);
    appDiv.appendChild(errorContainer);
  }
} else {
  console.error('Could not find app div!');
}
