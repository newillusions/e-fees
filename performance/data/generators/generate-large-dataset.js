#!/usr/bin/env node
/**
 * Large Dataset Generator for E-Fees Performance Testing
 * 
 * Generates realistic test data at various scales for performance testing.
 * Uses production data patterns and distributions for accurate testing.
 */

import { faker } from '@faker-js/faker';
import fs from 'fs/promises';
import path from 'path';
import { createWriteStream } from 'fs';
import { Surreal } from 'surrealdb.js';

// Import configurations
import { databaseConfig } from '../config/database.config.js';

class DatasetGenerator {
  constructor() {
    this.db = null;
    this.config = databaseConfig;
    this.outputDir = path.join(process.cwd(), 'data', 'generated');
    
    // Production data patterns (from DATABASE_SCHEMA.md analysis)
    this.patterns = {
      projectStatus: {
        'Draft': 0.05,
        'RFP': 0.15, 
        'Active': 0.40,
        'Awarded': 0.05,
        'Completed': 0.25,
        'Lost': 0.05,
        'Cancelled': 0.05
      },
      
      feeStatus: {
        'Draft': 0.30,
        'Sent': 0.25,
        'Negotiation': 0.15,
        'Awarded': 0.10,
        'Completed': 0.05,
        'Lost': 0.10,
        'Cancelled': 0.03,
        'On Hold': 0.02
      },

      countries: {
        'U.A.E.': { weight: 0.60, dialCode: 971 },
        'Saudi Arabia': { weight: 0.30, dialCode: 966 },
        'Qatar': { weight: 0.05, dialCode: 974 },
        'Kuwait': { weight: 0.03, dialCode: 965 },
        'Bahrain': { weight: 0.02, dialCode: 973 }
      },

      projectActivities: [
        'Design and Consultancy',
        'Management', 
        'Construction',
        'Maintenance',
        'Research'
      ],

      stages: {
        'Concept': 0.20,
        'Design Development': 0.30,
        'Documentation': 0.25,
        'Tender': 0.15,
        'Construction': 0.08,
        'Handover': 0.02
      }
    };

    // Reference data (static)
    this.referenceData = {
      countries: [], // Will be populated from existing data
      currencies: [], // Will be populated from existing data
    };
  }

  async initialize() {
    console.log('🚀 Initializing dataset generator...');
    
    // Ensure output directory exists
    await fs.mkdir(this.outputDir, { recursive: true });
    
    // Connect to database for reference data
    try {
      this.db = new Surreal();
      const dbConfig = this.config.connection.test;
      await this.db.connect(dbConfig.url);
      await this.db.use({ ns: dbConfig.namespace, db: dbConfig.database });
      await this.db.signin(dbConfig.auth);
      
      console.log('✅ Connected to database');
      
      // Load reference data
      await this.loadReferenceData();
      
    } catch (error) {
      console.warn('⚠️  Database connection failed, using fallback data:', error.message);
      this.initializeFallbackReferenceData();
    }
  }

  async loadReferenceData() {
    try {
      // Load countries
      const countries = await this.db.query('SELECT * FROM country');
      this.referenceData.countries = countries[0]?.result || [];
      
      // Load currencies  
      const currencies = await this.db.query('SELECT * FROM currency');
      this.referenceData.currencies = currencies[0]?.result || [];
      
      console.log(`📊 Loaded ${this.referenceData.countries.length} countries, ${this.referenceData.currencies.length} currencies`);
      
    } catch (error) {
      console.warn('⚠️  Failed to load reference data:', error.message);
      this.initializeFallbackReferenceData();
    }
  }

  initializeFallbackReferenceData() {
    // Minimal reference data for testing
    this.referenceData.countries = [
      { id: 'country:uae', name: 'U.A.E.', dial_code: 971, code: 'AE' },
      { id: 'country:saudi', name: 'Saudi Arabia', dial_code: 966, code: 'SA' },
      { id: 'country:qatar', name: 'Qatar', dial_code: 974, code: 'QA' },
    ];
    
    this.referenceData.currencies = [
      { id: 'currency:aed', code: 'AED', name: 'UAE Dirham' },
      { id: 'currency:sar', code: 'SAR', name: 'Saudi Riyal' },
      { id: 'currency:usd', code: 'USD', name: 'US Dollar' },
    ];
  }

