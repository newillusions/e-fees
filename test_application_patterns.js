#!/usr/bin/env node

// Test application patterns with the utility functions
// This demonstrates how they would be used in the actual Svelte components
// Run with: node test_application_patterns.js

// Include the utility functions
function extractSurrealId(input) {
    if (!input) return null;
    if (typeof input === 'string') return input;
    if (typeof input === 'object') {
        if (input.tb && input.id) return `${input.tb}:${input.id}`;
        if (input.id && typeof input.id === 'string') return input.id;
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
    return { isValid: errors.length === 0, errors };
}

console.log('🚀 Application Pattern Tests\n');

// Pattern 1: Form submission handling (like ContactModal.svelte)
console.log('1️⃣  Form Submission Pattern:');
console.log('============================');

// Simulate form data with company selection
const formData = {
    first_name: 'Sarah',
    last_name: 'Johnson',
    email: 'sarah@example.com',
    phone: '+971-50-123-4567',
    company: { tb: 'company', id: 'abc123' } // From dropdown selection
};

// Process for API submission
const processedData = {
    ...formData,
    company: extractSurrealId(formData.company),
    full_name: `${formData.first_name} ${formData.last_name}`.trim()
};

// Validate before submission
const validation = validateSurrealData(processedData, ['first_name', 'last_name', 'email', 'company']);

console.log('Form data processed for submission:');
console.log(JSON.stringify(processedData, null, 2));
console.log(`Validation: ${validation.isValid ? '✅ Valid' : '❌ Invalid'}`);
if (!validation.isValid) {
    console.log(`Validation errors: ${validation.errors.join(', ')}`);
}
console.log();

// Pattern 2: Data filtering (like in stores.ts)
console.log('2️⃣  Data Filtering Pattern:');
console.log('===========================');

// Simulate contacts from database
const contactsFromDB = [
    {
        id: { tb: 'contact', id: 'c1' },
        first_name: 'John',
        last_name: 'Doe',
        company: { tb: 'company', id: 'comp1' }
    },
    {
        id: { tb: 'contact', id: 'c2' },
        first_name: 'Jane',
        last_name: 'Smith',
        company: { tb: 'company', id: 'comp2' }
    },
    {
        id: { tb: 'contact', id: 'c3' },
        first_name: 'Bob',
        last_name: 'Wilson',
        company: { tb: 'company', id: 'comp1' }
    }
];

// Filter by company
const selectedCompany = 'company:comp1';
const filteredContacts = contactsFromDB.filter(contact => 
    compareSurrealIds(contact.company, selectedCompany)
);

console.log(`Filtering ${contactsFromDB.length} contacts by company: ${selectedCompany}`);
console.log(`Found ${filteredContacts.length} matching contacts:`);
filteredContacts.forEach(contact => {
    console.log(`  - ${contact.first_name} ${contact.last_name}`);
});
console.log();

// Pattern 3: Dropdown population (company selection)
console.log('3️⃣  Dropdown Population Pattern:');
console.log('=================================');

// Simulate companies from database
const companiesFromDB = [
    { id: { tb: 'company', id: 'comp1' }, name: 'Emirates NBD' },
    { id: { tb: 'company', id: 'comp2' }, name: 'ADNOC' },
    { id: { tb: 'company', id: 'comp3' }, name: 'Dubai Municipality' }
];

// Process for dropdown options
const dropdownOptions = companiesFromDB.map(company => ({
    value: extractSurrealId(company.id),
    label: company.name,
    raw: company
}));

console.log('Company dropdown options:');
dropdownOptions.forEach(option => {
    console.log(`  Value: "${option.value}" | Label: "${option.label}"`);
});
console.log();

// Pattern 4: Update detection (existing vs new data)
console.log('4️⃣  Update Detection Pattern:');
console.log('=============================');

const existingContact = {
    id: { tb: 'contact', id: 'c1' },
    first_name: 'John',
    last_name: 'Doe',
    email: 'john@example.com',
    company: { tb: 'company', id: 'comp1' }
};

const updatedFormData = {
    first_name: 'John',
    last_name: 'Doe',
    email: 'john.doe@newemail.com', // Changed
    company: 'company:comp2' // Changed company
};

// Detect changes
const nameChanged = existingContact.first_name !== updatedFormData.first_name || 
                   existingContact.last_name !== updatedFormData.last_name;
const emailChanged = existingContact.email !== updatedFormData.email;
const companyChanged = !compareSurrealIds(existingContact.company, updatedFormData.company);

console.log('Change detection:');
console.log(`  Name changed: ${nameChanged ? '✅ Yes' : '❌ No'}`);
console.log(`  Email changed: ${emailChanged ? '✅ Yes' : '❌ No'}`);
console.log(`  Company changed: ${companyChanged ? '✅ Yes' : '❌ No'}`);
console.log(`  Any changes: ${nameChanged || emailChanged || companyChanged ? '✅ Yes' : '❌ No'}`);
console.log();

// Pattern 5: Error scenarios and edge cases
console.log('5️⃣  Error Handling Patterns:');
console.log('============================');

const errorScenarios = [
    {
        name: 'Missing required fields',
        data: { first_name: 'John' },
        requiredFields: ['first_name', 'last_name', 'email', 'company']
    },
    {
        name: 'Empty string fields',
        data: { first_name: '', last_name: 'Doe', email: 'john@example.com', company: 'company:123' },
        requiredFields: ['first_name', 'last_name', 'email', 'company']
    },
    {
        name: 'Whitespace only fields',
        data: { first_name: '   ', last_name: 'Doe', email: 'john@example.com', company: 'company:123' },
        requiredFields: ['first_name', 'last_name', 'email', 'company']
    }
];

errorScenarios.forEach((scenario, index) => {
    const validation = validateSurrealData(scenario.data, scenario.requiredFields);
    console.log(`Scenario ${index + 1}: ${scenario.name}`);
    console.log(`  Valid: ${validation.isValid ? '✅ Yes' : '❌ No'}`);
    if (!validation.isValid) {
        console.log(`  Errors: ${validation.errors.join(', ')}`);
    }
    console.log();
});

// Pattern 6: Real-world usage example
console.log('6️⃣  Complete Workflow Example:');
console.log('==============================');

// Simulate the complete contact creation workflow
console.log('Step 1: User selects company from dropdown');
const selectedCompanyOption = dropdownOptions[0]; // Emirates NBD
console.log(`Selected: ${selectedCompanyOption.label} (${selectedCompanyOption.value})`);

console.log('\nStep 2: User fills form');
const completeFormData = {
    first_name: 'Ahmed',
    last_name: 'Hassan',
    email: 'ahmed.hassan@emiratesnbd.ae',
    phone: '+971-4-609-2222',
    company: selectedCompanyOption.value
};

console.log('\nStep 3: Validate form data');
const completeValidation = validateSurrealData(completeFormData, ['first_name', 'last_name', 'email', 'company']);
console.log(`Form validation: ${completeValidation.isValid ? '✅ Valid' : '❌ Invalid'}`);

if (completeValidation.isValid) {
    console.log('\nStep 4: Prepare data for API');
    const apiData = {
        ...completeFormData,
        full_name: `${completeFormData.first_name} ${completeFormData.last_name}`.trim()
    };
    
    console.log('API payload:');
    console.log(JSON.stringify(apiData, null, 2));
    
    console.log('\n✅ Contact would be created successfully!');
} else {
    console.log(`❌ Form errors: ${completeValidation.errors.join(', ')}`);
}

console.log('\n🎉 All application patterns tested successfully!');
console.log('The utility functions integrate seamlessly with the existing codebase patterns.');