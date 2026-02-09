/**
 * Agent API Client Module
 *
 * HTTP client for the E-Fees agent API server. This allows AI agents
 * and external tools to programmatically interact with E-Fees data
 * via REST endpoints served by the embedded HTTP server.
 *
 * Usage:
 *   const client = new AgentApiClient('http://localhost:3100');
 *   const projects = await client.listProjects();
 *   const fee = await client.createFee({ ... });
 */

// ============================================================================
// RESPONSE TYPES
// ============================================================================

export interface AgentHealthResponse {
  status: string;
  version: string;
  db_connected: boolean;
  uptime_seconds: number;
}

export interface AgentStatsResponse {
  total_projects: number;
  total_companies: number;
  total_contacts: number;
  total_fees: number;
  active_fees: number;
}

export interface AgentProjectResponse {
  id: string;
  name: string;
  name_short: string;
  status: string;
  country: string;
  city: string;
  area: string;
  number: string;
  folder: string;
}

export interface AgentFeeResponse {
  id: string;
  name: string;
  number: string;
  rev: number;
  status: string;
  project_id: string;
  company_id: string;
  total_fee: number;
  currency: string;
}

export interface AgentCompanyResponse {
  id: string;
  name: string;
  name_short: string;
  abbreviation: string;
  city: string;
  country: string;
}

export interface AgentListResponse<T> {
  items: T[];
  total: number;
}

export interface AgentContactResponse {
  id: string;
  first_name: string;
  last_name: string;
  full_name: string;
  email: string;
  phone: string;
  position: string;
  company_id: string;
}

export type AgentProjectCreate = Omit<AgentProjectResponse, 'id'>;
export type AgentCompanyCreate = Omit<AgentCompanyResponse, 'id'>;

export interface AgentContactCreate {
  first_name: string;
  last_name: string;
  email: string;
  phone: string;
  position: string;
  company_id: string;
}

export interface AgentExportResponse {
  path: string;
  filename: string;
}

export interface AgentFeeCreate {
  name: string;
  number: string;
  project_id: string;
  company_id: string;
  contact_id: string;
  status?: string;
  issue_date?: string;
  activity?: string;
  package?: string;
  staff_name?: string;
  staff_email?: string;
  staff_phone?: string;
  staff_position?: string;
  strap_line?: string;
}

// ============================================================================
// CLIENT
// ============================================================================

export class AgentApiClient {
  public readonly baseUrl: string;

  constructor(baseUrl: string = 'http://localhost:3100') {
    this.baseUrl = baseUrl.replace(/\/$/, '');
  }

  private async request<T>(path: string, options: RequestInit = {}): Promise<T> {
    const url = `${this.baseUrl}${path}`;
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
    });

    if (!response.ok) {
      let errorMessage: string;
      try {
        const errorBody = await response.json();
        errorMessage = errorBody.error || `HTTP ${response.status}`;
      } catch {
        errorMessage = `HTTP ${response.status}`;
      }
      throw new Error(errorMessage);
    }

    return response.json();
  }

  async health(): Promise<AgentHealthResponse> {
    return this.request('/api/health');
  }

  async stats(): Promise<AgentStatsResponse> {
    return this.request('/api/stats');
  }

  async listProjects(): Promise<AgentListResponse<AgentProjectResponse>> {
    return this.request('/api/projects');
  }

  async getProject(id: string): Promise<AgentProjectResponse> {
    return this.request(`/api/projects/${id}`);
  }

  async createProject(project: AgentProjectCreate): Promise<AgentProjectResponse> {
    return this.request('/api/projects', {
      method: 'POST',
      body: JSON.stringify(project),
    });
  }

  async listFees(projectId?: string): Promise<AgentListResponse<AgentFeeResponse>> {
    const path = projectId ? `/api/fees?project_id=${encodeURIComponent(projectId)}` : '/api/fees';
    return this.request(path);
  }

  async getFee(id: string): Promise<AgentFeeResponse> {
    return this.request(`/api/fees/${id}`);
  }

  async createFee(fee: AgentFeeCreate): Promise<AgentFeeResponse> {
    return this.request('/api/fees', {
      method: 'POST',
      body: JSON.stringify(fee),
    });
  }

  async listCompanies(): Promise<AgentListResponse<AgentCompanyResponse>> {
    return this.request('/api/companies');
  }

  async createCompany(company: AgentCompanyCreate): Promise<AgentCompanyResponse> {
    return this.request('/api/companies', {
      method: 'POST',
      body: JSON.stringify(company),
    });
  }

  async listContacts(companyId?: string): Promise<AgentListResponse<AgentContactResponse>> {
    const path = companyId ? `/api/contacts?company_id=${encodeURIComponent(companyId)}` : '/api/contacts';
    return this.request(path);
  }

  async getContact(id: string): Promise<AgentContactResponse> {
    return this.request(`/api/contacts/${id}`);
  }

  async createContact(contact: AgentContactCreate): Promise<AgentContactResponse> {
    return this.request('/api/contacts', {
      method: 'POST',
      body: JSON.stringify(contact),
    });
  }

  async exportFeeExcel(id: string): Promise<AgentExportResponse> {
    return this.request(`/api/fees/${id}/export`, {
      method: 'POST',
    });
  }
}
