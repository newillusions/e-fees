import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock fetch globally
const mockFetch = vi.fn();
vi.stubGlobal('fetch', mockFetch);

describe('scope API', () => {
  beforeEach(() => {
    mockFetch.mockReset();
  });

  describe('getScope', () => {
    it('returns null on 404', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        json: () => Promise.resolve({ message: 'Not found' })
      });

      const { getScope } = await import('./scope');
      const result = await getScope('25_97101_1');
      expect(result).toBeNull();
    });

    it('returns ScopeAssembly on success (unwraps data envelope)', async () => {
      const mockScope = {
        id: 'scope:123',
        fee_id: '25_97101_1',
        clauses: [{ number: '1.0', title: 'Admin', clauses: [] }],
        generated_text: '1.0 ADMIN',
        llm_polished: true,
        created_at: '2026-01-01',
        updated_at: '2026-01-01'
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockScope })
      });

      const { getScope } = await import('./scope');
      const result = await getScope('25_97101_1');
      expect(result).toEqual(mockScope);
    });

    it('throws on non-404 errors', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
        json: () => Promise.resolve({ message: 'DB error' })
      });

      const { getScope } = await import('./scope');
      await expect(getScope('25_97101_1')).rejects.toThrow('DB error');
    });
  });

  describe('generateScope', () => {
    it('passes stages in request body', async () => {
      const mockResult = {
        clauses: [{ number: '1.0', title: 'Admin', clauses: [] }],
        generated_text: '1.0 ADMIN',
        llm_polished: false,
        stages_snapshot: ['Schematic Design']
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockResult })
      });

      const { generateScope } = await import('./scope');
      const result = await generateScope({
        fee_id: '25_97101_1',
        polish: false,
        stages: [{ name: 'Schematic Design', code: 'SD', is_post_contract: false, order: 1 }]
      });

      expect(result.stages_snapshot).toEqual(['Schematic Design']);
      const fetchCall = mockFetch.mock.calls[0];
      const body = JSON.parse(fetchCall[1].body);
      expect(body.stages).toHaveLength(1);
      expect(body.stages[0].name).toBe('Schematic Design');
    });

    it('includes stages in request and receives stages_snapshot', async () => {
      const mockResult = {
        clauses: [{ number: '1.0', title: 'Admin', clauses: [] }],
        generated_text: 'During Schematic Design...',
        llm_polished: true,
        stages_snapshot: ['Schematic Design', 'Design Development'],
        current_revision: 1
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ data: mockResult })
      });

      const { generateScope } = await import('./scope');
      const result = await generateScope({
        fee_id: '25_97101_1',
        polish: true,
        stages: [
          { name: 'Schematic Design', code: 'SD', is_post_contract: false, order: 1 },
          { name: 'Design Development', code: 'DD', is_post_contract: false, order: 2 }
        ]
      });

      expect(result.stages_snapshot).toEqual(['Schematic Design', 'Design Development']);
      expect(result.current_revision).toBe(1);
    });
  });

  describe('getClauseSelection', () => {
    const mockResponse = {
      fee_id: '25_97101_1',
      has_custom_selection: false,
      selections: []
    };

    it('omits the conditions query param when none are passed', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve(mockResponse)
      });

      const { getClauseSelection } = await import('./scope');
      await getClauseSelection('25_97101_1');

      const url = mockFetch.mock.calls[0][0] as string;
      expect(url).toBe('http://10.0.21.81:3201/scope/25_97101_1/clause-selection');
    });

    it('omits the conditions query param when an empty object is passed', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve(mockResponse)
      });

      const { getClauseSelection } = await import('./scope');
      await getClauseSelection('25_97101_1', {});

      const url = mockFetch.mock.calls[0][0] as string;
      expect(url).toBe('http://10.0.21.81:3201/scope/25_97101_1/clause-selection');
    });

    it('JSON-encodes conditions into the query string (Stage 2 gating)', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve(mockResponse)
      });

      const { getClauseSelection } = await import('./scope');
      await getClauseSelection('25_97101_1', { project_type: 'hospitality', min_area: 500 });

      const url = mockFetch.mock.calls[0][0] as string;
      const parsed = new URL(url);
      expect(parsed.pathname).toBe('/scope/25_97101_1/clause-selection');
      expect(JSON.parse(parsed.searchParams.get('conditions')!)).toEqual({
        project_type: 'hospitality',
        min_area: 500
      });
    });
  });

  describe('getClauseSuggestions', () => {
    it('fetches the suggestions endpoint for the given fee', async () => {
      const mockResponse = {
        fee_id: '25_97101_1',
        suggestions: [
          {
            clause_id: 'abc123',
            title: 'Fees / Payment Terms',
            category: 'Commercial',
            usage_count: 12,
            sample_project_numbers: ['24-96603', '25-97103'],
            classified_at: '2026-07-10T16:44:11.102650516Z'
          }
        ]
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve(mockResponse)
      });

      const { getClauseSuggestions } = await import('./scope');
      const result = await getClauseSuggestions('25_97101_1');

      const url = mockFetch.mock.calls[0][0] as string;
      expect(url).toBe('http://10.0.21.81:3201/scope/25_97101_1/clause-suggestions');
      expect(result).toEqual(mockResponse);
    });

    it('returns an empty suggestions array without throwing when unmined', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 200,
        json: () => Promise.resolve({ fee_id: '25_97101_1', suggestions: [] })
      });

      const { getClauseSuggestions } = await import('./scope');
      const result = await getClauseSuggestions('25_97101_1');

      expect(result.suggestions).toEqual([]);
    });

    it('propagates errors on non-ok responses', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
        json: () => Promise.resolve({ message: 'DB error' })
      });

      const { getClauseSuggestions } = await import('./scope');
      await expect(getClauseSuggestions('25_97101_1')).rejects.toThrow('DB error');
    });
  });
});
