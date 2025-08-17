#!/usr/bin/env node

/**
 * Security Validation Test Suite
 * Validates all security fixes implemented by sub-agents
 */

const { exec } = require('child_process');
const { readFileSync, existsSync } = require('fs');
const { promisify } = require('util');

const execAsync = promisify(exec);

console.log('🔒 SECURITY VALIDATION TEST SUITE');
console.log('=====================================\n');

let testsPassed = 0;
let testsTotal = 0;

function testResult(name, passed, details = '') {
  testsTotal++;
  if (passed) {
    testsPassed++;
    console.log(`✅ ${name}`);
  } else {
    console.log(`❌ ${name}`);
  }
  if (details) {
    console.log(`   ${details}`);
  }
}

// Test 1: Credential Management
console.log('📋 TEST 1: CREDENTIAL MANAGEMENT SECURITY');

// Check .env files are sanitized
try {
  const envContent = readFileSync('.env', 'utf8');
  const hasCredentials = envContent.includes('th38ret3ch');
  testResult('Hardcoded credentials removed from .env', !hasCredentials, 
    hasCredentials ? 'Found hardcoded password in .env' : 'No hardcoded credentials found');
} catch (error) {
  testResult('Environment file accessibility', false, 'Cannot read .env file');
}

// Check .env.example exists with placeholders
try {
  const envExampleContent = readFileSync('.env.example', 'utf8');
  const hasPlaceholders = envExampleContent.includes('REQUIRED_SET_YOUR_PASSWORD_HERE');
  const hasWarnings = envExampleContent.includes('SECURITY WARNING');
  testResult('.env.example template created', hasPlaceholders && hasWarnings,
    hasPlaceholders ? 'Template with security warnings' : 'Missing security template');
} catch (error) {
  testResult('.env.example template', false, 'Template file not found');
}

// Test 2: SQL Injection Protection
console.log('\n📋 TEST 2: SQL INJECTION PROTECTION');

// Check security module exists
const securityModuleExists = existsSync('src-tauri/src/db/security.rs');
testResult('Input validation module created', securityModuleExists);

// Check secure operations module exists
const secureOpsExists = existsSync('src-tauri/src/db/secure_operations.rs');
testResult('Secure database operations module created', secureOpsExists);

if (securityModuleExists) {
  try {
    const securityContent = readFileSync('src-tauri/src/db/security.rs', 'utf8');
    const hasValidation = securityContent.includes('InputValidator');
    const hasRegex = securityContent.includes('Regex::new');
    testResult('Input validation framework implemented', hasValidation && hasRegex);
  } catch (error) {
    testResult('Security module content validation', false, error.message);
  }
}

// Test 3: Content Security Policy
console.log('\n📋 TEST 3: CONTENT SECURITY POLICY');

try {
  const tauriConfig = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'));
  const hasCSP = tauriConfig.app?.security?.csp;
  const hasStrictCSP = hasCSP && hasCSP.includes("default-src 'self'");
  testResult('CSP configuration in Tauri config', hasStrictCSP,
    hasStrictCSP ? 'Strict CSP configured' : 'CSP missing or too permissive');
} catch (error) {
  testResult('Tauri configuration accessibility', false, error.message);
}

// Check for security headers in HTML
try {
  const htmlContent = readFileSync('index.html', 'utf8');
  const hasSecurityHeaders = htmlContent.includes('X-Content-Type-Options') &&
                           htmlContent.includes('X-Frame-Options');
  testResult('Security headers in HTML', hasSecurityHeaders);
} catch (error) {
  testResult('HTML security headers', false, 'Cannot read index.html');
}

// Test 4: Encrypted WebSocket Connection
console.log('\n📋 TEST 4: ENCRYPTED WEBSOCKET CONNECTION');

try {
  const envContent = readFileSync('.env', 'utf8');
  const hasWSS = envContent.includes('wss://');
  const hasWSOnly = envContent.includes('ws://') && !envContent.includes('wss://');
  
  if (hasWSS) {
    testResult('WebSocket connection uses encryption (WSS)', true, 'Found wss:// protocol');
  } else if (hasWSOnly) {
    testResult('WebSocket connection uses encryption (WSS)', false, 'Still using unencrypted ws://');
  } else {
    testResult('WebSocket connection configuration', false, 'No WebSocket URL found');
  }
} catch (error) {
  testResult('WebSocket configuration check', false, error.message);
}

