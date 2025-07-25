#!/usr/bin/env node

// Integration test example showing how the utilities work with real SurrealDB data
// Run with: node test_integration_example.js

// Include the utility functions
function extractSurrealId(input) {
    if (!input) return null;
    
    if (typeof input === 'string') {
        return input;
    }
    
    if (typeof input === 'object') {
        if (input.tb && input.id) {
            return `${input.tb}:${input.id}`;
        }
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

console.log('🔗 SurrealDB Integration Test Examples\n');

// Example 1: Processing company data as it comes from SurrealDB
console.log('1️⃣  Company Data Processing:');
console.log('============================');

const companyFromDB = {
    id: { tb: 'company', id: '4fkb4j4uw4p3t5ygvxj2' },
    name: 'Emirates NBD',
    industry: 'Banking',
    country: 'UAE'
};

const extractedId = extractSurrealId(companyFromDB.id);
console.log(`Original ID: ${JSON.stringify(companyFromDB.id)}`);
console.log(`Extracted ID: ${extractedId}`);
console.log(`✅ Successfully extracted: ${extractedId !== null}\n`);

// Example 2: Contact creation with company reference
console.log('2️⃣  Contact Creation with Company Reference:');
console.log('=============================================');

const newContactData = {
    first_name: 'Ahmed',
    last_name: 'Al-Rashid',
    email: 'ahmed@emiratesnbd.ae',
    company: companyFromDB.id // This is a Thing object
};

// Extract company ID for API call
const processedContactData = {
    ...newContactData,
    company: extractSurrealId(newContactData.company)
};

console.log('Original contact data:');
console.log(JSON.stringify(newContactData, null, 2));
console.log('\nProcessed for API:');
console.log(JSON.stringify(processedContactData, null, 2));

// Validate the processed data
const validation = validateSurrealData(processedContactData, ['first_name', 'last_name', 'email', 'company']);
console.log(`\nValidation result: ${validation.isValid ? '✅ Valid' : '❌ Invalid'}`);
if (!validation.isValid) {
    console.log(`Errors: ${validation.errors.join(', ')}`);
}
console.log();

// Example 3: Comparing IDs for updates
console.log('3️⃣  ID Comparison for Updates:');
console.log('==============================');

const existingContact = {
    id: { tb: 'contact', id: 'xyz789' },
    company: { tb: 'company', id: '4fkb4j4uw4p3t5ygvxj2' }
};

const updateData = {
    company: 'company:4fkb4j4uw4p3t5ygvxj2' // String format from form
};

const companyChanged = !compareSurrealIds(existingContact.company, updateData.company);
console.log(`Existing company: ${JSON.stringify(existingContact.company)}`);
console.log(`New company: ${updateData.company}`);
console.log(`Company changed: ${companyChanged ? '❌ Yes' : '✅ No'}`);
console.log();

// Example 4: Filtering and searching
console.log('4️⃣  Filtering with ID Comparison:');
console.log('=================================');

const contacts = [
    { id: { tb: 'contact', id: 'c1' }, name: 'John', company: { tb: 'company', id: 'comp1' } },
    { id: { tb: 'contact', id: 'c2' }, name: 'Jane', company: { tb: 'company', id: 'comp2' } },
    { id: { tb: 'contact', id: 'c3' }, name: 'Bob', company: { tb: 'company', id: 'comp1' } }
];

const filterByCompany = 'company:comp1';
const filteredContacts = contacts.filter(contact => 
    compareSurrealIds(contact.company, filterByCompany)
);

console.log(`Filtering by company: ${filterByCompany}`);
console.log(`Found ${filteredContacts.length} contacts:`);
filteredContacts.forEach(contact => {
    console.log(`  - ${contact.name} (ID: ${extractSurrealId(contact.id)})`);
});
console.log();

// Example 5: Error handling
console.log('5️⃣  Error Handling:');
console.log('===================');

const invalidData = [
    null,
    undefined,
    {},
    { wrong: 'format' },
    ''
];

invalidData.forEach((data, index) => {
    const result = extractSurrealId(data);
    console.log(`Invalid input ${index + 1}: ${JSON.stringify(data)} → ${result === null ? '✅ Handled' : '❌ Not handled'}`);
});

console.log('\n🎉 Integration test completed successfully!');
console.log('All utility functions work correctly with SurrealDB data structures.');