  // Generate weighted random selection
  selectWeighted(distribution) {
    const random = Math.random();
    let cumulative = 0;
    
    for (const [value, weight] of Object.entries(distribution)) {
      cumulative += weight;
      if (random < cumulative) {
        return value;
      }
    }
    
    // Fallback to first option
    return Object.keys(distribution)[0];
  }

  // Generate project number in YY-CCCNN format
  generateProjectNumber(year, country, sequence) {
    const countryInfo = this.referenceData.countries.find(c => c.name === country);
    const dialCode = countryInfo?.dial_code || 971;
    const formattedSeq = sequence.toString().padStart(2, '0');
    
    return {
      year: year,
      country: dialCode,
      seq: sequence,
      id: `${year}-${dialCode}${formattedSeq}`
    };
  }

  // Generate realistic companies
  generateCompanies(count) {
    console.log(`📦 Generating ${count} companies...`);
    const companies = [];
    
    for (let i = 0; i < count; i++) {
      const country = this.selectWeighted(this.patterns.countries);
      const companyType = faker.helpers.arrayElement([
        'Development', 'Construction', 'Consultancy', 'Investment', 'Real Estate'
      ]);
      
      const company = {
        id: `company:gen_${i + 1}`,
        name: `${faker.company.name()} ${companyType}`,
        name_short: faker.company.companySuffix(),
        abbreviation: faker.hacker.abbreviation().toUpperCase(),
        city: this.getCityForCountry(country),
        country: country,
        reg_no: faker.string.alphanumeric({ length: 8, casing: 'upper' }),
        tax_no: faker.string.numeric(10),
        time: {
          created_at: faker.date.past({ years: 3 }),
          updated_at: faker.date.recent({ days: 30 })
        }
      };
      
      companies.push(company);
    }
    
    return companies;
  }

  // Generate realistic contacts linked to companies
  generateContacts(companies, averagePerCompany = 2.5) {
    const totalContacts = Math.floor(companies.length * averagePerCompany);
    console.log(`👥 Generating ${totalContacts} contacts...`);
    
    const contacts = [];
    
    for (let i = 0; i < totalContacts; i++) {
      const company = faker.helpers.arrayElement(companies);
      const firstName = faker.person.firstName();
      const lastName = faker.person.lastName();
      
      const contact = {
        id: `contacts:gen_${i + 1}`,
        first_name: firstName,
        last_name: lastName,
        full_name: `${firstName} ${lastName}`,
        email: faker.internet.email({ firstName, lastName, provider: 'example.com' }),
        phone: `+${company.country === 'U.A.E.' ? '971' : '966'}${faker.string.numeric(8)}`,
        position: faker.helpers.arrayElement([
          'Project Manager', 'Development Manager', 'CEO', 'COO', 
          'Design Manager', 'Technical Manager', 'Operations Director'
        ]),
        company: company.id,
        time: {
          created_at: faker.date.between({ 
            from: company.time.created_at, 
            to: new Date() 
          }),
          updated_at: faker.date.recent({ days: 14 })
        }
      };
      
      contacts.push(contact);
    }
    
    return contacts;
  }

  // Generate realistic projects
  generateProjects(count, startYear = 25) {
    console.log(`🏗️  Generating ${count} projects...`);
    const projects = [];
    
    // Track sequence numbers per country/year
    const sequences = {};
    
    for (let i = 0; i < count; i++) {
      const country = this.selectWeighted(this.patterns.countries);
      const year = faker.helpers.arrayElement([startYear - 1, startYear, startYear + 1]);
      
      // Track sequences
      const key = `${year}_${country}`;
      if (!sequences[key]) sequences[key] = 0;
      sequences[key]++;
      
      const projectNumber = this.generateProjectNumber(year, country, sequences[key]);
      
      const projectName = this.generateProjectName();
      
      const project = {
        id: `projects:${projectNumber.id.replace('-', '_')}`,
        name: projectName,
        name_short: this.shortenProjectName(projectName),
        activity: faker.helpers.arrayElement(this.patterns.projectActivities),
        package: faker.helpers.arrayElement(['Full Service', 'Design Only', 'Consultancy', 'Management']),
        status: this.selectWeighted(this.patterns.projectStatus),
        stage: this.selectWeighted(this.patterns.stages),
        area: this.generateArea(country),
        city: this.getCityForCountry(country),
        country: country,
        folder: `${projectNumber.id} ${this.shortenProjectName(projectName)}`,
        number: projectNumber,
        time: {
          created_at: faker.date.past({ years: 2 }),
          updated_at: faker.date.recent({ days: 60 })
        }
      };
      
      projects.push(project);
    }
    
    return projects;
  }

