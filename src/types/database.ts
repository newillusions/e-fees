// SurrealDB Type Definitions
// Auto-generated from database schema

// Enums
export type ProjectActivity = 
  | 'Design and Consultancy'
  | 'Management'
  | 'Construction'
  | 'Maintenance'
  | 'Research'
  | 'Other';

export type ProjectStatus = 
  | 'Draft'
  | 'RFP'
  | 'Active'
  | 'On Hold'
  | 'Completed'
  | 'Cancelled';

export type ProjectStage = 
  | 'Concept'
  | 'Design Development'
  | 'Documentation'
  | 'Tender'
  | 'Construction'
  | 'Handover';

export type RfpStatus = 
  | 'Draft'
  | 'Prepared'
  | 'Sent'
  | 'Negotiation'
  | 'Awarded'
  | 'Lost'
  | 'Cancelled';

export type RfpStage = 
  | 'Draft'
  | 'Prepared'
  | 'Sent'
  | 'Under Review'
  | 'Clarification'
  | 'Negotiation'
  | 'Awarded'
  | 'Lost';

// Base types
export interface TimeInfo {
  created_at: string; // datetime
  updated_at: string; // datetime
}

export interface ProjectNumber {
  year: number; // 20-50
  country: number; // dial code
  seq: number; // 1-999
  id: string; // computed: YY-CCCNN
}

export interface Revision {
  revision_number: number;
  revision_date: string; // datetime
  author_email: string; // valid email
  author_name: string;
  notes: string;
}

// Main entities
export interface Project {
  id?: string; // projects:YY_CCCNN
  name: string;
  name_short: string;
  status: ProjectStatus;
  area: string;
  city: string;
  country: string;
  folder: string;
  number: ProjectNumber;
  time?: TimeInfo;
}

export interface Rfp {
  id?: string; // rfp:YY_CCCNN_R
  name: string;
  number: string;
  project_id: string; // record<projects>
  company_id: string; // record<company>
  contact_id: string; // record<contacts>
  status: RfpStatus;
  stage: RfpStage;
  issue_date: string; // YYMMDD format
  activity?: string;
  package?: string;
  strap_line?: string;
  staff_name?: string;
  staff_email?: string;
  staff_phone?: string;
  staff_position?: string;
  rev?: number; // auto-computed
  revisions: Revision[];
  time?: TimeInfo;
}

export interface Company {
  id?: string; // company:ABBREVIATION
  name: string;
  name_short: string;
  abbreviation: string;
  city: string;
  country: string;
  reg_no?: string;
  tax_no?: string;
  time?: TimeInfo;
}

export interface Contact {
  id?: string; // contacts:random_id
  first_name: string;
  last_name: string;
  email: string; // unique
  phone: string; // must contain '+'
  position: string;
  company: string; // record<company>
  full_name?: string; // auto-computed
  time?: TimeInfo;
}

export interface Country {
  id?: string; // country:CODE
  name: string;
  name_formal: string;
  name_official: string;
  code: string;
  code_alt: string;
  dial_code: number;
  currency_code?: string; // record<currency>
}

export interface Currency {
  id?: string; // currency:CODE
  code: string;
  name: string;
}

// Create/Update types (without computed fields)
export type ProjectCreate = Omit<Project, 'id' | 'time'>;
export type ProjectUpdate = Partial<ProjectCreate>;

export type RfpCreate = Omit<Rfp, 'id' | 'time' | 'rev'>;
export type RfpUpdate = Partial<RfpCreate>;

export type CompanyCreate = Omit<Company, 'id' | 'time'>;
export type CompanyUpdate = Partial<CompanyCreate>;

export type ContactCreate = Omit<Contact, 'id' | 'time' | 'full_name'>;
export type ContactUpdate = Partial<ContactCreate>;

// Validation helpers
export const ValidationRules = {
  project: {
    name: { minLength: 1 },
    number: {
      year: { min: 20, max: 50 },
      seq: { min: 1, max: 999 }
    }
  },
  rfp: {
    name: { minLength: 1 },
    issue_date: { pattern: /^\d{6}$/ } // YYMMDD
  },
  company: {
    name: { minLength: 1 }
  },
  contact: {
    email: { pattern: /^[^\s@]+@[^\s@]+\.[^\s@]+$/ },
    phone: { pattern: /\+/, minLength: 1 }
  }
};

// Helper functions
export function generateProjectId(year: number, country: number, seq: number): string {
  const yy = year.toString().padStart(2, '0');
  const ccc = country.toString();
  const nn = seq.toString().padStart(2, '0');
  return `${yy}-${ccc}${nn}`;
}

export function parseProjectId(id: string): ProjectNumber | null {
  const match = id.match(/^(\d{2})-(\d{3})(\d{2})$/);
  if (!match) return null;
  
  return {
    year: parseInt(match[1]),
    country: parseInt(match[2]),
    seq: parseInt(match[3]),
    id
  };
}

export function formatIssueDate(date: Date): string {
  const yy = date.getFullYear().toString().slice(-2);
  const mm = (date.getMonth() + 1).toString().padStart(2, '0');
  const dd = date.getDate().toString().padStart(2, '0');
  return `${yy}${mm}${dd}`;
}

export function parseIssueDate(dateStr: string): Date | null {
  if (!/^\d{6}$/.test(dateStr)) return null;
  
  const yy = parseInt(dateStr.slice(0, 2));
  const mm = parseInt(dateStr.slice(2, 4)) - 1;
  const dd = parseInt(dateStr.slice(4, 6));
  
  // Assume 20xx for years 00-50, 19xx for years 51-99
  const yyyy = yy <= 50 ? 2000 + yy : 1900 + yy;
  
  return new Date(yyyy, mm, dd);
}
