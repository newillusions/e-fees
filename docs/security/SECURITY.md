# Security Configuration Documentation

## Overview

This document outlines the comprehensive Content Security Policy (CSP) and security enhancements implemented in the Fee Proposal Management application.

## Content Security Policy (CSP)

### Configuration

The application implements a strict CSP configuration in `src-tauri/tauri.conf.json`:

```json
{
  "security": {
    "csp": "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self' data:; connect-src 'self' ws://10.0.1.17:8000 wss://10.0.1.17:8000; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'none'; upgrade-insecure-requests"
  }
}
```

### CSP Directives Explained

| Directive | Value | Purpose |
|-----------|-------|---------|
| `default-src` | `'self'` | Default policy for all resource types from same origin only |
| `script-src` | `'self' 'unsafe-inline' 'unsafe-eval'` | Allow scripts from same origin, inline scripts, and eval (required for Svelte) |
| `style-src` | `'self' 'unsafe-inline'` | Allow styles from same origin and inline styles (required for component styling) |
| `img-src` | `'self' data: blob:` | Allow images from same origin, data URLs, and blob URLs |
| `font-src` | `'self' data:` | Allow fonts from same origin and data URLs |
| `connect-src` | `'self' ws://10.0.1.17:8000 wss://10.0.1.17:8000` | Allow connections to same origin and SurrealDB WebSocket |
| `object-src` | `'none'` | Completely block plugins and objects |
| `base-uri` | `'self'` | Restrict base element to same origin |
| `form-action` | `'self'` | Only allow form submissions to same origin |
| `frame-ancestors` | `'none'` | Prevent embedding in frames (clickjacking protection) |
| `upgrade-insecure-requests` | | Automatically upgrade HTTP requests to HTTPS |

### Security Trade-offs

**Allowing `'unsafe-inline'` and `'unsafe-eval'`:**
- Required for Svelte's dynamic component rendering
- Required for Vite's development hot module replacement
- Mitigated by strict origin controls and comprehensive monitoring

**WebSocket Connections:**
- SurrealDB connection requires explicit WebSocket allowlist
- Limited to specific IP address and ports
- SSL/TLS encryption enforced where possible

## Security Headers

### HTTP Security Headers

Additional security headers are configured in `index.html`:

```html
<!-- Security Headers -->
<meta http-equiv="X-Content-Type-Options" content="nosniff" />
<meta http-equiv="X-Frame-Options" content="DENY" />
<meta http-equiv="X-XSS-Protection" content="1; mode=block" />
<meta http-equiv="Referrer-Policy" content="strict-origin-when-cross-origin" />
<meta http-equiv="Permissions-Policy" content="camera=(), microphone=(), geolocation=(), payment=(), usb=(), bluetooth=(), accelerometer=(), gyroscope=(), magnetometer=(), ambient-light-sensor=()" />
```

| Header | Value | Purpose |
|--------|-------|---------|
| `X-Content-Type-Options` | `nosniff` | Prevent MIME type sniffing attacks |
| `X-Frame-Options` | `DENY` | Prevent clickjacking attacks |
| `X-XSS-Protection` | `1; mode=block` | Enable browser XSS protection |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | Control referrer information |
| `Permissions-Policy` | Various restrictions | Disable unnecessary browser APIs |

## Permission Hardening

### Tauri Capabilities

The application uses principle of least privilege in `src-tauri/capabilities/default.json`:

**Removed permissions:**
- File system access beyond application scope
- System shell access
- Network access beyond WebSocket
- Desktop integration beyond basic window management

**Retained permissions:**
- Basic window management
- Dialog access for file operations
- Event system for component communication
- Logging for security monitoring

### Security Monitoring

### Real-time CSP Violation Monitoring

The application includes comprehensive security monitoring in `src/lib/security.ts`:

#### Features:
- **CSP Violation Detection**: Automatically captures and logs CSP violations
- **Unsafe Practice Detection**: Monitors for eval() usage and inline script injection
- **Security Event Logging**: Maintains history of security events
- **Performance Impact Monitoring**: Tracks security-related performance metrics

#### Usage:

```typescript
import { securityMonitor, getSecurityReport } from './lib/security';

// Get current security status
const report = getSecurityReport();
console.log('Violations:', report.violations);
console.log('Summary:', report.summary);
console.log('Validation:', report.validation);
```

#### Development Tools:

In development mode, security reporting is available globally:
```javascript
// Available in browser console
window.getSecurityReport();
```

## Backend Security

### Rust Security Module

The backend includes security utilities in `src-tauri/src/security.rs`:

#### Features:
- **CSP Header Validation**: Programmatic CSP policy validation
- **Input Sanitization**: XSS prevention utilities
- **Secure Token Generation**: Cryptographically secure random tokens
- **Password Hashing**: Argon2 implementation for future authentication
- **Rate Limiting**: Configurable rate limiting for API endpoints

