/**
 * E2E Tests for Company and Contact CRUD Operations via Agent API
 *
 * Tests company creation, contact creation linked to companies,
 * listing with filters, and relationship verification.
 * All test data uses "DELETE ME" prefix.
 *
 * Requires the Tauri app + Agent API server running on localhost:3100.
 */

import { describe, test, expect, beforeAll, afterAll } from 'vitest';
import {
  AgentApiClient,
  type AgentCompanyResponse,
  type AgentContactResponse
} from '$lib/api/agent';

const AGENT_URL = process.env.EFEES_AGENT_URL || 'http://localhost:3100';

const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
const TEST_PREFIX = `DELETE ME - E2E CompCont ${timestamp}`;

describe('Company & Contact CRUD Operations - Agent API E2E', () => {
  let client: AgentApiClient;
  let serverAvailable = false;

  let createdCompanyId: string | undefined;
  let createdContactId: string | undefined;

  beforeAll(async () => {
    client = new AgentApiClient(AGENT_URL);

    try {
      const health = await client.health();
      serverAvailable = health.status === 'ok';
      console.log(`Agent API health: ${JSON.stringify(health)}`);
    } catch {
      console.warn('Agent API server not available — company/contact E2E tests will be skipped');
      serverAvailable = false;
    }
  });

  afterAll(async () => {
    if (!serverAvailable) return;
    console.log(`Company/Contact E2E: created records with "${TEST_PREFIX}" prefix`);
  });

  test('should skip if agent server is not running', () => {
    if (!serverAvailable) {
      console.log('Skipping company/contact E2E tests — agent server not available');
    }
    expect(true).toBe(true);
  });

  test('should create a company via POST /api/companies', async () => {
    if (!serverAvailable) return;

    const companyResp = await client.createCompany({
      name: `${TEST_PREFIX} - Dubai Construction LLC`,
      name_short: 'DEL-DCL',
      abbreviation: 'DELDCL',
      city: 'Dubai',
      country: 'United Arab Emirates'
    });

    expect(companyResp.id).toBeTruthy();
    expect(companyResp.name).toContain('DELETE ME');
    expect(companyResp.name_short).toBe('DEL-DCL');
    expect(companyResp.abbreviation).toBe('DELDCL');
    expect(companyResp.city).toBe('Dubai');
    expect(companyResp.country).toBe('United Arab Emirates');

    createdCompanyId = companyResp.id;
    console.log(`Created company: ${companyResp.id}`);
  });

  test('should list companies and find the created one', async () => {
    if (!serverAvailable || !createdCompanyId) return;

    const listResp = await client.listCompanies();

    expect(listResp.items).toBeDefined();
    expect(Array.isArray(listResp.items)).toBe(true);
    expect(listResp.total).toBeGreaterThanOrEqual(1);

    const found = listResp.items.find(c => c.id === createdCompanyId);
    expect(found).toBeDefined();
    expect(found!.name).toContain('DELETE ME');
    expect(found!.city).toBe('Dubai');
  });

  test('should create a contact linked to the company via POST /api/contacts', async () => {
    if (!serverAvailable || !createdCompanyId) return;

    const contactResp = await client.createContact({
      first_name: `${TEST_PREFIX} - Ahmed`,
      last_name: 'TestUser',
      email: `delete-me-contact-${timestamp}@test.com`,
      phone: '+971509876543',
      position: 'Project Manager (DELETE ME Test)',
      company_id: createdCompanyId
    });

    expect(contactResp.id).toBeTruthy();
    expect(contactResp.first_name).toContain('DELETE ME');
    expect(contactResp.last_name).toBe('TestUser');
    expect(contactResp.email).toContain('delete-me');
    expect(contactResp.company_id).toBe(createdCompanyId);

    createdContactId = contactResp.id;
    console.log(`Created contact: ${contactResp.id} linked to company ${createdCompanyId}`);
  });

  test('should list contacts filtered by company_id', async () => {
    if (!serverAvailable || !createdCompanyId || !createdContactId) return;

    const filtered = await client.listContacts(createdCompanyId);

    expect(filtered.items).toBeDefined();
    expect(Array.isArray(filtered.items)).toBe(true);

    // All returned contacts should belong to our company
    for (const contact of filtered.items) {
      expect(contact.company_id).toBe(createdCompanyId);
    }

    // Our created contact should be in the filtered results
    const found = filtered.items.find(c => c.id === createdContactId);
    expect(found).toBeDefined();
    expect(found!.first_name).toContain('DELETE ME');
  });

  test('should get contact by ID and verify company_id reference', async () => {
    if (!serverAvailable || !createdContactId || !createdCompanyId) return;

    const contact = await client.getContact(createdContactId);

    expect(contact.id).toBe(createdContactId);
    expect(contact.first_name).toContain('DELETE ME');
    expect(contact.last_name).toBe('TestUser');
    expect(contact.email).toContain('delete-me');
    expect(contact.phone).toBe('+971509876543');
    expect(contact.company_id).toBe(createdCompanyId);
  });

  test('should return 404 for non-existent contact', async () => {
    if (!serverAvailable) return;

    await expect(client.getContact('nonexistent-contact-id-99999')).rejects.toThrow();
  });

  test('should list all contacts without filter', async () => {
    if (!serverAvailable) return;

    const allContacts = await client.listContacts();

    expect(allContacts.items).toBeDefined();
    expect(Array.isArray(allContacts.items)).toBe(true);
    expect(allContacts.total).toBeGreaterThanOrEqual(0);
    expect(typeof allContacts.total).toBe('number');
  });
});
