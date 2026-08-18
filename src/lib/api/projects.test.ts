/**
 * Projects API Module Tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  getProjects,
  getProjectsPage,
  getProjectById,
  searchProjects,
  createProject,
  updateProject,
  deleteProject,
  previewProjectMerge,
  mergeProjects,
  previewProjectDelete,
  deleteProjectCascade,
  generateNextProjectNumber,
  validateProjectNumber,
  createProjectWithTemplate,
  copyProjectTemplate,
  populateProjectData
} from './projects';
import type {
  Project,
  ProjectCreate,
  ProjectUpdate,
  PaginatedResponse,
  ProjectMergePreview,
  ProjectMergeResult,
  ProjectDeletePreview,
  ProjectDeleteResult
} from '../../types';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

const mockProject: Project = {
  id: 'projects:test',
  name: 'Test Project',
  name_short: 'Test',
  status: 'Design',
  area: 'Downtown',
  city: 'Dubai',
  country: 'U.A.E.',
  folder: '25-97101 Test',
  number: { year: 25, country: 971, seq: 1, id: '25-97101' },
  time: { created_at: '2025-01-18T00:00:00Z', updated_at: '2025-01-18T00:00:00Z' }
};

describe('Projects API Module', () => {
  const mockInvoke = vi.mocked(invoke);

  beforeEach(() => {
    vi.clearAllMocks();
    vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  describe('getProjects', () => {
    it('should call invoke with correct command', async () => {
      mockInvoke.mockResolvedValueOnce([mockProject]);

      const result = await getProjects();

      expect(mockInvoke).toHaveBeenCalledWith('get_projects');
      expect(result).toEqual([mockProject]);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Database error'));

      await expect(getProjects()).rejects.toThrow('Database error');
    });
  });

  describe('getProjectsPage', () => {
    it('should call invoke with pagination params', async () => {
      const mockResponse: PaginatedResponse<Project> = {
        items: [mockProject],
        total: 1,
        page: 1,
        page_size: 50,
        has_more: false
      };
      mockInvoke.mockResolvedValueOnce(mockResponse);

      const result = await getProjectsPage(1, 50);

      expect(mockInvoke).toHaveBeenCalledWith('get_projects_page', { page: 1, pageSize: 50 });
      expect(result).toEqual(mockResponse);
    });

    it('should use default pagination values', async () => {
      const mockResponse: PaginatedResponse<Project> = {
        items: [],
        total: 0,
        page: 1,
        page_size: 50,
        has_more: false
      };
      mockInvoke.mockResolvedValueOnce(mockResponse);

      await getProjectsPage();

      expect(mockInvoke).toHaveBeenCalledWith('get_projects_page', { page: 1, pageSize: 50 });
    });
  });

  describe('getProjectById', () => {
    it('should call invoke with project id', async () => {
      mockInvoke.mockResolvedValueOnce(mockProject);

      const result = await getProjectById('test');

      expect(mockInvoke).toHaveBeenCalledWith('get_project_by_id', { id: 'test' });
      expect(result).toEqual(mockProject);
    });

    it('should return null for non-existent project', async () => {
      mockInvoke.mockResolvedValueOnce(null);

      const result = await getProjectById('non-existent');

      expect(result).toBeNull();
    });
  });

  describe('searchProjects', () => {
    it('should call invoke with search query', async () => {
      mockInvoke.mockResolvedValueOnce([mockProject]);

      const result = await searchProjects('test');

      expect(mockInvoke).toHaveBeenCalledWith('search_projects', { query: 'test' });
      expect(result).toEqual([mockProject]);
    });
  });

  describe('createProject', () => {
    it('should call invoke with project data and timestamps', async () => {
      mockInvoke.mockResolvedValueOnce(mockProject);

      const projectData: ProjectCreate = {
        name: 'New Project',
        name_short: 'New',
        status: 'Lead',
        area: 'Area',
        city: 'City',
        country: 'Country',
        folder: 'folder'
      };

      const result = await createProject(projectData);

      expect(mockInvoke).toHaveBeenCalledWith(
        'create_project',
        expect.objectContaining({
          project: expect.objectContaining({
            name: 'New Project',
            id: null,
            time: expect.objectContaining({
              created_at: expect.any(String),
              updated_at: expect.any(String)
            })
          })
        })
      );
      expect(result).toEqual(mockProject);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Create failed'));

      await expect(createProject({} as ProjectCreate)).rejects.toThrow('Create failed');
    });
  });

  describe('updateProject', () => {
    it('should call invoke with id and update data', async () => {
      mockInvoke.mockResolvedValueOnce(mockProject);

      const update: ProjectUpdate = { name: 'Updated Name' };
      const result = await updateProject('test', update);

      expect(mockInvoke).toHaveBeenCalledWith('update_project', {
        id: 'test',
        projectUpdate: update
      });
      expect(result).toEqual(mockProject);
    });
  });

  describe('deleteProject', () => {
    it('should call invoke with project id', async () => {
      mockInvoke.mockResolvedValueOnce(mockProject);

      const result = await deleteProject('test');

      expect(mockInvoke).toHaveBeenCalledWith('delete_project', { id: 'test' });
      expect(result).toEqual(mockProject);
    });
  });

  describe('previewProjectMerge', () => {
    it('should call invoke with sourceId and targetId', async () => {
      const mockPreview: ProjectMergePreview = {
        source: mockProject,
        target: { ...mockProject, id: 'projects:target' },
        fees_to_move: 2,
        rev_changes: [{ fee_id: 'fee1', fee_number: 'test-1', old_rev: 1, new_rev: 2 }]
      };
      mockInvoke.mockResolvedValueOnce(mockPreview);

      const result = await previewProjectMerge('source', 'target');

      expect(mockInvoke).toHaveBeenCalledWith('preview_project_merge', {
        sourceId: 'source',
        targetId: 'target'
      });
      expect(result).toEqual(mockPreview);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Cannot merge a project into itself'));

      await expect(previewProjectMerge('a', 'a')).rejects.toThrow(
        'Cannot merge a project into itself'
      );
    });
  });

  describe('mergeProjects', () => {
    it('should call invoke with sourceId and targetId', async () => {
      const mockResult: ProjectMergeResult = {
        target: mockProject,
        fees_moved: 2,
        rev_changes: [],
        source_deleted_id: 'source'
      };
      mockInvoke.mockResolvedValueOnce(mockResult);

      const result = await mergeProjects('source', 'target');

      expect(mockInvoke).toHaveBeenCalledWith('merge_projects', {
        sourceId: 'source',
        targetId: 'target'
      });
      expect(result).toEqual(mockResult);
    });

    it('should throw on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Failed to merge projects'));

      await expect(mergeProjects('source', 'target')).rejects.toThrow(
        'Failed to merge projects'
      );
    });
  });

  describe('previewProjectDelete', () => {
    it('should call invoke with project id', async () => {
      const mockPreview: ProjectDeletePreview = {
        project: mockProject,
        dependent_fees: []
      };
      mockInvoke.mockResolvedValueOnce(mockPreview);

      const result = await previewProjectDelete('test');

      expect(mockInvoke).toHaveBeenCalledWith('preview_project_delete', { id: 'test' });
      expect(result).toEqual(mockPreview);
    });
  });

  describe('deleteProjectCascade', () => {
    it('should call invoke with id and cascade flag', async () => {
      const mockResult: ProjectDeleteResult = {
        deleted_project: mockProject,
        deleted_fees: []
      };
      mockInvoke.mockResolvedValueOnce(mockResult);

      const result = await deleteProjectCascade('test', true);

      expect(mockInvoke).toHaveBeenCalledWith('delete_project_cascade', {
        id: 'test',
        cascade: true
      });
      expect(result).toEqual(mockResult);
    });

    it('should pass cascade=false through unchanged', async () => {
      const mockResult: ProjectDeleteResult = {
        deleted_project: mockProject,
        deleted_fees: []
      };
      mockInvoke.mockResolvedValueOnce(mockResult);

      await deleteProjectCascade('test', false);

      expect(mockInvoke).toHaveBeenCalledWith('delete_project_cascade', {
        id: 'test',
        cascade: false
      });
    });

    it('should throw when the backend refuses an uncascaded delete with dependents', async () => {
      mockInvoke.mockRejectedValueOnce(
        new Error(
          "Project 'test' has 2 fee proposal(s); pass cascade=true to delete them together with the project"
        )
      );

      await expect(deleteProjectCascade('test', false)).rejects.toThrow('cascade=true');
    });
  });

  describe('generateNextProjectNumber', () => {
    it('should call invoke with country name', async () => {
      mockInvoke.mockResolvedValueOnce('25-97102');

      const result = await generateNextProjectNumber('United Arab Emirates');

      expect(mockInvoke).toHaveBeenCalledWith('generate_next_project_number', {
        countryName: 'United Arab Emirates',
        year: null
      });
      expect(result).toBe('25-97102');
    });

    it('should pass year when provided', async () => {
      mockInvoke.mockResolvedValueOnce('24-97101');

      await generateNextProjectNumber('United Arab Emirates', 24);

      expect(mockInvoke).toHaveBeenCalledWith('generate_next_project_number', {
        countryName: 'United Arab Emirates',
        year: 24
      });
    });
  });

  describe('validateProjectNumber', () => {
    it('should call invoke and return boolean', async () => {
      mockInvoke.mockResolvedValueOnce(true);

      const result = await validateProjectNumber('25-97101');

      expect(mockInvoke).toHaveBeenCalledWith('validate_project_number', {
        projectNumber: '25-97101'
      });
      expect(result).toBe(true);
    });
  });

  describe('createProjectWithTemplate', () => {
    it('should call invoke with partial project', async () => {
      mockInvoke.mockResolvedValueOnce(mockProject);

      const result = await createProjectWithTemplate({ name: 'Template Project' });

      expect(mockInvoke).toHaveBeenCalledWith('create_project_with_template', {
        project: { name: 'Template Project' }
      });
      expect(result).toEqual(mockProject);
    });
  });

  describe('copyProjectTemplate', () => {
    it('should call invoke with project number and name', async () => {
      mockInvoke.mockResolvedValueOnce('Success: folder created');

      const result = await copyProjectTemplate('25-97101', 'Test Project');

      expect(mockInvoke).toHaveBeenCalledWith('copy_project_template', {
        projectNumber: '25-97101',
        projectShortName: 'Test Project'
      });
      expect(result).toBe('Success: folder created');
    });
  });

  describe('populateProjectData', () => {
    it('should call invoke with all parameters', async () => {
      mockInvoke.mockResolvedValueOnce('Data populated');

      const result = await populateProjectData('fee:123', '25-97101', 'Test Project');

      expect(mockInvoke).toHaveBeenCalledWith('populate_project_data', {
        fpId: 'fee:123',
        projectNumber: '25-97101',
        projectShortName: 'Test Project'
      });
      expect(result).toBe('Data populated');
    });
  });
});