  // Generate realistic fee proposals linked to projects and companies
  generateFeeProposals(projects, companies, contacts, averagePerProject = 1.2) {
    const totalProposals = Math.floor(projects.length * averagePerProject);
    console.log(`📋 Generating ${totalProposals} fee proposals...`);
    
    const proposals = [];
    
    for (let i = 0; i < totalProposals; i++) {
      const project = faker.helpers.arrayElement(projects);
      const company = faker.helpers.arrayElement(companies);
      const companyContacts = contacts.filter(c => c.company === company.id);
      const contact = companyContacts.length > 0 
        ? faker.helpers.arrayElement(companyContacts)
        : faker.helpers.arrayElement(contacts);

      const issueDate = faker.date.between({
        from: project.time.created_at,
        to: new Date()
      });
      
      const proposal = {
        id: `fee:${project.number.id.replace('-', '_')}_${i + 1}`,
        name: `Fee Proposal - ${project.name}`,
        number: `FP-${project.number.id}-${i + 1}`,
        project_id: project.id,
        company_id: company.id,
        contact_id: contact.id,
        status: this.selectWeighted(this.patterns.feeStatus),
        issue_date: this.formatDate(issueDate),
        activity: project.activity,
        package: project.package,
        strap_line: 'sensory design studio',
        staff_name: faker.helpers.arrayElement([
          'Martin Murphy', 'Sarah Johnson', 'Ahmed Al-Rashid', 'Lisa Chen'
        ]),
        staff_email: 'team@emittiv.com',
        staff_phone: '+971 50 123 4567',
        staff_position: 'Design Director',
        rev: faker.number.int({ min: 0, max: 3 }),
        revisions: this.generateRevisions(faker.number.int({ min: 0, max: 3 })),
        time: {
          created_at: issueDate,
          updated_at: faker.date.between({ from: issueDate, to: new Date() })
        }
      };
      
      proposals.push(proposal);
    }
    
    return proposals;
  }

  // Helper methods
  generateProjectName() {
    const types = ['Tower', 'Mall', 'Hotel', 'Resort', 'Complex', 'Plaza', 'Center'];
    const adjectives = ['Grand', 'Royal', 'Modern', 'Elite', 'Premium', 'Luxury', 'Urban'];
    const locations = ['Marina', 'Downtown', 'Hills', 'Bay', 'Park', 'Square', 'Gardens'];
    
    return `${faker.helpers.arrayElement(adjectives)} ${faker.helpers.arrayElement(locations)} ${faker.helpers.arrayElement(types)}`;
  }

  shortenProjectName(name) {
    const words = name.split(' ');
    if (words.length <= 2) return name;
    return words.slice(-2).join(' ');
  }

  getCityForCountry(country) {
    const cities = {
      'U.A.E.': ['Dubai', 'Abu Dhabi', 'Sharjah', 'Ajman', 'Ras Al Khaimah'],
      'Saudi Arabia': ['Riyadh', 'Jeddah', 'Mecca', 'Medina', 'Dammam'],
      'Qatar': ['Doha', 'Al Rayyan', 'Al Wakrah'],
      'Kuwait': ['Kuwait City', 'Al Ahmadi', 'Hawalli'],
      'Bahrain': ['Manama', 'Riffa', 'Muharraq']
    };
    
    return faker.helpers.arrayElement(cities[country] || cities['U.A.E.']);
  }

  generateArea(country) {
    const areas = {
      'U.A.E.': ['Dubai Marina', 'Downtown Dubai', 'Business Bay', 'DIFC', 'Abu Dhabi Corniche'],
      'Saudi Arabia': ['King Fahd District', 'Olaya', 'Al Malqa', 'Diplomatic Quarter'],
      'Qatar': ['West Bay', 'The Pearl', 'Lusail', 'Education City']
    };
    
    return faker.helpers.arrayElement(areas[country] || areas['U.A.E.']);
  }

  formatDate(date) {
    const year = date.getFullYear().toString().slice(-2);
    const month = (date.getMonth() + 1).toString().padStart(2, '0');
    const day = date.getDate().toString().padStart(2, '0');
    return `${year}${month}${day}`;
  }

