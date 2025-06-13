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
  isConnected: boolean;
  status: string;
  lastChecked?: Date;
}

export interface Project {
  id: string;
  name: string;
  description: string;
  status: 'active' | 'completed' | 'on-hold';
  createdAt: Date;
  updatedAt: Date;
}

export interface Proposal {
  id: string;
  title: string;
  client: string;
  amount: number;
  status: 'draft' | 'sent' | 'approved' | 'rejected';
  projectId?: string;
  createdAt: Date;
  dueDate?: Date;
}

export interface Company {
  id: string;
  name: string;
  industry: string;
  contactCount: number;
  address?: string;
  website?: string;
  createdAt: Date;
}

export interface Contact {
  id: string;
  name: string;
  email: string;
  phone?: string;
  position?: string;
  companyId: string;
  company: string;
  createdAt: Date;
}