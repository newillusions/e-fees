/**
 * UpdateNotification Component Tests
 *
 * Regression coverage for the update-dialog version display bug (owner UX
 * report, 2026-08-13): the dialog showed the OFFERED version twice (header
 * "Version X" + body) and never showed the CURRENT running version, so a
 * user had no way to tell what they'd be updating FROM. Fixed by reading
 * `update.currentVersion` (already present on the Tauri updater's `Update`
 * object returned from `check()` - no separate app-version call needed) and
 * rendering "vCURRENT -> vNEW" once, instead of repeating the target version.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/svelte';
import UpdateNotification from './UpdateNotification.svelte';

// jsdom doesn't implement the Web Animations API that Svelte's transition:fade
// / transition:scale rely on internally (same gap documented in
// FirstRunSetup.test.ts, the first component test in the repo to hit it).
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

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: vi.fn()
}));
vi.mock('@tauri-apps/plugin-process', () => ({
  relaunch: vi.fn()
}));
vi.mock('$lib/api/system', () => ({
  getDevMode: vi.fn().mockResolvedValue(false),
  logMessage: vi.fn().mockResolvedValue(undefined)
}));

import { check } from '@tauri-apps/plugin-updater';

describe('UpdateNotification - version display', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('shows the current and target versions once each, not the target version twice', async () => {
    vi.mocked(check).mockResolvedValue({
      version: '0.19.0',
      currentVersion: '0.18.1',
      date: '2026-08-14',
      body: 'Some release notes'
    } as unknown as Awaited<ReturnType<typeof check>>);

    render(UpdateNotification);

    // checkForUpdates is deferred by a 3s setTimeout in onMount; fake timers
    // let the test avoid a real 3s wait. advanceTimersByTimeAsync also
    // flushes the microtask queue so the async checkForUpdates() body (which
    // awaits check()) settles before we assert.
    await vi.advanceTimersByTimeAsync(3000);

    expect(screen.getByText('v0.18.1 → v0.19.0')).toBeInTheDocument();

    // The target version must not appear a second time anywhere else in the
    // dialog chrome (release notes body is exempt - it's arbitrary text).
    expect(screen.queryByText('Version 0.19.0')).not.toBeInTheDocument();
    expect(screen.queryByText(/^0\.19\.0$/)).not.toBeInTheDocument();
  });
});
