import './styles/app.css'
import { mount } from 'svelte'
import App from './App.svelte'
import { securityMonitor, getSecurityReport } from './lib/security'

const appDiv = document.getElementById('app');
if (appDiv) {
  try {
    const app = mount(App, {
      target: appDiv,
    });

    // Initialize security monitoring in development
    if (import.meta.env.DEV) {
      
      // Make security report available globally for debugging
      (window as any).getSecurityReport = getSecurityReport;
      
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
    // Fallback to show error
    appDiv.innerHTML = `
      <div style="color: white; padding: 20px; background: linear-gradient(135deg, #000, #333); min-height: 100vh;">
        <h1>Svelte Error</h1>
        <p>Failed to load Svelte app: ${error}</p>
      </div>
    `;
  }
} else {
  console.error('Could not find app div!');
}