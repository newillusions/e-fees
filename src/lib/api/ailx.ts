/**
 * AILX API Client Module
 *
 * HTTP client for the AILX API server (AI Lighting eXpert).
 * Provides access to lighting product data, recommendations, and project assignment.
 *
 * Usage:
 *   const client = new AilxApiClient('http://10.0.21.11:3000');
 *   const health = await client.health();
 *   const products = await client.searchProducts({ q: 'downlight', manufacturer: 'WAC' });
 *   const fixture = await client.getProduct('abc123');
 */

// ============================================================================
// RESPONSE TYPES
// ============================================================================

export interface AilxHealthResponse {
  status: 'healthy' | 'degraded' | 'unhealthy';
  version: string;
  uptime: number;
  services: {
    surrealdb: ServiceStatus;
    elasticsearch: ServiceStatus;
  };
}

export interface ServiceStatus {
  status: 'connected' | 'disconnected' | 'error';
  latency_ms?: number;
  error?: string;
}

export interface ProductSummary {
  id: string;
  model: string;
  part_number: string;
  product_type: string;
  manufacturer_name: string;
  wattage?: number | null;
  lumens?: number | null;
  cct_kelvin?: number | null;
  mounting_type?: string | null;
}

export interface ProductSearchResponse {
  total: number;
  limit: number;
  offset: number;
  results: ProductSummary[];
}

export interface ProductRecommendResponse {
  data: ProductSummary[];
  total: number;
  filters_applied: string[];
}

export interface ProductDetailResponse {
  id: string;
  model: string;
  series?: string | null;
  part_number: string;
  catalog_number?: string | null;
  product_type: string;
  description?: string | null;
  manufacturer: {
    id: string;
    name: string;
    brand?: string | null;
    country?: string | null;
    website?: string | null;
  };
  light_source?: Record<string, unknown> | null;
  physical?: Record<string, unknown> | null;
  electrical?: Record<string, unknown> | null;
  driver?: Record<string, unknown> | null;
  optical?: Record<string, unknown> | null;
  environmental?: Record<string, unknown> | null;
  accessories?: Record<string, unknown>[] | null;
  documentation?: Record<string, unknown> | null;
  taxonomy?: Record<string, unknown> | null;
}

export interface AssignProjectRequest {
  project_number: string;
  project_name: string;
  role?: 'specified' | 'alternate' | 'basis_of_design';
  space_name?: string;
  quantity?: number;
  specification_section?: string;
  status?: 'Proposed' | 'Approved' | 'Substitution_Requested' | 'Installed';
}

export interface AssignProjectResponse {
  product_id: string;
  project_fixture_id: string;
  project_context: AssignProjectRequest & { status: string };
  status: 'linked';
}

export interface DuplicateCheckResponse {
  exists: boolean;
  product?: {
    id: string;
    model: string;
    part_number: string;
    manufacturer_name: string;
  } | null;
}

// ============================================================================
// REQUEST PARAMETER TYPES
// ============================================================================

export interface ProductSearchParams {
  q?: string;
  manufacturer?: string;
  product_type?: string;
  min_wattage?: number;
  max_wattage?: number;
  mounting_type?: string;
  control_protocol?: string;
  limit?: number;
  offset?: number;
}

export interface ProductRecommendParams {
  mounting_type?: string;
  min_wattage?: number;
  max_wattage?: number;
  min_cri?: number;
  product_type?: string;
  manufacturer?: string;
  limit?: number;
  offset?: number;
}

export interface DuplicateCheckParams {
  part_number: string;
  manufacturer_name?: string;
}

// ============================================================================
// CLIENT
// ============================================================================

export class AilxApiClient {
  public readonly baseUrl: string;

  constructor(baseUrl: string = 'http://10.0.21.11:3000') {
    this.baseUrl = baseUrl.replace(/\/$/, '');
  }

  private async request<T>(path: string, options: RequestInit = {}): Promise<T> {
    const url = `${this.baseUrl}${path}`;
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers
      }
    });

    if (!response.ok) {
      let errorMessage: string;
      try {
        const errorBody = await response.json();
        errorMessage = errorBody.message || errorBody.error || `HTTP ${response.status}`;
      } catch {
        errorMessage = `HTTP ${response.status}`;
      }
      throw new Error(errorMessage);
    }

    return response.json();
  }

  private buildQueryString(params: Record<string, unknown>): string {
    const filtered = Object.entries(params)
      .filter(([_, v]) => v !== undefined && v !== null)
      .map(([k, v]) => [k, String(v)]);

    if (filtered.length === 0) return '';
    return '?' + new URLSearchParams(filtered).toString();
  }

  // ============================================================================
  // HEALTH & STATUS
  // ============================================================================

  async health(): Promise<AilxHealthResponse> {
    return this.request('/api/health');
  }

  // ============================================================================
  // PRODUCT SEARCH & QUERY
  // ============================================================================

  async searchProducts(params: ProductSearchParams = {}): Promise<ProductSearchResponse> {
    const queryString = this.buildQueryString(params as Record<string, unknown>);
    return this.request(`/api/products/search${queryString}`);
  }

  async recommendProducts(params: ProductRecommendParams = {}): Promise<ProductRecommendResponse> {
    const queryString = this.buildQueryString(params as Record<string, unknown>);
    return this.request(`/api/products/recommend${queryString}`);
  }

  async getProduct(id: string): Promise<ProductDetailResponse> {
    return this.request(`/api/products/${encodeURIComponent(id)}`);
  }

  // ============================================================================
  // DUPLICATE CHECKING
  // ============================================================================

  async checkDuplicate(params: DuplicateCheckParams): Promise<DuplicateCheckResponse> {
    const queryString = this.buildQueryString(params as unknown as Record<string, unknown>);
    return this.request(`/api/fixtures/check-duplicate${queryString}`);
  }

  // ============================================================================
  // PROJECT ASSIGNMENT
  // ============================================================================

  async assignProductToProject(
    productId: string,
    assignment: AssignProjectRequest
  ): Promise<AssignProjectResponse> {
    return this.request(`/api/fixtures/${encodeURIComponent(productId)}/assign-project`, {
      method: 'POST',
      body: JSON.stringify(assignment)
    });
  }
}
