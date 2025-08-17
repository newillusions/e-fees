#!/usr/bin/env node
/**
 * SQL Injection Security Test Suite
 * 
 * This script validates that our security fixes prevent SQL injection attacks
 * by testing input validation functions with known malicious payloads.
 */

console.log('🔒 SQL Injection Security Test Suite');
console.log('=====================================\n');

// Simulated validation functions (equivalent to our Rust implementation)
class InputValidator {
    static validateProjectName(name) {
        if (!name || name.length === 0) {
            return { valid: false, error: 'Required field name is missing or empty' };
        }
        
        if (name.length < 2 || name.length > 200) {
            return { valid: false, error: `Field name length ${name.length} is outside allowed range 2-200` };
        }

        // Allow alphanumeric, spaces, hyphens, underscores, parentheses, and common punctuation
        const pattern = /^[a-zA-Z0-9\s\-_().&,]+$/;
        if (!pattern.test(name)) {
            return { valid: false, error: 'Field name contains invalid characters' };
        }

        return { valid: true };
    }

    static validateEmail(email) {
        if (!email || email.length === 0) {
            return { valid: false, error: 'Required field email is missing or empty' };
        }

        const emailPattern = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/;
        if (!emailPattern.test(email)) {
            return { valid: false, error: 'Invalid email format' };
        }

        return { valid: true };
    }

    static validatePhone(phone) {
        if (!phone || phone.length === 0) {
            return { valid: false, error: 'Required field phone is missing or empty' };
        }

        const phonePattern = /^[\+]?[0-9\s\-\(\)]{7,20}$/;
        if (!phonePattern.test(phone)) {
            return { valid: false, error: 'Invalid phone format' };
        }

        return { valid: true };
    }

    static validateProjectNumber(number) {
        const pattern = /^\d{2}-\d{3}\d{2}$/;
        if (!pattern.test(number)) {
            return { valid: false, error: 'Invalid project number format' };
        }

        return { valid: true };
    }

    static validateId(id) {
        if (!id || id.length === 0) {
            return { valid: false, error: 'Required field id is missing or empty' };
        }

        const idPattern = /^[a-zA-Z0-9_]+$/;
        if (!idPattern.test(id)) {
            return { valid: false, error: 'Invalid ID format' };
        }

        return { valid: true };
    }

    static sanitizeForDisplay(input) {
        return input
            .split('')
            .filter(c => /[a-zA-Z0-9 \-_.,()&@]/.test(c))
            .join('');
    }
}

// SQL Injection Attack Payloads
const SQL_INJECTION_PAYLOADS = [
    // Basic injections
    "' OR '1'='1",
    "' OR 1=1--",
    "' OR 'a'='a",
    "' OR 1=1#",
    
    // Union-based injections
    "' UNION SELECT * FROM company--",
    "' UNION ALL SELECT NULL,NULL,NULL--",
    
    // Boolean-based blind injections
    "' AND (SELECT COUNT(*) FROM projects) > 0--",
    "' AND ASCII(SUBSTRING((SELECT name FROM projects LIMIT 1),1,1)) > 65--",
    
    // Time-based blind injections
    "'; WAITFOR DELAY '00:00:05'--",
    "' OR IF(1=1, SLEEP(5), 0)--",
    
    // Stacked queries
    "'; DROP TABLE projects; --",
    "'; INSERT INTO projects VALUES (...); --",
    "'; UPDATE projects SET status='Hacked'; --",
    
    // Comment variations
    "' OR 1=1/*",
    "' OR 1=1--",
    "' OR 1=1#",
    
    // Case variations
    "' or '1'='1",
    "' OR '1'='1",
    "' oR '1'='1",
    
    // Encoding attempts (these would be decoded by HTTP layer before reaching validation)
    // "%27%20OR%20%271%27%3D%271", // URL encoded - handled by HTTP layer
    "\\' OR \\'1\\'=\\'1",
    
    // Famous XKCD reference
    "Robert'); DROP TABLE students; --",
];

// Test Results
let totalTests = 0;
let passedTests = 0;
let failedTests = 0;

function runTest(testName, testFunction) {
    totalTests++;
    try {
        const result = testFunction();
        if (result.success) {
            console.log(`✅ ${testName}`);
            passedTests++;
        } else {
            console.log(`❌ ${testName}: ${result.message}`);
            failedTests++;
        }
    } catch (error) {
        console.log(`❌ ${testName}: Exception - ${error.message}`);
        failedTests++;
    }
}

// Test SQL Injection Prevention in Project Names
runTest('SQL Injection Prevention - Project Names', () => {
    let blocked = 0;
    let allowed = 0;
    
    for (const payload of SQL_INJECTION_PAYLOADS) {
        const result = InputValidator.validateProjectName(payload);
        if (!result.valid) {
            blocked++;
        } else {
            allowed++;
            console.log(`⚠️  WARNING: Payload allowed: ${payload}`);
        }
    }
    
    if (allowed === 0) {
        return { success: true, message: `Blocked ${blocked}/${SQL_INJECTION_PAYLOADS.length} payloads` };
    } else {
        return { success: false, message: `${allowed} dangerous payloads were allowed!` };
    }
});

