#!/usr/bin/env node

// Simple test script for SurrealDB utility functions
// Run with: node test_surreal_utils.js

// Inline implementations of the utility functions to test
function extractSurrealId(input) {
    if (!input) return null;
    
    // Handle string format
    if (typeof input === 'string') {
        // Already in correct format
        if (input.includes(':')) {
            return input;
        }
        // Just the ID part
        return input;
    }
    
    // Handle object format (Thing object from SurrealDB)
    if (typeof input === 'object') {
        // Format: { tb: 'table', id: 'identifier' }
        if (input.tb && input.id) {
            return `${input.tb}:${input.id}`;
        }
        // Format: { id: 'table:identifier' }
        if (input.id && typeof input.id === 'string') {
            return input.id;
        }
    }
    
    return null;
}

function compareSurrealIds(id1, id2) {
    const extracted1 = extractSurrealId(id1);
    const extracted2 = extractSurrealId(id2);
    
    if (!extracted1 || !extracted2) return false;
    
    return extracted1 === extracted2;
}

function validateSurrealData(data, requiredFields) {
    const errors = [];
    
    for (const field of requiredFields) {
        if (!data[field] || (typeof data[field] === 'string' && data[field].trim() === '')) {
            errors.push(`Missing required field: ${field}`);
        }
    }
    
    return {
        isValid: errors.length === 0,
        errors
    };
}

// Test cases
console.log('🧪 Testing SurrealDB Utility Functions\n');

// Test extractSurrealId
console.log('1️⃣  Testing extractSurrealId:');
console.log('=====================================');

const testCases = [
    { input: 'company:abc123', expected: 'company:abc123', description: 'String format with table:id' },
    { input: 'abc123', expected: 'abc123', description: 'String format with just id' },
    { input: { tb: 'company', id: 'abc123' }, expected: 'company:abc123', description: 'Object with tb and id' },
    { input: { id: 'company:abc123' }, expected: 'company:abc123', description: 'Object with full id' },
    { input: null, expected: null, description: 'Null input' },
    { input: undefined, expected: null, description: 'Undefined input' },
    { input: {}, expected: null, description: 'Empty object' },
    { input: { wrong: 'format' }, expected: null, description: 'Invalid object format' }
];

let passed = 0;
let failed = 0;

testCases.forEach((test, index) => {
    const result = extractSurrealId(test.input);
    const success = result === test.expected;
    
    if (success) {
        console.log(`✅ Test ${index + 1}: ${test.description}`);
        console.log(`   Input: ${JSON.stringify(test.input)}`);
        console.log(`   Output: ${result}`);
        passed++;
    } else {
        console.log(`❌ Test ${index + 1}: ${test.description}`);
        console.log(`   Input: ${JSON.stringify(test.input)}`);
        console.log(`   Expected: ${test.expected}`);
        console.log(`   Got: ${result}`);
        failed++;
    }
    console.log();
});

// Test compareSurrealIds
console.log('\n2️⃣  Testing compareSurrealIds:');
console.log('=====================================');

const compareTests = [
    { id1: 'company:abc123', id2: 'company:abc123', expected: true, description: 'Same string IDs' },
    { id1: 'company:abc123', id2: { tb: 'company', id: 'abc123' }, expected: true, description: 'String vs object (same)' },
    { id1: { tb: 'company', id: 'abc123' }, id2: { id: 'company:abc123' }, expected: true, description: 'Different object formats (same)' },
    { id1: 'company:abc123', id2: 'company:xyz789', expected: false, description: 'Different IDs' },
    { id1: 'company:abc123', id2: 'contact:abc123', expected: false, description: 'Different tables' },
    { id1: null, id2: 'company:abc123', expected: false, description: 'Null vs valid' },
    { id1: 'company:abc123', id2: null, expected: false, description: 'Valid vs null' }
];

compareTests.forEach((test, index) => {
    const result = compareSurrealIds(test.id1, test.id2);
    const success = result === test.expected;
    
    if (success) {
        console.log(`✅ Test ${index + 1}: ${test.description}`);
        console.log(`   ID1: ${JSON.stringify(test.id1)}`);
        console.log(`   ID2: ${JSON.stringify(test.id2)}`);
        console.log(`   Result: ${result}`);
        passed++;
    } else {
        console.log(`❌ Test ${index + 1}: ${test.description}`);
        console.log(`   ID1: ${JSON.stringify(test.id1)}`);
        console.log(`   ID2: ${JSON.stringify(test.id2)}`);
        console.log(`   Expected: ${test.expected}`);
        console.log(`   Got: ${result}`);
        failed++;
    }
    console.log();
});

// Test validateSurrealData
console.log('\n3️⃣  Testing validateSurrealData:');
console.log('=====================================');

const validationTests = [
    {
        data: { name: 'John Doe', email: 'john@example.com', company: 'company:abc123' },
        requiredFields: ['name', 'email', 'company'],
        expectedValid: true,
        description: 'All required fields present'
    },
    {
        data: { name: 'John Doe', email: '' },
        requiredFields: ['name', 'email'],
        expectedValid: false,
        description: 'Empty string field'
    },
    {
        data: { name: 'John Doe' },
        requiredFields: ['name', 'email', 'company'],
        expectedValid: false,
        description: 'Missing required fields'
    },
    {
        data: { name: '   ', email: 'john@example.com' },
        requiredFields: ['name', 'email'],
        expectedValid: false,
        description: 'Whitespace-only field'
    }
];

validationTests.forEach((test, index) => {
    const result = validateSurrealData(test.data, test.requiredFields);
    const success = result.isValid === test.expectedValid;
    
    if (success) {
        console.log(`✅ Test ${index + 1}: ${test.description}`);
        console.log(`   Data: ${JSON.stringify(test.data)}`);
        console.log(`   Required: ${JSON.stringify(test.requiredFields)}`);
        console.log(`   Valid: ${result.isValid}`);
        if (result.errors.length > 0) {
            console.log(`   Errors: ${result.errors.join(', ')}`);
        }
        passed++;
    } else {
        console.log(`❌ Test ${index + 1}: ${test.description}`);
        console.log(`   Data: ${JSON.stringify(test.data)}`);
        console.log(`   Required: ${JSON.stringify(test.requiredFields)}`);
        console.log(`   Expected Valid: ${test.expectedValid}`);
        console.log(`   Got Valid: ${result.isValid}`);
        console.log(`   Errors: ${result.errors.join(', ')}`);
        failed++;
    }
    console.log();
});

// Summary
console.log('\n📊 Test Summary:');
console.log('=====================================');
console.log(`✅ Passed: ${passed}`);
console.log(`❌ Failed: ${failed}`);
console.log(`📈 Total: ${passed + failed}`);
console.log(`🎯 Success Rate: ${((passed / (passed + failed)) * 100).toFixed(1)}%`);

// Exit with appropriate code
process.exit(failed > 0 ? 1 : 0);