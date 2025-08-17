/**
 * Comprehensive CSP and Security Testing Script
 * Run this in the browser console to test security implementations
 */

console.log('🔒 Starting Comprehensive Security Tests...\n');

// Test 1: CSP Configuration
console.log('📋 Test 1: Content Security Policy Configuration');
const metaTags = Array.from(document.querySelectorAll('meta[http-equiv]'));
const securityHeaders = metaTags.filter(tag => 
  ['Content-Security-Policy', 'X-Content-Type-Options', 'X-Frame-Options', 'X-XSS-Protection'].includes(tag.getAttribute('http-equiv'))
);

console.log('Security headers found:', securityHeaders.length);
securityHeaders.forEach(tag => {
  console.log(`  ✓ ${tag.getAttribute('http-equiv')}: ${tag.getAttribute('content')}`);
});

// Test 2: CSP Violation Monitoring
console.log('\n📋 Test 2: CSP Violation Monitoring');
if (typeof window.getSecurityReport === 'function') {
  const report = window.getSecurityReport();
  console.log('Security monitor active:', true);
  console.log('Current violations:', report.violations.length);
  console.log('Validation results:', report.validation);
  
  // Display any existing violations
  if (report.violations.length > 0) {
    console.log('Recent violations:');
    report.violations.slice(-5).forEach((violation, index) => {
      console.log(`  ${index + 1}. ${violation.type} - ${violation.timestamp}`);
    });
  }
} else {
  console.log('❌ Security monitor not found - check main.ts import');
}

// Test 3: Try to trigger CSP violations (these should be blocked)
console.log('\n📋 Test 3: CSP Violation Tests (should be blocked)');

// Test inline script injection
try {
  const testScript = document.createElement('script');
  testScript.textContent = 'console.log("Inline script executed");';
  document.head.appendChild(testScript);
  console.log('❌ Inline script was NOT blocked by CSP');
} catch (error) {
  console.log('✓ Inline script blocked by CSP:', error.message);
}

// Test external script loading
try {
  const externalScript = document.createElement('script');
  externalScript.src = 'https://evil.example.com/malicious.js';
  document.head.appendChild(externalScript);
  console.log('⚠️  External script element created (may be blocked by CSP)');
} catch (error) {
  console.log('✓ External script blocked:', error.message);
}

// Test eval usage
console.log('\n📋 Test 4: Eval Usage Detection');
try {
  eval('console.log("Eval executed - this may trigger security monitor")');
  console.log('⚠️  Eval was executed (monitor should have logged this)');
} catch (error) {
  console.log('✓ Eval blocked by CSP:', error.message);
}

// Test 5: Security Headers Validation
console.log('\n📋 Test 5: Security Headers Validation');
const requiredHeaders = [
  'X-Content-Type-Options',
  'X-Frame-Options', 
  'X-XSS-Protection',
  'Referrer-Policy',
  'Permissions-Policy'
];

const foundHeaders = new Set();
metaTags.forEach(tag => {
  const headerName = tag.getAttribute('http-equiv');
  if (requiredHeaders.includes(headerName)) {
    foundHeaders.add(headerName);
    console.log(`  ✓ ${headerName}: ${tag.getAttribute('content')}`);
  }
});

const missingHeaders = requiredHeaders.filter(header => !foundHeaders.has(header));
if (missingHeaders.length > 0) {
  console.log('❌ Missing security headers:', missingHeaders);
} else {
  console.log('✓ All required security headers present');
}

// Test 6: Resource Loading Validation
console.log('\n📋 Test 6: Resource Loading Validation');
const resources = {
  images: document.querySelectorAll('img[src]'),
  scripts: document.querySelectorAll('script[src]'),
  stylesheets: document.querySelectorAll('link[rel="stylesheet"]'),
  iframes: document.querySelectorAll('iframe[src]')
};