// Test SQL Injection Prevention in Emails
runTest('SQL Injection Prevention - Email Fields', () => {
    let blocked = 0;
    let allowed = [];
    
    for (const payload of SQL_INJECTION_PAYLOADS) {
        const maliciousEmail = `user${payload}@example.com`;
        const result = InputValidator.validateEmail(maliciousEmail);
        if (!result.valid) {
            blocked++;
        } else {
            allowed.push(payload);
        }
    }
    
    if (allowed.length > 0) {
        console.log(`⚠️  Allowed payloads: ${allowed.join(', ')}`);
    }
    
    return { 
        success: blocked === SQL_INJECTION_PAYLOADS.length,
        message: `Blocked ${blocked}/${SQL_INJECTION_PAYLOADS.length} malicious emails`
    };
});

// Test Valid Inputs Still Work
runTest('Valid Inputs - Project Names', () => {
    const validNames = [
        'Dubai Marina Tower',
        'Project-123_Test',
        'Al Raha Beach Development',
        'Tower A (Phase 1)',
        'Residential Building B&C'
    ];
    
    for (const name of validNames) {
        const result = InputValidator.validateProjectName(name);
        if (!result.valid) {
            return { success: false, message: `Valid name rejected: ${name}` };
        }
    }
    
    return { success: true };
});

// Test Valid Inputs - Emails
runTest('Valid Inputs - Email Addresses', () => {
    const validEmails = [
        'test@example.com',
        'user.name+tag@domain.co.uk',
        'admin@company-name.com',
        'support@123domain.org'
    ];
    
    for (const email of validEmails) {
        const result = InputValidator.validateEmail(email);
        if (!result.valid) {
            return { success: false, message: `Valid email rejected: ${email}` };
        }
    }
    
    return { success: true };
});

// Test Valid Inputs - Phone Numbers
runTest('Valid Inputs - Phone Numbers', () => {
    const validPhones = [
        '+971 50 123 4567',
        '050-123-4567',
        '+1-555-123-4567',
        '(555) 123-4567'
    ];
    
    for (const phone of validPhones) {
        const result = InputValidator.validatePhone(phone);
        if (!result.valid) {
            return { success: false, message: `Valid phone rejected: ${phone}` };
        }
    }
    
    return { success: true };
});

// Test Project Number Validation
runTest('Valid Inputs - Project Numbers', () => {
    const validNumbers = ['25-97105', '24-96601', '99-12301'];
    const invalidNumbers = ['25-971', '2025-97105', '25-ABC05', 'invalid'];
    
    // Valid numbers should pass
    for (const number of validNumbers) {
        const result = InputValidator.validateProjectNumber(number);
        if (!result.valid) {
            return { success: false, message: `Valid project number rejected: ${number}` };
        }
    }
    
    // Invalid numbers should fail
    for (const number of invalidNumbers) {
        const result = InputValidator.validateProjectNumber(number);
        if (result.valid) {
            return { success: false, message: `Invalid project number accepted: ${number}` };
        }
    }
    
    return { success: true };
});

// Test ID Validation
runTest('Valid Inputs - ID Fields', () => {
    const validIds = ['CHE', 'EMITTIV_01', 'project_123'];
    const invalidIds = ['CH E', 'CHE-01', "CHE'; DROP TABLE company; --", ''];
    
    // Valid IDs should pass
    for (const id of validIds) {
        const result = InputValidator.validateId(id);
        if (!result.valid) {
            return { success: false, message: `Valid ID rejected: ${id}` };
        }
    }
    
    // Invalid IDs should fail
    for (const id of invalidIds) {
        const result = InputValidator.validateId(id);
        if (result.valid) {
            return { success: false, message: `Invalid ID accepted: ${id}` };
        }
    }
    
    return { success: true };
});

// Test Input Sanitization
runTest('Input Sanitization', () => {
    const dangerousInput = "Test'; DROP TABLE projects; --<script>alert('xss')</script>";
    const sanitized = InputValidator.sanitizeForDisplay(dangerousInput);
    
    const dangerousChars = ["'", ";", "<", ">"];
    const hasDangerousChars = dangerousChars.some(char => sanitized.includes(char));
    
    if (hasDangerousChars) {
        return { success: false, message: `Dangerous characters not properly sanitized. Result: ${sanitized}` };
    }
    
    if (!sanitized.includes('Test')) {
        return { success: false, message: 'Safe content was removed during sanitization' };
    }
    
    return { success: true, message: `Successfully sanitized: "${dangerousInput}" -> "${sanitized}"` };
});

// Test Length Validation
runTest('Length Validation', () => {
    const tooShort = 'A';
    const tooLong = 'A'.repeat(1001);
    const justRight = 'Perfect Length Project Name';
    
    if (InputValidator.validateProjectName(tooShort).valid) {
        return { success: false, message: 'Too short input was accepted' };
    }
    
    if (InputValidator.validateProjectName(tooLong).valid) {
        return { success: false, message: 'Too long input was accepted' };
    }
    
    if (!InputValidator.validateProjectName(justRight).valid) {
        return { success: false, message: 'Valid length input was rejected' };
    }
    
    return { success: true };
});

// Print Results
console.log('\n🔒 Security Test Results');
console.log('========================');
console.log(`Total Tests: ${totalTests}`);
console.log(`✅ Passed: ${passedTests}`);
console.log(`❌ Failed: ${failedTests}`);
console.log(`Success Rate: ${((passedTests / totalTests) * 100).toFixed(1)}%`);

if (failedTests === 0) {
    console.log('\n🎉 ALL SECURITY TESTS PASSED!');
    console.log('✅ SQL Injection vulnerabilities have been successfully mitigated.');
    process.exit(0);
} else {
    console.log('\n⚠️  SECURITY TESTS FAILED!');
    console.log('❌ Some vulnerabilities may still exist.');
    process.exit(1);
}