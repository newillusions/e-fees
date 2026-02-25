/**
 * AILX API Client Tests
 *
 * Comprehensive test suite for the AilxApiClient class covering all endpoints,
 * error handling, and network scenarios.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { AilxApiClient } from './ailx';
import type {
  AilxHealthResponse,
  ProductSearchResponse,
  ProductRecommendResponse,
  ProductDetailResponse,
  AssignProjectResponse,
  DuplicateCheckResponse,
  ProductSearchParams,
  ProductRecommendParams,
  AssignProjectRequest,
  ProductSummary
} from './ailx';

// Mock the global fetch function
global.fetch = vi.fn();

// Mock data for testing
const mockProductSummary: ProductSummary = {
  id: 'product:abc123',
  model: 'Aether 4" Round',
  part_number: 'WAC-R4RD1L-N830-BK',
  product_type: 'Recessed Downlight',
  manufacturer_name: 'WAC Lighting',
  wattage: 12,
  lumens: 800,
  cct_kelvin: 3000,
  mounting_type: 'Recessed'
};

const mockHealthResponse: AilxHealthResponse = {
  status: 'healthy',
  version: '1.0.0',
  uptime: 3600,
  services: {
    surrealdb: {
      status: 'connected',
      latency_ms: 5
    },
    elasticsearch: {
      status: 'connected',
      latency_ms: 12
    }
  }
};

const mockProductDetail: ProductDetailResponse = {
  id: 'product:abc123',
  model: 'Aether 4" Round',
  series: 'Aether',
  part_number: 'WAC-R4RD1L-N830-BK',
  catalog_number: 'WAC-R4RD1L-N830-BK',
  product_type: 'Recessed Downlight',
  description: '4" round recessed LED downlight',
  manufacturer: {
    id: 'manufacturer:wac_lighting',
    name: 'WAC Lighting',
    brand: 'WAC',
    country: 'USA',
    website: 'https://www.waclighting.com'
  },
  light_source: {
    output_lumens: 800,
    cct_kelvin: 3000,
    cri: 90
  },
  physical: {
    mounting_type: 'Recessed',
    cutout_diameter_mm: 102
  },
  electrical: {
    wattage: 12,
    voltage_range: '120V'
  },
  optical: {
    beam_angle_deg: 40,
    distribution_pattern: 'Narrow Flood'
  },
  driver: {
    driver_type: 'Integrated',
    dimming_protocol: ['TRIAC', '0-10V']
  },
  environmental: {
    ip_rating: 'IP20',
    wet_location: false
  },
  accessories: [],
  documentation: null,
  taxonomy: null
};

const mockSearchResponse: ProductSearchResponse = {
  total: 25,
  limit: 20,
  offset: 0,
  results: [mockProductSummary]
};

const mockRecommendResponse: ProductRecommendResponse = {
  data: [mockProductSummary],
  total: 15,
  filters_applied: ['mounting_type', 'min_wattage']
};

const mockAssignResponse: AssignProjectResponse = {
  product_id: 'product:abc123',
  project_fixture_id: 'project_fixture:25_97101_abc123',
  project_context: {
    project_number: '25-97101',
    project_name: 'Test Project',
    role: 'specified',
    space_name: 'Lobby',
    quantity: 12,
    status: 'Proposed'
  },
  status: 'linked'
};

const mockDuplicateResponse: DuplicateCheckResponse = {
  exists: true,
  product: {
    id: 'product:abc123',
    model: 'Aether 4" Round',
    part_number: 'WAC-R4RD1L-N830-BK',
    manufacturer_name: 'WAC Lighting'
  }
};

describe('AilxApiClient', () => {
  const mockFetch = vi.mocked(fetch);
  let client: AilxApiClient;

  beforeEach(() => {
    vi.clearAllMocks();
    client = new AilxApiClient('http://10.0.21.11:3000');
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('Constructor', () => {
    it('should use default base URL when none provided', () => {
      const defaultClient = new AilxApiClient();
      expect(defaultClient.baseUrl).toBe('http://10.0.21.11:3000');
    });

    it('should use custom base URL when provided', () => {
      const customClient = new AilxApiClient('http://localhost:4000');
      expect(customClient.baseUrl).toBe('http://localhost:4000');
    });

    it('should remove trailing slash from base URL', () => {
      const customClient = new AilxApiClient('http://localhost:4000/');
      expect(customClient.baseUrl).toBe('http://localhost:4000');
    });
  });

  describe('health', () => {
    it('should return health status when successful', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockHealthResponse
      } as Response);

      const result = await client.health();

      expect(result).toEqual(mockHealthResponse);
      expect(mockFetch).toHaveBeenCalledWith('http://10.0.21.11:3000/api/health', {
        headers: {
          'Content-Type': 'application/json'
        }
      });
    });

    it('should handle degraded status', async () => {
      const degradedResponse: AilxHealthResponse = {
        ...mockHealthResponse,
        status: 'degraded',
        services: {
          ...mockHealthResponse.services,
          elasticsearch: {
            status: 'disconnected',
            error: 'Connection refused'
          }
        }
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => degradedResponse
      } as Response);

      const result = await client.health();

      expect(result.status).toBe('degraded');
      expect(result.services.elasticsearch.status).toBe('disconnected');
    });

    it('should throw error when health check fails', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 503,
        json: async () => ({ error: 'Service Unavailable', message: 'SurrealDB not connected' })
      } as Response);

      await expect(client.health()).rejects.toThrow('SurrealDB not connected');
    });
  });

  describe('searchProducts', () => {
    it('should search products with query string', async () => {
      const params: ProductSearchParams = {
        q: 'downlight',
        manufacturer: 'WAC Lighting',
        limit: 20
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockSearchResponse
      } as Response);

      const result = await client.searchProducts(params);

      expect(result).toEqual(mockSearchResponse);
      expect(mockFetch).toHaveBeenCalledWith(
        'http://10.0.21.11:3000/api/products/search?q=downlight&manufacturer=WAC+Lighting&limit=20',
        expect.objectContaining({
          headers: {
            'Content-Type': 'application/json'
          }
        })
      );
    });

    it('should handle empty search params', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockSearchResponse
      } as Response);

      const result = await client.searchProducts({});

      expect(result).toEqual(mockSearchResponse);
      expect(mockFetch).toHaveBeenCalledWith('http://10.0.21.11:3000/api/products/search', expect.any(Object));
    });

    it('should handle all search filter parameters', async () => {
      const params: ProductSearchParams = {
        q: 'LED',
        manufacturer: 'WAC',
        product_type: 'Downlight',
        min_wattage: 5,
        max_wattage: 15,
        mounting_type: 'Recessed',
        control_protocol: 'DALI',
        limit: 50,
        offset: 10
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockSearchResponse
      } as Response);

      await client.searchProducts(params);

      const calledUrl = (mockFetch.mock.calls[0][0] as string);
      expect(calledUrl).toContain('q=LED');
      expect(calledUrl).toContain('manufacturer=WAC');
      expect(calledUrl).toContain('product_type=Downlight');
      expect(calledUrl).toContain('min_wattage=5');
      expect(calledUrl).toContain('max_wattage=15');
      expect(calledUrl).toContain('mounting_type=Recessed');
      expect(calledUrl).toContain('control_protocol=DALI');
      expect(calledUrl).toContain('limit=50');
      expect(calledUrl).toContain('offset=10');
    });

    it('should handle search errors', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        json: async () => ({ error: 'Search Error', message: 'Elasticsearch query failed' })
      } as Response);

      await expect(client.searchProducts({ q: 'test' })).rejects.toThrow('Elasticsearch query failed');
    });
  });

  describe('recommendProducts', () => {
    it('should get product recommendations with filters', async () => {
      const params: ProductRecommendParams = {
        mounting_type: 'Recessed',
        min_wattage: 10,
        max_wattage: 20,
        min_cri: 90
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockRecommendResponse
      } as Response);

      const result = await client.recommendProducts(params);

      expect(result).toEqual(mockRecommendResponse);
      expect(result.filters_applied).toContain('mounting_type');
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/products/recommend'),
        expect.any(Object)
      );
    });

    it('should handle all recommendation filter parameters', async () => {
      const params: ProductRecommendParams = {
        mounting_type: 'Surface',
        min_wattage: 5,
        max_wattage: 25,
        min_cri: 80,
        product_type: 'Linear',
        manufacturer: 'ERCO',
        limit: 30,
        offset: 5
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockRecommendResponse
      } as Response);

      await client.recommendProducts(params);

      const calledUrl = (mockFetch.mock.calls[0][0] as string);
      expect(calledUrl).toContain('mounting_type=Surface');
      expect(calledUrl).toContain('min_wattage=5');
      expect(calledUrl).toContain('max_wattage=25');
      expect(calledUrl).toContain('min_cri=80');
      expect(calledUrl).toContain('product_type=Linear');
      expect(calledUrl).toContain('manufacturer=ERCO');
      expect(calledUrl).toContain('limit=30');
      expect(calledUrl).toContain('offset=5');
    });

    it('should return empty results when no products match', async () => {
      const emptyResponse: ProductRecommendResponse = {
        data: [],
        total: 0,
        filters_applied: ['min_wattage']
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => emptyResponse
      } as Response);

      const result = await client.recommendProducts({ min_wattage: 1000 });

      expect(result.data).toEqual([]);
      expect(result.total).toBe(0);
    });
  });

  describe('getProduct', () => {
    it('should get product details by ID', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockProductDetail
      } as Response);

      const result = await client.getProduct('abc123');

      expect(result).toEqual(mockProductDetail);
      expect(mockFetch).toHaveBeenCalledWith(
        'http://10.0.21.11:3000/api/products/abc123',
        expect.any(Object)
      );
    });

    it('should handle URL encoding for product IDs', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockProductDetail
      } as Response);

      await client.getProduct('product:abc/123');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://10.0.21.11:3000/api/products/product%3Aabc%2F123',
        expect.any(Object)
      );
    });

    it('should throw error for non-existent product', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        json: async () => ({ error: 'Not Found', message: 'Product abc123 not found' })
      } as Response);

      await expect(client.getProduct('abc123')).rejects.toThrow('Product abc123 not found');
    });

    it('should handle malformed product data', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        json: async () => ({ error: 'Query Error', message: 'Invalid product schema' })
      } as Response);

      await expect(client.getProduct('abc123')).rejects.toThrow('Invalid product schema');
    });
  });

  describe('checkDuplicate', () => {
    it('should check for duplicate products', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockDuplicateResponse
      } as Response);

      const result = await client.checkDuplicate({
        part_number: 'WAC-R4RD1L-N830-BK',
        manufacturer_name: 'WAC Lighting'
      });

      expect(result).toEqual(mockDuplicateResponse);
      expect(result.exists).toBe(true);
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/fixtures/check-duplicate'),
        expect.any(Object)
      );
    });

    it('should return false when product does not exist', async () => {
      const noDuplicateResponse: DuplicateCheckResponse = {
        exists: false,
        product: null
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => noDuplicateResponse
      } as Response);

      const result = await client.checkDuplicate({ part_number: 'NONEXISTENT-123' });

      expect(result.exists).toBe(false);
      expect(result.product).toBeNull();
    });

    it('should require part_number parameter', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 400,
        json: async () => ({ error: 'Validation Error', message: 'part_number query parameter is required' })
      } as Response);

      await expect(client.checkDuplicate({ part_number: '' })).rejects.toThrow('part_number query parameter is required');
    });
  });

  describe('assignProductToProject', () => {
    it('should assign product to project successfully', async () => {
      const assignment: AssignProjectRequest = {
        project_number: '25-97101',
        project_name: 'Test Project',
        role: 'specified',
        space_name: 'Lobby',
        quantity: 12,
        specification_section: '26 51 00',
        status: 'Proposed'
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 201,
        json: async () => mockAssignResponse
      } as Response);

      const result = await client.assignProductToProject('abc123', assignment);

      expect(result).toEqual(mockAssignResponse);
      expect(result.status).toBe('linked');
      expect(mockFetch).toHaveBeenCalledWith(
        'http://10.0.21.11:3000/api/fixtures/abc123/assign-project',
        {
          method: 'POST',
          body: JSON.stringify(assignment),
          headers: {
            'Content-Type': 'application/json'
          }
        }
      );
    });

    it('should handle minimal assignment data', async () => {
      const minimalAssignment: AssignProjectRequest = {
        project_number: '25-97101',
        project_name: 'Test Project'
      };

      mockFetch.mockResolvedValueOnce({
        ok: true,
        status: 201,
        json: async () => ({
          ...mockAssignResponse,
          project_context: minimalAssignment
        })
      } as Response);

      const result = await client.assignProductToProject('abc123', minimalAssignment);

      expect(result.project_context.project_number).toBe('25-97101');
      expect(result.project_context.project_name).toBe('Test Project');
    });

    it('should handle duplicate assignment conflict', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 409,
        json: async () => ({
          error: 'Conflict',
          message: 'Product abc123 is already assigned to project 25-97101'
        })
      } as Response);

      await expect(
        client.assignProductToProject('abc123', {
          project_number: '25-97101',
          project_name: 'Test Project'
        })
      ).rejects.toThrow('Product abc123 is already assigned to project 25-97101');
    });

    it('should validate required fields', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 400,
        json: async () => ({
          error: 'Validation Error',
          message: 'project_number is required'
        })
      } as Response);

      await expect(
        client.assignProductToProject('abc123', { project_number: '', project_name: 'Test' })
      ).rejects.toThrow('project_number is required');
    });

    it('should handle product not found', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        json: async () => ({
          error: 'Not Found',
          message: 'Product abc123 not found'
        })
      } as Response);

      await expect(
        client.assignProductToProject('abc123', {
          project_number: '25-97101',
          project_name: 'Test Project'
        })
      ).rejects.toThrow('Product abc123 not found');
    });
  });

  describe('Error Handling', () => {
    it('should handle network timeouts', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Request timeout'));

      await expect(client.health()).rejects.toThrow('Request timeout');
    });

    it('should handle HTTP 503 Service Unavailable', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 503,
        json: async () => ({
          error: 'Service Unavailable',
          message: 'SurrealDB not connected'
        })
      } as Response);

      await expect(client.searchProducts({})).rejects.toThrow('SurrealDB not connected');
    });

    it('should handle malformed JSON responses', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        json: async () => {
          throw new Error('Invalid JSON');
        }
      } as unknown as Response);

      await expect(client.health()).rejects.toThrow('HTTP 500');
    });

    it('should handle missing error message in response', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 500,
        json: async () => ({})
      } as Response);

      await expect(client.health()).rejects.toThrow('HTTP 500');
    });

    it('should use error field when message is missing', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 400,
        json: async () => ({ error: 'Bad Request' })
      } as Response);

      await expect(client.health()).rejects.toThrow('Bad Request');
    });
  });

  describe('Query String Building', () => {
    it('should filter out undefined and null values', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockSearchResponse
      } as Response);

      await client.searchProducts({
        q: 'test',
        manufacturer: undefined,
        limit: 20,
        offset: undefined
      });

      const calledUrl = (mockFetch.mock.calls[0][0] as string);
      expect(calledUrl).toContain('q=test');
      expect(calledUrl).toContain('limit=20');
      expect(calledUrl).not.toContain('manufacturer');
      expect(calledUrl).not.toContain('offset');
    });

    it('should handle empty query strings', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockSearchResponse
      } as Response);

      await client.searchProducts({});

      const calledUrl = (mockFetch.mock.calls[0][0] as string);
      expect(calledUrl).toBe('http://10.0.21.11:3000/api/products/search');
    });

    it('should properly encode special characters', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockSearchResponse
      } as Response);

      await client.searchProducts({
        q: 'LED 4" Round',
        manufacturer: 'WAC & Co.'
      });

      const calledUrl = (mockFetch.mock.calls[0][0] as string);
      expect(calledUrl).toContain('q=LED+4%22+Round');
      expect(calledUrl).toContain('manufacturer=WAC+%26+Co.');
    });
  });

  describe('Concurrent Operations', () => {
    it('should handle multiple concurrent requests', async () => {
      mockFetch
        .mockResolvedValueOnce({ ok: true, json: async () => mockHealthResponse } as Response)
        .mockResolvedValueOnce({ ok: true, json: async () => mockSearchResponse } as Response)
        .mockResolvedValueOnce({ ok: true, json: async () => mockProductDetail } as Response);

      const results = await Promise.all([
        client.health(),
        client.searchProducts({ q: 'test' }),
        client.getProduct('abc123')
      ]);

      expect(results[0]).toEqual(mockHealthResponse);
      expect(results[1]).toEqual(mockSearchResponse);
      expect(results[2]).toEqual(mockProductDetail);
      expect(mockFetch).toHaveBeenCalledTimes(3);
    });

    it('should handle partial failures in concurrent operations', async () => {
      mockFetch
        .mockResolvedValueOnce({ ok: true, json: async () => mockHealthResponse } as Response)
        .mockResolvedValueOnce({
          ok: false,
          status: 500,
          json: async () => ({ error: 'Internal Error' })
        } as Response)
        .mockResolvedValueOnce({ ok: true, json: async () => mockProductDetail } as Response);

      await expect(
        Promise.all([
          client.health(),
          client.searchProducts({ q: 'test' }),
          client.getProduct('abc123')
        ])
      ).rejects.toThrow();
    });
  });
});
