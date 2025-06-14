export interface NavItem {
  id: string;
  label: string;
  icon: string;
  shortcut?: string;
}

export interface AppRoute {
  id: string;
  component: any;
  title: string;
}

export interface ConnectionStatus {
  is_connected: boolean;
  last_check?: string;
  error_message?: string;
}

export interface Project {
  id?: string;
  name: string;
  name_short: string;
  status: 'Draft' | 'RFP' | 'Active' | 'On Hold' | 'Completed' | 'Cancelled';
  area: string;
  city: string;
  country: string;
  folder: string;
  number: {
    year: number;
    country: number;
    seq: number;
    id: string;
  };
  time: {
    created_at: string;
    updated_at: string;
  };
}

export interface Company {
  id?: string;
  name: string;
  name_short: string;
  abbreviation: string;
  city: string;
  country: string;
  reg_no?: string;
  tax_no?: string;
  time: {
    created_at: string;
    updated_at: string;
  };
}

export interface Contact {
  id?: string;
  first_name: string;
  last_name: string;
  full_name: string;
  email: string;
  phone: string;
  position: string;
  company: string;
  time: {
    created_at: string;
    updated_at: string;
  };
}

export interface Rfp {
  id?: string;
  name: string;
  number: string;
  rev: number;
  status: 'Draft' | 'Active' | 'Sent' | 'Awarded' | 'Lost' | 'Cancelled';
  stage: 'Draft' | 'Prepared' | 'Sent' | 'Under Review' | 'Clarification' | 'Negotiation' | 'Awarded' | 'Lost';
  issue_date: string;
  activity: string;
  package: string;
  project_id: string;
  company_id: string;
  contact_id: string;
  staff_name: string;
  staff_email: string;
  staff_phone: string;
  staff_position: string;
  strap_line: string;
  revisions: Revision[];
  time: {
    created_at: string;
    updated_at: string;
  };
}

export interface Revision {
  revision_number: number;
  revision_date: string;
  author_email: string;
  author_name: string;
  notes: string;
}

// Legacy alias for backwards compatibility during transition
export interface Proposal extends Rfp {}