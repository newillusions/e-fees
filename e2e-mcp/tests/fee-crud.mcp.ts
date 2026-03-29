/**
 * E2E Tests for Fee CRUD Operations via Agent API
 *
 * Tests fee creation, retrieval, listing, filtering, and Excel export
 * using the HTTP Agent API server. All test data uses "DELETE ME" prefix.
 *
 * Requires the Tauri app + Agent API server running on localhost:3100.
 */

import { describe, test, expect, beforeAll, afterAll } from 'vitest';
import {
  AgentApiClient,
  type AgentFeeResponse,
  type AgentProjectResponse,
  type AgentCompanyResponse,
  type AgentContactResponse
} from '$lib/api/agent';

const AGENT_URL = process.env.EFEES_AGENT_URL || 'http://localhost:3100';
const API_KEY = process.env.EFEES_AGENT_API_KEY || undefined;

const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
const TEST_PREFIX = `DELETE ME - E2E Fee ${timestamp}`;

describe('Fee CRUD Operations - Agent API E2E', () => {
  let client: AgentApiClient;
  let serverAvailable = false;

  // IDs of records created during tests (for cleanup)
  let createdProjectId: string | undefined;
  let createdCompanyId: string | undefined;
  let createdContactId: string | undefined;
  const createdFeeIds: string[] = [];

  beforeAll(async () => {
    client = new AgentApiClient(AGENT_URL);

    // Check if the agent server is running
    try {
      const health = await client.health();
      serverAvailable = health.status === 'ok';
      console.log(`Agent API health: ${JSON.stringify(health)}`);
    } catch {
      console.warn('Agent API server not available — fee E2E tests will be skipped');
      serverAvailable = false;
    }
  });

  afterAll(async () => {
    if (!serverAvailable) return;
    // Cleanup is handled by the "DELETE ME" pattern — records with this prefix
    // can be cleaned up via the cleanup-utilities or manually.
    // Individual test cleanup would require DELETE endpoints (not yet available).
    console.log(
      `Fee E2E: created ${createdFeeIds.length} fee(s), 1 project, 1 company, 1 contact with "${TEST_PREFIX}" prefix`
    );
  });

  test('should skip if agent server is not running', () => {
    if (!serverAvailable) {
      console.log('Skipping fee E2E tests — agent server not available');
    }
    // This test always passes — it's a guard for the describe block
    expect(true).toBe(true);
  });

  test('should create prerequisite project, company, and contact', async () => {
    if (!serverAvailable) return;

    // Create a project to link fees to
    const projectResp = await client.createProject({
      name: `${TEST_PREFIX} - Project`,
      name_short: 'DEL-FEE',
      status: 'Active',
      country: 'United Arab Emirates',
      city: 'Dubai',
      area: 'Downtown Dubai',
      number: `26-97199`,
      folder: `/tmp/delete-me-fee-test-${timestamp}`
    });
    expect(projectResp.id).toBeTruthy();
    createdProjectId = projectResp.id;
    console.log(`Created prerequisite project: ${projectResp.id}`);

    // Create a company
    const companyResp = await client.createCompany({
      name: `${TEST_PREFIX} - Company`,
      name_short: 'DEL-CO',
      abbreviation: 'DELCO',
      city: 'Dubai',
      country: 'United Arab Emirates'
    });
    expect(companyResp.id).toBeTruthy();
    createdCompanyId = companyResp.id;
    console.log(`Created prerequisite company: ${companyResp.id}`);

    // Create a contact
    const contactResp = await client.createContact({
      first_name: `${TEST_PREFIX} - Contact`,
      last_name: 'TestUser',
      email: `delete-me-fee-${timestamp}@test.com`,
      phone: '+971501234567',
      position: 'Test Manager (DELETE ME)',
      company_id: createdCompanyId
    });
    expect(contactResp.id).toBeTruthy();
    createdContactId = contactResp.id;
    console.log(`Created prerequisite contact: ${contactResp.id}`);
  });

  test('should create a fee via POST /api/fees', async () => {
    if (!serverAvailable || !createdProjectId || !createdCompanyId || !createdContactId) return;

    const feeResp = await client.createFee({
      name: `${TEST_PREFIX} - Lighting Design Fee`,
      number: `E-26-97199-01`,
      project_id: createdProjectId,
      company_id: createdCompanyId,
      contact_id: createdContactId,
      status: 'Draft'
    });

    expect(feeResp.id).toBeTruthy();
    expect(feeResp.name).toContain('DELETE ME');
    expect(feeResp.number).toBe('E-26-97199-01');
    expect(feeResp.status).toBe('Draft');
    expect(feeResp.rev).toBe(1);
    expect(typeof feeResp.total_fee).toBe('number');
    expect(typeof feeResp.currency).toBe('string');

    createdFeeIds.push(feeResp.id);
    console.log(`Created fee: ${feeResp.id}`);
  });

  test('should retrieve the fee via GET /api/fees/:id', async () => {
    if (!serverAvailable || createdFeeIds.length === 0) return;

    const feeId = createdFeeIds[0];
    const fee = await client.getFee(feeId);

    expect(fee.id).toBe(feeId);
    expect(fee.name).toContain('DELETE ME');
    expect(fee.number).toBe('E-26-97199-01');
    expect(fee.project_id).toBe(createdProjectId);
    expect(fee.company_id).toBe(createdCompanyId);
  });

  test('should list fees and find the created one', async () => {
    if (!serverAvailable || createdFeeIds.length === 0) return;

    const listResp = await client.listFees();

    expect(listResp.items).toBeDefined();
    expect(Array.isArray(listResp.items)).toBe(true);
    expect(listResp.total).toBeGreaterThanOrEqual(1);

    const found = listResp.items.find(f => f.id === createdFeeIds[0]);
    expect(found).toBeDefined();
    expect(found!.name).toContain('DELETE ME');
  });

  test('should filter fees by project_id', async () => {
    if (!serverAvailable || !createdProjectId || createdFeeIds.length === 0) return;

    const filtered = await client.listFees(createdProjectId);

    expect(filtered.items).toBeDefined();
    expect(Array.isArray(filtered.items)).toBe(true);

    // All returned fees should belong to our project
    for (const fee of filtered.items) {
      expect(fee.project_id).toBe(createdProjectId);
    }

    // Our created fee should be in the filtered results
    const found = filtered.items.find(f => f.id === createdFeeIds[0]);
    expect(found).toBeDefined();
  });

  test('should export fee to Excel via POST /api/fees/:id/export', async () => {
    if (!serverAvailable || createdFeeIds.length === 0) return;

    const feeId = createdFeeIds[0];

    // Use raw fetch for the export endpoint since AgentApiClient doesn't have an export method
    const headers: Record<string, string> = { 'Content-Type': 'application/json' };
    if (API_KEY) {
      headers['X-API-Key'] = API_KEY;
    }

    const response = await fetch(`${AGENT_URL}/api/fees/${feeId}/export`, {
      method: 'POST',
      headers
    });

    expect(response.ok).toBe(true);

    const exportResult = (await response.json()) as { path: string; filename: string };
    expect(exportResult.path).toBeTruthy();
    expect(exportResult.filename).toBeTruthy();
    expect(exportResult.filename).toMatch(/\.xlsx$/);
    expect(exportResult.filename).toContain('fee-');

    console.log(`Exported fee to: ${exportResult.path} (${exportResult.filename})`);
  });

  test('should return 404 for non-existent fee', async () => {
    if (!serverAvailable) return;

    await expect(client.getFee('nonexistent-fee-id-12345')).rejects.toThrow();
  });
});