  generateRevisions(count) {
    const revisions = [];
    for (let i = 0; i < count; i++) {
      revisions.push({
        revision_number: i + 1,
        revision_date: faker.date.past({ years: 1 }),
        author_email: 'team@emittiv.com',
        author_name: 'Design Team',
        notes: faker.lorem.sentence()
      });
    }
    return revisions;
  }

  // Save data to files
  async saveDataset(name, data) {
    const filePath = path.join(this.outputDir, `${name}.json`);
    await fs.writeFile(filePath, JSON.stringify(data, null, 2));
    console.log(`💾 Saved ${data.length} ${name} to ${filePath}`);
  }

  // Generate SQL INSERT statements for SurrealDB
  async generateSQLInserts(datasetName, data, tableName) {
    const sqlPath = path.join(this.outputDir, `${datasetName}_inserts.surql`);
    const stream = createWriteStream(sqlPath);
    
    stream.write(`-- Generated INSERT statements for ${tableName}\n`);
    stream.write(`-- Dataset: ${datasetName}\n`);
    stream.write(`-- Count: ${data.length}\n\n`);
    
    for (const record of data) {
      const { id, ...recordData } = record;
      const sql = `CREATE ${id} CONTENT ${JSON.stringify(recordData, null, 0)};\n`;
      stream.write(sql);
    }
    
    stream.end();
    console.log(`📝 Generated SQL inserts for ${tableName}: ${sqlPath}`);
  }

  // Generate complete dataset for specified scale
  async generateDataset(scale = 'medium') {
    console.log(`🎯 Generating ${scale} scale dataset...`);
    
    const config = this.config.dataScaling[scale];
    if (!config) {
      throw new Error(`Unknown scale: ${scale}. Available: ${Object.keys(this.config.dataScaling).join(', ')}`);
    }
    
    console.log(`📊 Target counts:`, config);
    
    // Generate data in dependency order
    const companies = this.generateCompanies(config.companies);
    await this.saveDataset(`${scale}_companies`, companies);
    await this.generateSQLInserts(`${scale}_companies`, companies, 'company');
    
    const contacts = this.generateContacts(companies);
    await this.saveDataset(`${scale}_contacts`, contacts);
    await this.generateSQLInserts(`${scale}_contacts`, contacts, 'contacts');
    
    const projects = this.generateProjects(config.projects);
    await this.saveDataset(`${scale}_projects`, projects);
    await this.generateSQLInserts(`${scale}_projects`, projects, 'projects');
    
    const proposals = this.generateFeeProposals(projects, companies, contacts);
    await this.saveDataset(`${scale}_proposals`, proposals);
    await this.generateSQLInserts(`${scale}_proposals`, proposals, 'fee');
    
    // Generate summary
    const summary = {
      scale,
      generated_at: new Date().toISOString(),
      counts: {
        companies: companies.length,
        contacts: contacts.length,
        projects: projects.length,
        proposals: proposals.length
      },
      files: [
        `${scale}_companies.json`,
        `${scale}_contacts.json`, 
        `${scale}_projects.json`,
        `${scale}_proposals.json`,
        `${scale}_companies_inserts.surql`,
        `${scale}_contacts_inserts.surql`,
        `${scale}_projects_inserts.surql`,
        `${scale}_proposals_inserts.surql`
      ]
    };
    
    await this.saveDataset(`${scale}_summary`, [summary]);
    
    console.log('✅ Dataset generation complete!');
    console.log('📈 Summary:', summary.counts);
    
    return summary;
  }

  async cleanup() {
    if (this.db) {
      await this.db.close();
    }
  }
}

// CLI interface
async function main() {
  const args = process.argv.slice(2);
  const scale = args[0] || 'medium';
  const availableScales = ['small', 'medium', 'large', 'extreme'];
  
  if (!availableScales.includes(scale)) {
    console.error(`❌ Invalid scale: ${scale}`);
    console.error(`Available scales: ${availableScales.join(', ')}`);
    process.exit(1);
  }
  
  const generator = new DatasetGenerator();
  
  try {
    await generator.initialize();
    await generator.generateDataset(scale);
  } catch (error) {
    console.error('❌ Generation failed:', error);
    process.exit(1);
  } finally {
    await generator.cleanup();
  }
}

// Run if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}

export default DatasetGenerator;