Object.entries(resources).forEach(([type, elements]) => {
  console.log(`${type}: ${elements.length} found`);
  Array.from(elements).forEach((element, index) => {
    const src = element.src || element.href;
    if (src) {
      const url = new URL(src, window.location.origin);
      if (url.origin !== window.location.origin && !url.protocol.includes('tauri')) {
        console.log(`  ⚠️  External ${type}: ${src}`);
      } else {
        console.log(`  ✓ Safe ${type}: ${src}`);
      }
    }
  });
});

// Test 7: Dangerous DOM Patterns
console.log('\n📋 Test 7: Dangerous DOM Patterns');
const dangerousPatterns = [
  { selector: '[onclick]', name: 'Inline onclick handlers' },
  { selector: '[onload]', name: 'Inline onload handlers' },
  { selector: '[onerror]', name: 'Inline onerror handlers' },
  { selector: '[href^="javascript:"]', name: 'JavaScript URLs' },
  { selector: 'iframe:not([sandbox])', name: 'Unsandboxed iframes' }
];

dangerousPatterns.forEach(pattern => {
  const elements = document.querySelectorAll(pattern.selector);
  if (elements.length > 0) {
    console.log(`❌ Found ${elements.length} ${pattern.name}`);
  } else {
    console.log(`✓ No ${pattern.name} found`);
  }
});

// Test 8: Mixed Content Detection
console.log('\n📋 Test 8: Mixed Content Detection');
if (window.location.protocol === 'https:') {
  const httpResources = document.querySelectorAll('[src^="http:"], [href^="http:"]');
  if (httpResources.length > 0) {
    console.log(`❌ Found ${httpResources.length} HTTP resources in HTTPS context`);
    Array.from(httpResources).slice(0, 5).forEach(resource => {
      console.log(`  - ${resource.tagName}: ${resource.src || resource.href}`);
    });
  } else {
    console.log('✓ No mixed content detected');
  }
} else {
  console.log('ℹ️  Not HTTPS - mixed content check skipped');
}

// Test 9: Tauri Security Context
console.log('\n📋 Test 9: Tauri Security Context');
if (window.__TAURI__) {
  console.log('✓ Tauri context available');
  
  // Check if dangerous APIs are accessible
  const tauriAPIs = ['fs', 'shell', 'http', 'dialog'];
  tauriAPIs.forEach(api => {
    if (window.__TAURI__[api]) {
      console.log(`  ⚠️  Tauri ${api} API accessible`);
    } else {
      console.log(`  ✓ Tauri ${api} API not accessible`);
    }
  });
} else {
  console.log('ℹ️  Tauri context not available (normal in dev mode)');
}

// Test 10: Performance and Security Metrics
console.log('\n📋 Test 10: Performance and Security Metrics');
if (window.performance && window.performance.getEntriesByType) {
  const navigationEntries = window.performance.getEntriesByType('navigation');
  if (navigationEntries.length > 0) {
    const nav = navigationEntries[0];
    console.log(`Page load time: ${Math.round(nav.loadEventEnd - nav.fetchStart)}ms`);
    console.log(`DOM ready time: ${Math.round(nav.domContentLoadedEventEnd - nav.fetchStart)}ms`);
  }
  
  const resourceEntries = window.performance.getEntriesByType('resource');
  const externalResources = resourceEntries.filter(entry => 
    !entry.name.includes(window.location.origin) && 
    !entry.name.includes('tauri://')
  );
  
  if (externalResources.length > 0) {
    console.log(`⚠️  ${externalResources.length} external resources loaded`);
  } else {
    console.log('✓ No external resources detected');
  }
}

// Summary
console.log('\n🔒 Security Test Summary');
console.log('========================================');
console.log('Run this script periodically to monitor security posture');
console.log('Check browser console for any CSP violations');
console.log('Monitor network tab for unexpected external requests');

// Wait and check for delayed violations
setTimeout(() => {
  console.log('\n📋 Delayed Violation Check (after 5 seconds)');
  if (typeof window.getSecurityReport === 'function') {
    const report = window.getSecurityReport();
    if (report.violations.length > 0) {
      console.log(`New violations detected: ${report.violations.length}`);
      console.log('Summary:', report.summary);
    } else {
      console.log('✓ No violations detected during test period');
    }
  }
}, 5000);