#### Example Usage:

```rust
use crate::security::{sanitize_input, validate_csp_header, generate_secure_token};

// Sanitize user input
let safe_input = sanitize_input(user_input);

// Validate CSP configuration
let warnings = validate_csp_header(csp_string)?;

// Generate secure tokens
let token = generate_secure_token(32);
```

## Testing and Validation

### Automated Security Testing

Run the comprehensive security test script:

```bash
# In browser console
node test-csp-security.js
```

### Manual Testing Checklist

- [ ] CSP violations are logged and blocked
- [ ] Inline scripts are prevented (when not required)
- [ ] External resource loading is controlled
- [ ] WebSocket connections are limited to allowed origins
- [ ] Security headers are present and correct
- [ ] No mixed content warnings in HTTPS context
- [ ] Rate limiting functions correctly
- [ ] Input sanitization prevents XSS

### Security Validation

The security monitor includes automated validation:

```typescript
const validation = securityMonitor.validateSecurityConfiguration();
console.log('Passed:', validation.passed);
console.log('Warnings:', validation.warnings);
console.log('Errors:', validation.errors);
```

## Security Best Practices

### Development Guidelines

1. **Never disable CSP**: Always maintain CSP protection
2. **Validate all inputs**: Use sanitization utilities for user input
3. **Monitor violations**: Check security reports regularly
4. **Update dependencies**: Keep security-related packages current
5. **Test thoroughly**: Run security tests before deployment

### Content Guidelines

1. **Avoid inline handlers**: Use event listeners instead of onclick attributes
2. **Validate external content**: Sanitize any user-generated content
3. **Use HTTPS**: Ensure all external connections use SSL/TLS
4. **Minimize eval()**: Avoid eval() where possible, monitor when necessary

### Deployment Security

1. **Environment Variables**: Store sensitive configuration in environment variables
2. **HTTPS Only**: Deploy only over HTTPS in production
3. **Security Headers**: Ensure all security headers are present
4. **Regular Audits**: Perform regular security audits and penetration testing

## Threat Model

### Mitigated Threats

- **Cross-Site Scripting (XSS)**: CSP + input sanitization
- **Clickjacking**: X-Frame-Options header
- **MIME Sniffing**: X-Content-Type-Options header
- **Code Injection**: Strict CSP and input validation
- **Unauthorized API Access**: Principle of least privilege permissions

### Remaining Risks

- **Social Engineering**: User education required
- **Physical Access**: Application-level security cannot prevent physical compromise
- **Network Attacks**: External network security required
- **Dependency Vulnerabilities**: Regular updates and auditing required

## Incident Response

### CSP Violation Response

1. **Immediate**: Check security monitor for violation details
2. **Investigate**: Determine if violation is legitimate or malicious
3. **Update Policy**: Adjust CSP if legitimate content is blocked
4. **Report**: Log security incidents for pattern analysis

### Security Event Escalation

1. **Critical Events**: Immediate investigation required
2. **Pattern Detection**: Multiple violations may indicate attack
3. **Response Plan**: Have incident response procedures ready
4. **Recovery**: Procedures for security breach recovery

## Configuration Management

### Environment-Specific Settings

```javascript
// Development
const CSP_DEV = "default-src 'self' 'unsafe-inline' 'unsafe-eval'";

// Production  
const CSP_PROD = "default-src 'self'; script-src 'self'";
```

### Security Configuration Versioning

- Track CSP policy changes in version control
- Document security configuration changes
- Test security changes in staging environment
- Maintain rollback procedures for security configurations

## Compliance and Standards

### Security Standards Compliance

- **OWASP Top 10**: Protection against common web vulnerabilities
- **Content Security Policy Level 3**: Modern CSP implementation
- **Secure Headers**: Implementation of security-focused HTTP headers
- **Principle of Least Privilege**: Minimal permission requirements

### Regular Security Reviews

- Monthly CSP violation review
- Quarterly security configuration audit
- Annual penetration testing
- Continuous dependency vulnerability scanning

---

## Quick Reference

### Security Commands

```bash
# Run security tests
node test-csp-security.js

# Check CSP violations (browser console)
window.getSecurityReport()

# Validate security configuration
npm run security:validate
```

### Emergency CSP Disable

**ONLY in extreme emergency - removes all CSP protection:**

```json
{
  "security": {
    "csp": null
  }
}
```

### Contact

For security issues or questions:
- Review this documentation
- Check security monitor logs
- Test with provided security scripts
- Follow incident response procedures

---

**Last Updated**: August 17, 2025  
**Security Configuration Version**: 1.0  
**Next Review**: September 17, 2025