/**
 * Win Ratio Page - render, error visibility, and sort behaviour.
 *
 * Follows the same error-visibility pattern established by
 * Companies.error.test.ts / Proposals.error.test.ts (2026-08-13): a failed
 * load must show the real error and a Retry button, never render the
 * generic empty state as if the account genuinely had no data.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent, within } from '@testing-library/svelte';
import { invoke } from '@tauri-apps/api/core';
import WinRatioPage from './WinRatioPage.svelte';
import type { WinRatioReport } from '../types';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

const twoClientReport: WinRatioReport = {
  clients: [
    {
      company_id: 'company:PTG',
      company_name: 'Petrus Group',
      won_count: 6,
      lost_count: 0,
      no_response_count: 3,
      cancelled_count: 0,
      pending_count: 1,
      decided_count: 9,
      win_ratio: 6 / 9,
      won_value: [{ currency: 'AED', amount: 300000 }],
      lost_value: [{ currency: 'AED', amount: 90000 }]
    },
    {
      company_id: 'company:HYP',
      company_name: 'Hyperion',
      won_count: 1,
      lost_count: 0,
      no_response_count: 4,
      cancelled_count: 0,
      pending_count: 0,
      decided_count: 5,
      win_ratio: 1 / 5,
      won_value: [{ currency: 'USD', amount: 12000 }],
      lost_value: [{ currency: 'USD', amount: 48000 }]
    }
  ],
  unattributed_decided_count: 0
};

describe('WinRatioPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('shows the real load error instead of a silent empty table', async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error('No database client available'));

    render(WinRatioPage);

    await waitFor(() => {
      expect(screen.getByText(/Failed to load the win-ratio report/)).toBeInTheDocument();
    });
    expect(screen.getByText('Retry')).toBeInTheDocument();
    expect(screen.queryByText('Petrus Group')).not.toBeInTheDocument();
  });

  it('retries the load when Retry is clicked, clearing the error on success', async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error('No database client available'));

    render(WinRatioPage);

    await waitFor(() => {
      expect(screen.getByText('Retry')).toBeInTheDocument();
    });

    vi.mocked(invoke).mockResolvedValueOnce(twoClientReport);
    await fireEvent.click(screen.getByText('Retry'));

    await waitFor(() => {
      expect(screen.getByText('Petrus Group')).toBeInTheDocument();
    });
    expect(screen.queryByText(/Failed to load the win-ratio report/)).not.toBeInTheDocument();
  });

  it('shows the genuine empty state (not an error) when there are no decided proposals yet', async () => {
    vi.mocked(invoke).mockResolvedValueOnce({ clients: [], unattributed_decided_count: 0 });

    render(WinRatioPage);

    await waitFor(() => {
      expect(screen.getByText('No decided proposals yet')).toBeInTheDocument();
    });
    expect(screen.queryByText('Retry')).not.toBeInTheDocument();
  });

  it('renders per-client rows with win ratio, counts, and per-currency values', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(twoClientReport);

    render(WinRatioPage);

    await waitFor(() => {
      expect(screen.getByText('Petrus Group')).toBeInTheDocument();
    });

    const ptgRow = screen.getByText('Petrus Group').closest('tr');
    expect(ptgRow).not.toBeNull();
    const cells = within(ptgRow as HTMLElement).getAllByRole('cell');
    // Client, Won, Lost, No Response, Cancelled, Pending, Decided, Win Ratio, Won Value, Lost Value
    expect(cells.map(c => c.textContent?.trim())).toEqual([
      'Petrus Group',
      '6',
      '0',
      '3',
      '0',
      '1',
      '9',
      '67%',
      'AED 300,000',
      'AED 90,000'
    ]);

    // Overall summary: 7 won of 14 decided across both clients.
    expect(screen.getByText('50%')).toBeInTheDocument();
    expect(screen.getByText('7 won of 14 decided')).toBeInTheDocument();
  });

  it('surfaces decided-but-unattributed projects instead of silently dropping them', async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      clients: [twoClientReport.clients[0]],
      unattributed_decided_count: 2
    });

    render(WinRatioPage);

    await waitFor(() => {
      expect(screen.getByText(/2 decided projects not linked to a fee/)).toBeInTheDocument();
    });
  });

  it('re-sorts the table when a sortable column header is clicked', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(twoClientReport);

    render(WinRatioPage);

    await waitFor(() => {
      expect(screen.getByText('Petrus Group')).toBeInTheDocument();
    });

    // Default sort is decided_count desc: Petrus Group (9) before Hyperion (5).
    let rows = screen.getAllByRole('row').slice(1); // drop header row
    expect(within(rows[0]).getByText('Petrus Group')).toBeInTheDocument();
    expect(within(rows[1]).getByText('Hyperion')).toBeInTheDocument();

    // Click "Client" to sort alphabetically ascending: Hyperion before Petrus Group.
    await fireEvent.click(screen.getByRole('button', { name: /^Client/ }));

    rows = screen.getAllByRole('row').slice(1);
    expect(within(rows[0]).getByText('Hyperion')).toBeInTheDocument();
    expect(within(rows[1]).getByText('Petrus Group')).toBeInTheDocument();
  });
});