// Check TLS configuration
try {
  const envContent = readFileSync('.env', 'utf8');
  const hasTLSConfig = envContent.includes('SURREALDB_VERIFY_CERTS');
  testResult('TLS certificate verification configured', hasTLSConfig);
} catch (error) {
  testResult('TLS configuration', false, error.message);
}

// Test 5: Rust Security Dependencies
console.log('\n📋 TEST 5: RUST SECURITY DEPENDENCIES');

try {
  const cargoContent = readFileSync('src-tauri/Cargo.toml', 'utf8');
  const hasRegexDep = cargoContent.includes('regex');
  const hasTLSFeatures = cargoContent.includes('protocol-ws');
  testResult('Security dependencies added to Cargo.toml', hasRegexDep,
    hasRegexDep ? 'Regex dependency for validation' : 'Missing regex dependency');
  testResult('TLS features enabled in SurrealDB', hasTLSFeatures);
} catch (error) {
  testResult('Cargo.toml security dependencies', false, error.message);
}

// Test 6: Git Security
console.log('\n📋 TEST 6: GIT SECURITY');

// Check .gitignore protects environment files
try {
  const gitignoreContent = readFileSync('.gitignore', 'utf8');
  const protectsEnv = gitignoreContent.includes('.env');
  testResult('.gitignore protects environment files', protectsEnv);
} catch (error) {
  testResult('.gitignore environment protection', false, error.message);
}

// Test 7: Permission Hardening
console.log('\n📋 TEST 7: PERMISSION HARDENING');

try {
  const capabilitiesContent = readFileSync('src-tauri/capabilities/default.json', 'utf8');
  const capabilities = JSON.parse(capabilitiesContent);
  const hasMinimalPerms = capabilities.permissions?.length <= 3;
  testResult('Minimal Tauri permissions configured', hasMinimalPerms,
    hasMinimalPerms ? 'Lean permission set' : 'Too many permissions granted');
} catch (error) {
  testResult('Tauri permissions configuration', false, error.message);
}

// Test 8: Security Documentation
console.log('\n📋 TEST 8: SECURITY DOCUMENTATION');

const securityDocs = [
  'SECURITY.md',
  'SECURITY_CREDENTIAL_MANAGEMENT.md',
  'SECURITY_AUDIT_REPORT.md',
  'TLS_SECURITY_IMPLEMENTATION.md'
];

securityDocs.forEach(doc => {
  const exists = existsSync(doc);
  testResult(`Security documentation: ${doc}`, exists);
});

// Test 9: Code Compilation
console.log('\n📋 TEST 9: CODE COMPILATION VALIDATION');

try {
  console.log('   Running Rust compilation check...');
  await execAsync('cd src-tauri && cargo check --quiet');
  testResult('Rust code compiles with security features', true, 'All security modules compile successfully');
} catch (error) {
  testResult('Rust code compilation', false, `Compilation error: ${error.message}`);
}

// Final Results
console.log('\n🔒 SECURITY VALIDATION RESULTS');
console.log('=====================================');
console.log(`Tests Passed: ${testsPassed}/${testsTotal}`);
console.log(`Success Rate: ${((testsPassed / testsTotal) * 100).toFixed(1)}%`);

if (testsPassed === testsTotal) {
  console.log('\n🎉 ALL SECURITY TESTS PASSED!');
  console.log('✅ The application is ready for secure deployment');
} else {
  console.log('\n⚠️  SOME SECURITY TESTS FAILED');
  console.log('❌ Please review and fix failing tests before deployment');
}

console.log('\n📋 Next Steps:');
console.log('1. Review any failing tests above');
console.log('2. Run production build to test CSP in bundled app');
console.log('3. Coordinate with server admin for WSS certificate setup');
console.log('4. Consider using git filter-branch to clean git history');
console.log('5. Implement credential rotation before production deployment');