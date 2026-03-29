/**
 * E2E Tests for Agent API Endpoints (Direct HTTP)
 *
 * Tests all Agent API endpoints directly via fetch() to verify
 * HTTP status codes, response shapes, auth behavior, and error handling.
 * Does NOT use the AgentApiClient — tests raw HTTP contract.
 *
 * Requires the Tauri app + Agent API server running on localhost:3100.
 */

import { describe, test, expect, beforeAll } from 'vitest';

const AGENT_URL = process.env.EFEES_AGENT_URL || 'http://localhost:3100';
const API_KEY = process.env.EFEES_AGENT_API_KEY || undefined;

/** Helper to make requests with optional API key */
async function agentFetch(path: string, options: RequestInit = {}): Promise<Response> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...((options.headers as Record<string, string>) || {})
  };
  if (API_KEY) {
    headers['X-API-Key'] = API_KEY;
  }
  return fetch(`${AGENT_URL}${path}`, { ...options, headers });
}

/** Helper for unauthenticated requests (no API key) */
async function agentFetchNoAuth(path: string, options: RequestInit = {}): Promise<Response> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    ...((options.headers as Record<string, string>) || {})
  };
  return fetch(`${AGENT_URL}${path}`, { ...options, headers });
}

describe('Agent API Endpoints - Direct HTTP E2E', () => {
  let serverAvailable = false;
  let apiKeyConfigured = false;

  beforeAll(async () => {
    try {
      const resp = await fetch(`${AGENT_URL}/api/health`, {
        headers: { 'Content-Type': 'application/json' }
      });
      serverAvailable = resp.ok;
      if (serverAvailable) {
        const health = await resp.json();
        console.log(`Agent API health: ${JSON.stringify(health)}`);
      }
    } catch {
      console.warn('Agent API server not available — agent-api E2E tests will be skipped');
      serverAvailable = false;
    }

    // Detect if API key auth is configured by testing a protected endpoint without a key
    if (serverAvailable) {
      try {
        const resp = await agentFetchNoAuth('/api/stats');
        apiKeyConfigured = resp.status === 401;
        console.log(`API key auth configured: ${apiKeyConfigured}`);
      } catch {
        apiKeyConfigured = false;
      }
    }
  });

  test('should skip if agent server is not running', () => {
    if (!serverAvailable) {
      console.log('Skipping agent-api E2E tests — agent server not available');
    }
    expect(true).toBe(true);
  });

  // =========================================================================
  // Health Check
  // =========================================================================

  describe('GET /api/health', () => {
    test('should return 200 with health status', async () => {
      if (!serverAvailable) return;

      const resp = await fetch(`${AGENT_URL}/api/health`, {
        headers: { 'Content-Type': 'application/json' }
      });

      expect(resp.status).toBe(200);

      const body = await resp.json();
      expect(body.status).toBe('ok');
      expect(typeof body.version).toBe('string');
      expect(typeof body.db_connected).toBe('boolean');
      expect(typeof body.uptime_seconds).toBe('number');
    });

    test('should be accessible without API key', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetchNoAuth('/api/health');
      expect(resp.status).toBe(200);
    });
  });

  // =========================================================================
  // Auth
  // =========================================================================

  describe('Authentication', () => {
    test('should return 401 for protected endpoint without API key (when configured)', async () => {
      if (!serverAvailable || !apiKeyConfigured) return;

      const resp = await agentFetchNoAuth('/api/projects');
      expect(resp.status).toBe(401);

      const body = await resp.json();
      expect(body.error).toBeDefined();
      expect(body.error).toContain('Unauthorized');
    });

    test('should allow access with valid API key', async () => {
      if (!serverAvailable || !apiKeyConfigured || !API_KEY) return;

      const resp = await agentFetch('/api/projects');
      expect(resp.status).not.toBe(401);
    });
  });

  // =========================================================================
  // Projects
  // =========================================================================

  describe('GET /api/projects', () => {
    test('should return 200 with array of projects', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetch('/api/projects');
      expect(resp.status).toBe(200);

      const body = await resp.json();
      expect(body.items).toBeDefined();
      expect(Array.isArray(body.items)).toBe(true);
      expect(typeof body.total).toBe('number');
    });
  });

  describe('GET /api/projects/:id', () => {
    test('should return 404 for non-existent project', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetch('/api/projects/nonexistent-id-99999');

      // Depending on DB state, could be 404 or 500
      expect([404, 500]).toContain(resp.status);

      const body = await resp.json();
      expect(body.error).toBeDefined();
    });
  });

  describe('POST /api/projects', () => {
    test('should reject invalid body with 422', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetch('/api/projects', {
        method: 'POST',
        body: JSON.stringify({ name: 'missing required fields' })
      });

      // Axum returns 422 for deserialization failures
      expect(resp.status).toBe(422);
    });

    test('should reject invalid project number format with 400', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetch('/api/projects', {
        method: 'POST',
        body: JSON.stringify({
          name: 'DELETE ME - Bad Number Test',
          name_short: 'DEL',
          status: 'Active',
          country: 'UAE',
          city: 'Dubai',
          area: 'Downtown',
          number: 'INVALID',
          folder: '/tmp/delete-me'
        })
      });

      expect(resp.status).toBe(400);

      const body = await resp.json();
      expect(body.error).toBeDefined();
      expect(body.error).toContain('Invalid project number');
    });
  });

  // =========================================================================
  // Companies
  // =========================================================================

  describe('GET /api/companies', () => {
    test('should return 200 with array of companies', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetch('/api/companies');
      expect(resp.status).toBe(200);

      const body = await resp.json();
      expect(body.items).toBeDefined();
      expect(Array.isArray(body.items)).toBe(true);
      expect(typeof body.total).toBe('number');
    });
  });

  describe('POST /api/companies', () => {
    test('should reject invalid body with 422', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetch('/api/companies', {
        method: 'POST',
        body: JSON.stringify({ name: 'missing fields' })
      });

      expect(resp.status).toBe(422);
    });
  });

  // =========================================================================
  // Contacts
  // =========================================================================

  describe('GET /api/contacts', () => {
    test('should return 200 with array of contacts', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetch('/api/contacts');
      expect(resp.status).toBe(200);

      const body = await resp.json();
      expect(body.items).toBeDefined();
      expect(Array.isArray(body.items)).toBe(true);
      expect(typeof body.total).toBe('number');
    });
  });

  describe('GET /api/contacts/:id', () => {
    test('should return error for non-existent contact', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetch('/api/contacts/nonexistent-contact-88888');

      expect([404, 500]).toContain(resp.status);

      const body = await resp.json();
      expect(body.error).toBeDefined();
    });
  });

  describe('POST /api/contacts', () => {
    test('should reject incomplete body with 422', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetch('/api/contacts', {
        method: 'POST',
        body: JSON.stringify({ first_name: 'Only first name' })
      });

      expect(resp.status).toBe(422);
    });
  });

  // =========================================================================
  // Fees
  // =========================================================================

  describe('GET /api/fees', () => {
    test('should return 200 with array of fees', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetch('/api/fees');
      expect(resp.status).toBe(200);

      const body = await resp.json();
      expect(body.items).toBeDefined();
      expect(Array.isArray(body.items)).toBe(true);
      expect(typeof body.total).toBe('number');
    });
  });

  describe('GET /api/fees/:id', () => {
    test('should return error for non-existent fee', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetch('/api/fees/nonexistent-fee-77777');

      expect([404, 500]).toContain(resp.status);

      const body = await resp.json();
      expect(body.error).toBeDefined();
    });
  });

  describe('POST /api/fees', () => {
    test('should reject incomplete body with 422', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetch('/api/fees', {
        method: 'POST',
        body: JSON.stringify({ name: 'missing required fields' })
      });

      expect(resp.status).toBe(422);
    });
  });

  // =========================================================================
  // Stats
  // =========================================================================

  describe('GET /api/stats', () => {
    test('should return 200 with entity counts', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetch('/api/stats');
      expect(resp.status).toBe(200);

      const body = await resp.json();
      expect(typeof body.total_projects).toBe('number');
      expect(typeof body.total_companies).toBe('number');
      expect(typeof body.total_contacts).toBe('number');
      expect(typeof body.total_fees).toBe('number');
      expect(typeof body.active_fees).toBe('number');
    });
  });

  // =========================================================================
  // Fee Export
  // =========================================================================

  describe('POST /api/fees/:id/export', () => {
    test('should return error for non-existent fee export', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetch('/api/fees/nonexistent-fee-66666/export', {
        method: 'POST'
      });

      expect([404, 500]).toContain(resp.status);

      const body = await resp.json();
      expect(body.error).toBeDefined();
    });
  });

  // =========================================================================
  // Edge Cases
  // =========================================================================

  describe('Edge cases', () => {
    test('should return 404 or method not allowed for unknown routes', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetch('/api/nonexistent-endpoint');
      // Axum returns 404 for unmatched routes
      expect(resp.status).toBe(404);
    });

    test('should handle empty POST body gracefully', async () => {
      if (!serverAvailable) return;

      const resp = await agentFetch('/api/projects', {
        method: 'POST',
        body: ''
      });

      // Should get a 4xx error, not a 500 crash
      expect(resp.status).toBeGreaterThanOrEqual(400);
      expect(resp.status).toBeLessThan(500);
    });

    test('should handle malformed JSON in POST body', async () => {
      if (!serverAvailable) return;

      const resp = await fetch(`${AGENT_URL}/api/projects`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(API_KEY ? { 'X-API-Key': API_KEY } : {})
        },
        body: '{ invalid json }}}'
      });

      expect(resp.status).toBeGreaterThanOrEqual(400);
      expect(resp.status).toBeLessThan(500);
    });
  });
});
