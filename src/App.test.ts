/**
 * App Startup Gate Regression Tests
 *
 * Bug (2026-08-13, root cause of the "0 proposals showing" report): the app
 * declared itself "ready" - and mounted Dashboard/Proposals/etc, which
 * immediately query the DB - after a flat 800ms timer that only checked
 * whether settings existed. It never waited for the backend's async DB
 * connect to actually finish (a real network round trip, worse for a
 * root-level user which needs up to 3 sequential signin attempts). A query
 * firing before a client existed failed instantly and left pages silently
 * empty with no visible error.
 *
 * Fix: handleSplashComplete() now awaits waitForConnection(), which polls
 * check_connection_status (via getConnectionStatus) for real readiness with
 * a bounded window, showing a 'connecting' state while waiting and a
 * 'failed' state with the backend's real error + a Retry button if the
 * window runs out - never faking 'ready'.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/svelte';
import App from './App.svelte';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

vi.mock('../tauri-plugin-mcp/dist-js/index.js', () => ({
  setupPluginListeners: vi.fn().mockResolvedValue(undefined),
  cleanupPluginListeners: vi.fn()
}));

vi.mock('$lib/stores/exchangeRates', () => ({
  refreshExchangeRates: vi.fn().mockResolvedValue(undefined)
}));

import { invoke } from '@tauri-apps/api/core';

const VALID_SETTINGS = {
  surrealdb_url: 'ws://10.0.23.11:8000',
  surrealdb_ns: 'emittiv',
  surrealdb_db: 'projects',
  surrealdb_user: 'martin',
  has_password: true
};

// jsdom doesn't implement the Web Animations API that Svelte's transition:fade
// relies on internally (used by App.svelte's route-content transition).
if (typeof Element !== 'undefined' && !Element.prototype.animate) {
  Element.prototype.animate = () =>
    ({
      finished: Promise.resolve(),
      cancel: () => {},
      play: () => {},
      pause: () => {},
      addEventListener: () => {},
      removeEventListener: () => {}
    }) as unknown as Animation;
}

describe('App startup connection gate', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.runOnlyPendingTimers();
    vi.useRealTimers();
  });

  async function passSplash() {
    // SplashScreen self-completes 3.5s after mount (3000ms + 500ms) in the
    // normal path.
    await vi.advanceTimersByTimeAsync(3600);
  }

  it('shows a connecting state, then becomes ready once the backend actually connects (happy path)', async () => {
    let connectCallCount = 0;
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_settings') return VALID_SETTINGS;
      if (cmd === 'get_connection_status') {
        connectCallCount++;
        // Not connected on the first couple of polls (still handshaking),
        // connected from the third poll onward - proves this is a real
        // poll loop, not a one-shot check.
        return {
          is_connected: connectCallCount >= 3,
          last_check: new Date().toISOString()
        };
      }
      return [];
    });

    render(App);

    await passSplash();

    // While waiting, the connecting state must be visible - not a bare
    // Dashboard, and not a fake-ready empty page.
    await waitFor(() => {
      expect(screen.getByText('Connecting to database...')).toBeInTheDocument();
    });

    // Advance through the polling interval enough times to reach the 3rd
    // check (500ms per poll).
    await vi.advanceTimersByTimeAsync(500);
    await vi.advanceTimersByTimeAsync(500);
    await vi.advanceTimersByTimeAsync(500);

    await waitFor(() => {
      expect(screen.queryByText('Connecting to database...')).not.toBeInTheDocument();
    });

    expect(connectCallCount).toBeGreaterThanOrEqual(3);
  });

  it('shows a real error and Retry - never fake-ready - when the DB never becomes reachable within the bounded window', async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_settings') return VALID_SETTINGS;
      if (cmd === 'get_connection_status') {
        return {
          is_connected: false,
          error_message:
            'Connection refused by SurrealDB server at ws://10.0.23.99:8000. Please check if SurrealDB is running.'
        };
      }
      return [];
    });

    render(App);

    await passSplash();

    await waitFor(() => {
      expect(screen.getByText('Connecting to database...')).toBeInTheDocument();
    });

    // Exhaust the 15s bounded polling window.
    await vi.advanceTimersByTimeAsync(15500);

    await waitFor(() => {
      expect(screen.getByText(/Connection refused by SurrealDB server/)).toBeInTheDocument();
    });

    expect(screen.getByText('Retry')).toBeInTheDocument();
    // The critical assertion: it must NOT have silently declared itself
    // ready and rendered the main app while the DB was unreachable.
    expect(screen.queryByText('Connecting to database...')).not.toBeInTheDocument();
  });

  it('retries the connection wait when Retry is clicked after a failure', async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_settings') return VALID_SETTINGS;
      if (cmd === 'get_connection_status') {
        return { is_connected: false, error_message: 'Connection refused' };
      }
      return [];
    });

    render(App);
    await passSplash();
    await vi.advanceTimersByTimeAsync(15500);

    await waitFor(() => {
      expect(screen.getByText('Retry')).toBeInTheDocument();
    });

    // Backend has since recovered.
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_settings') return VALID_SETTINGS;
      if (cmd === 'get_connection_status') {
        return { is_connected: true };
      }
      return [];
    });

    const { fireEvent } = await import('@testing-library/svelte');
    await fireEvent.click(screen.getByText('Retry'));

    await waitFor(() => {
      expect(screen.queryByText('Retry')).not.toBeInTheDocument();
    });
  });
});
