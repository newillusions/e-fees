/**
 * BulkActionBar Component Tests
 *
 * Covers the confirm-before-destroy delete flow, the status-select-then-apply
 * flow, singular/plural entity labeling, and confirmation reset on selection
 * change. This component had zero test coverage before this file.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import BulkActionBar from './BulkActionBar.svelte';

describe('BulkActionBar', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders nothing when selectedCount is 0', () => {
    const { container } = render(BulkActionBar, {
      props: { selectedCount: 0, entityType: 'projects' }
    });
    expect(container.querySelector('.emittiv-bulk-bar')).toBeNull();
  });

  it('shows the selected count and entity type', () => {
    render(BulkActionBar, { props: { selectedCount: 3, entityType: 'projects' } });
    expect(screen.getByText(/3/)).toBeInTheDocument();
    expect(screen.getByText(/projects selected/)).toBeInTheDocument();
  });

  describe('delete confirmation', () => {
    it('requires a second click before calling ondelete (plural label first)', async () => {
      const ondelete = vi.fn();
      render(BulkActionBar, {
        props: { selectedCount: 2, entityType: 'projects', ondelete }
      });

      const deleteButton = screen.getByRole('button', { name: 'Delete' });
      await fireEvent.click(deleteButton);

      expect(ondelete).not.toHaveBeenCalled();
      expect(screen.getByRole('button', { name: 'Delete 2 projects' })).toBeInTheDocument();
    });

    it('uses the singular entity label when exactly one item is selected', async () => {
      render(BulkActionBar, { props: { selectedCount: 1, entityType: 'projects' } });

      const deleteButton = screen.getByRole('button', { name: 'Delete' });
      await fireEvent.click(deleteButton);

      expect(screen.getByRole('button', { name: 'Delete 1 project' })).toBeInTheDocument();
    });

    it('calls ondelete and resets confirmation on the second click', async () => {
      const ondelete = vi.fn();
      render(BulkActionBar, {
        props: { selectedCount: 2, entityType: 'projects', ondelete }
      });

      await fireEvent.click(screen.getByRole('button', { name: 'Delete' }));
      await fireEvent.click(screen.getByRole('button', { name: 'Delete 2 projects' }));

      expect(ondelete).toHaveBeenCalledTimes(1);
      expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument();
    });
  });

  describe('status change flow', () => {
    it('does not show Apply until a status is chosen', () => {
      render(BulkActionBar, {
        props: { selectedCount: 2, entityType: 'projects', statuses: ['Lead', 'Lost'] }
      });
      expect(screen.queryByRole('button', { name: 'Apply' })).toBeNull();
    });

    it('calls onstatuschange with the selected value and resets the select', async () => {
      const onstatuschange = vi.fn();
      render(BulkActionBar, {
        props: {
          selectedCount: 2,
          entityType: 'projects',
          statuses: ['Lead', 'Lost'],
          onstatuschange
        }
      });

      const select = screen.getByRole('combobox') as HTMLSelectElement;
      await fireEvent.change(select, { target: { value: 'Lost' } });

      const applyButton = screen.getByRole('button', { name: 'Apply' });
      await fireEvent.click(applyButton);

      expect(onstatuschange).toHaveBeenCalledWith('Lost');
      expect(screen.queryByRole('button', { name: 'Apply' })).toBeNull();
    });

    it('does not render the status select when no statuses are provided', () => {
      render(BulkActionBar, { props: { selectedCount: 2, entityType: 'projects' } });
      expect(screen.queryByRole('combobox')).toBeNull();
    });
  });

  describe('clear', () => {
    it('calls onclear and resets any pending confirmation state', async () => {
      const onclear = vi.fn();
      render(BulkActionBar, {
        props: { selectedCount: 2, entityType: 'projects', onclear }
      });

      await fireEvent.click(screen.getByRole('button', { name: 'Delete' }));
      await fireEvent.click(screen.getByRole('button', { name: 'Clear' }));

      expect(onclear).toHaveBeenCalledTimes(1);
      // Confirmation state reset - a subsequent Delete click starts fresh
      expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument();
    });
  });

  it('resets pending confirmation state when selectedCount drops to 0 externally', async () => {
    const { rerender } = render(BulkActionBar, {
      props: { selectedCount: 2, entityType: 'projects' }
    });

    await fireEvent.click(screen.getByRole('button', { name: 'Delete' }));
    expect(screen.getByRole('button', { name: 'Delete 2 projects' })).toBeInTheDocument();

    await rerender({ selectedCount: 0, entityType: 'projects' });
    // Bar disappears entirely - nothing left to assert a stale confirm state on,
    // but re-selecting should start from a clean slate.
    await rerender({ selectedCount: 1, entityType: 'projects' });
    expect(screen.getByRole('button', { name: 'Delete' })).toBeInTheDocument();
  });
});
