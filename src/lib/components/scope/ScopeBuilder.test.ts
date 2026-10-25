/**
 * ScopeBuilder save-state honesty (Codex review R4, 2026-10-24).
 *
 * handleSave snapshots the deliverables, awaits the API, then used to show
 * "Scope saved successfully" even when the user kept editing during the
 * await (the later edits were never sent). It also routed a failed save into
 * the page-level error, which replaced the editor and lost every edit.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor, within } from '@testing-library/svelte';
import ScopeBuilder from './ScopeBuilder.svelte';

const api = vi.hoisted(() => ({
  getStages: vi.fn(),
  getDeliverables: vi.fn(),
  assembleDeliverables: vi.fn(),
  getScopeDeliverables: vi.fn(),
  saveScopeBuilder: vi.fn()
}));

vi.mock('$lib/api/scope', () => api);
vi.mock('$lib/services/logger', () => ({ logApiError: vi.fn() }));

function deliverable(id: string) {
  return {
    id,
    title: `Title ${id}`,
    short_name: `Short ${id}`,
    body: `Library body ${id}`,
    stage: 'Concept',
    layer: 'standard',
    sort_order: 1,
    status: 'active',
    version: 1,
    created_at: '',
    updated_at: ''
  };
}

const nothingSaved = {
  fee_id: 'fee1',
  deliverables_used: [],
  stage_labels: null,
  manual_items: []
};

function savedState(
  entries: Array<{ id: string; wording?: string }>,
  stageLabels: Record<string, string> = { Concept: 'Concept' }
) {
  return {
    fee_id: 'fee1',
    deliverables_used: entries.map((e, i) => ({
      deliverable_id: e.id,
      stage: 'Concept',
      short_name: `Short ${e.id}`,
      sort_order: i + 1,
      wording_snapshot: e.wording ?? `Library body ${e.id}`
    })),
    stage_labels: stageLabels,
    manual_items: []
  };
}

function deferred<T>() {
  let resolve!: (v: T) => void;
  let reject!: (e: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

async function renderLoaded() {
  render(ScopeBuilder, { params: { id: 'fee1' } });
  await screen.findAllByTitle('Remove deliverable');
}

describe('ScopeBuilder save state', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    api.getStages.mockResolvedValue({
      data: [
        {
          canonical_name: 'Concept',
          default_label: 'Concept',
          sort_order: 1,
          status: 'active'
        }
      ],
      total: 1
    });
    const items = [deliverable('d1'), deliverable('d2')];
    api.getDeliverables.mockResolvedValue({ data: items, total: 2 });
    api.getScopeDeliverables.mockResolvedValue(nothingSaved);
    api.assembleDeliverables.mockResolvedValue({
      stages: [{ canonical_name: 'Concept', label: 'Concept', deliverables: items }]
    });
  });

  it('does not claim "saved" when the user edited while the save was in flight', async () => {
    const save = deferred<unknown>();
    api.saveScopeBuilder.mockReturnValueOnce(save.promise);
    await renderLoaded();

    await fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await waitFor(() => expect(api.saveScopeBuilder).toHaveBeenCalledTimes(1));

    // Edit after the save request was built.
    await fireEvent.click(screen.getAllByTitle('Remove deliverable')[0]);

    save.resolve({});
    await waitFor(() => expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled());

    expect(screen.queryByText('Scope saved successfully')).toBeNull();
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
  });

  it('shows "saved" and no unsaved marker when nothing changed during the save', async () => {
    api.saveScopeBuilder.mockResolvedValueOnce({});
    await renderLoaded();

    await fireEvent.click(screen.getByRole('button', { name: 'Save' }));

    expect(await screen.findByText('Scope saved successfully')).toBeInTheDocument();
    expect(screen.queryByText('Unsaved changes')).toBeNull();
  });

  it('marks edits as unsaved before any save', async () => {
    await renderLoaded();
    expect(screen.queryByText('Unsaved changes')).toBeNull();

    await fireEvent.click(screen.getAllByTitle('Remove deliverable')[0]);

    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
  });

  it('keeps the editor and the edits when a save fails', async () => {
    api.saveScopeBuilder.mockRejectedValueOnce(new Error('scope service down'));
    await renderLoaded();
    await fireEvent.click(screen.getAllByTitle('Remove deliverable')[0]);
    const before = screen.getAllByTitle('Remove deliverable').length;

    await fireEvent.click(screen.getByRole('button', { name: 'Save' }));

    expect(await screen.findByText(/scope service down/)).toBeInTheDocument();
    expect(screen.getAllByTitle('Remove deliverable')).toHaveLength(before);
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
  });
  async function editFirstDeliverableInModal(text: string) {
    const card = screen.getAllByTitle('Remove deliverable')[0].closest('[role="listitem"]')!;
    await fireEvent.dblClick(card);
    const dialog = await screen.findByRole('dialog');
    await fireEvent.click(within(dialog).getByRole('button', { name: 'Edit' }));
    const box = within(dialog).getByRole('textbox') as HTMLTextAreaElement;
    await fireEvent.input(box, { target: { value: text } });
    await fireEvent.click(within(dialog).getByRole('button', { name: 'Save' }));
  }

  it('sends detail-modal wording edits as wording_override and only then reports saved', async () => {
    api.saveScopeBuilder.mockResolvedValueOnce({});
    await renderLoaded();

    await editFirstDeliverableInModal('Edited wording');
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();

    await fireEvent.click(screen.getByRole('button', { name: 'Save' }));

    expect(await screen.findByText('Scope saved successfully')).toBeInTheDocument();
    const sent = api.saveScopeBuilder.mock.calls[0][0];
    expect(sent.deliverables).toHaveLength(2);
    expect(sent.deliverables[0]).toMatchObject({
      deliverable_id: 'd1',
      wording_override: 'Edited wording'
    });
    expect(sent.deliverables[1]).toMatchObject({ deliverable_id: 'd2', wording_override: null });
    expect(screen.queryByText('Unsaved changes')).toBeNull();
  });

  it('stays dirty when a modal edit is not confirmed saved', async () => {
    api.saveScopeBuilder.mockRejectedValueOnce(new Error('scope service down'));
    await renderLoaded();
    await editFirstDeliverableInModal('Edited wording');

    await fireEvent.click(screen.getByRole('button', { name: 'Save' }));

    expect(await screen.findByText(/scope service down/)).toBeInTheDocument();
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
    // The edit is still pending for the next attempt.
    api.saveScopeBuilder.mockResolvedValueOnce({});
    await fireEvent.click(screen.getByRole('button', { name: 'Save' }));
    await screen.findByText('Scope saved successfully');
    expect(api.saveScopeBuilder.mock.calls[1][0].deliverables[0].wording_override).toBe(
      'Edited wording'
    );
  });

  it('asks before Reset discards unsaved edits, and keeps them when declined', async () => {
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false);
    await renderLoaded();
    await fireEvent.click(screen.getAllByTitle('Remove deliverable')[0]);
    const remaining = screen.getAllByTitle('Remove deliverable').length;
    const assembleCalls = api.assembleDeliverables.mock.calls.length;

    await fireEvent.click(screen.getByRole('button', { name: 'Reset' }));

    expect(confirmSpy).toHaveBeenCalledTimes(1);
    expect(api.assembleDeliverables).toHaveBeenCalledTimes(assembleCalls);
    expect(screen.getAllByTitle('Remove deliverable')).toHaveLength(remaining);
    expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
    confirmSpy.mockRestore();
  });

  it('Reset reloads and clears the unsaved marker when the user confirms', async () => {
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);
    await renderLoaded();
    const before = screen.getAllByTitle('Remove deliverable').length;
    await fireEvent.click(screen.getAllByTitle('Remove deliverable')[0]);
    const assembleCalls = api.assembleDeliverables.mock.calls.length;

    await fireEvent.click(screen.getByRole('button', { name: 'Reset' }));

    await waitFor(() => expect(api.assembleDeliverables).toHaveBeenCalledTimes(assembleCalls + 1));
    await waitFor(() => expect(screen.getAllByTitle('Remove deliverable')).toHaveLength(before));
    expect(screen.queryByText('Unsaved changes')).toBeNull();
    confirmSpy.mockRestore();
  });

  it('Reset on a clean builder does not prompt', async () => {
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);
    await renderLoaded();

    await fireEvent.click(screen.getByRole('button', { name: 'Reset' }));

    expect(confirmSpy).not.toHaveBeenCalled();
    confirmSpy.mockRestore();
  });
  describe('load and reset use the saved scope', () => {
    it('shows library defaults when nothing has been saved', async () => {
      await renderLoaded();

      expect(api.getScopeDeliverables).toHaveBeenCalledWith('fee1');
      expect(api.assembleDeliverables).toHaveBeenCalledTimes(1);
      expect(screen.getAllByTitle('Remove deliverable')).toHaveLength(2);
      expect(screen.queryByText('Unsaved changes')).toBeNull();
    });

    it('shows the saved scope and wording, not library defaults, when one exists', async () => {
      api.getScopeDeliverables.mockResolvedValue(
        savedState([{ id: 'd2', wording: 'Saved wording' }])
      );
      api.saveScopeBuilder.mockResolvedValueOnce({});
      await renderLoaded();

      expect(api.assembleDeliverables).not.toHaveBeenCalled();
      expect(screen.getAllByTitle('Remove deliverable')).toHaveLength(1);
      expect(screen.queryByText('Unsaved changes')).toBeNull();

      // The saved wording is carried into the next save, not reset to the library body.
      await fireEvent.click(screen.getByRole('button', { name: 'Save' }));
      await screen.findByText('Scope saved successfully');
      expect(api.saveScopeBuilder.mock.calls[0][0].deliverables).toEqual([
        { deliverable_id: 'd2', stage: 'Concept', sort_order: 1, wording_override: 'Saved wording' }
      ]);
    });

    it('save then reload shows the saved scope and wording', async () => {
      let stored: unknown = nothingSaved;
      api.saveScopeBuilder.mockImplementation(async req => {
        stored = {
          fee_id: 'fee1',
          deliverables_used: req.deliverables.map((e: any) => ({
            deliverable_id: e.deliverable_id,
            stage: e.stage,
            sort_order: e.sort_order,
            wording_snapshot: e.wording_override ?? `Library body ${e.deliverable_id}`
          })),
          stage_labels: req.stage_labels,
          manual_items: []
        };
        return {};
      });
      api.getScopeDeliverables.mockImplementation(async () => stored);

      const first = render(ScopeBuilder, { params: { id: 'fee1' } });
      await screen.findAllByTitle('Remove deliverable');
      await fireEvent.click(screen.getAllByTitle('Remove deliverable')[1]); // drop d2
      await editFirstDeliverableInModal('Edited wording');
      await fireEvent.click(screen.getByRole('button', { name: 'Save' }));
      await screen.findByText('Scope saved successfully');
      first.unmount();

      api.assembleDeliverables.mockClear();
      render(ScopeBuilder, { params: { id: 'fee1' } });
      await screen.findAllByTitle('Remove deliverable');

      expect(api.assembleDeliverables).not.toHaveBeenCalled();
      expect(screen.getAllByTitle('Remove deliverable')).toHaveLength(1);
      // Re-save without touching it: the edited wording is still what is sent.
      await fireEvent.click(screen.getByRole('button', { name: 'Save' }));
      await waitFor(() => expect(api.saveScopeBuilder).toHaveBeenCalledTimes(2));
      expect(api.saveScopeBuilder.mock.calls[1][0].deliverables).toEqual([
        {
          deliverable_id: 'd1',
          stage: 'Concept',
          sort_order: 1,
          wording_override: 'Edited wording'
        }
      ]);
    });

    it('Reset returns to the last saved state, not to library defaults', async () => {
      api.getScopeDeliverables.mockResolvedValue(savedState([{ id: 'd1' }]));
      const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);
      await renderLoaded();
      expect(screen.getAllByTitle('Remove deliverable')).toHaveLength(1);

      await fireEvent.click(screen.getAllByTitle('Remove deliverable')[0]);
      expect(screen.queryAllByTitle('Remove deliverable')).toHaveLength(0);

      await fireEvent.click(screen.getByRole('button', { name: 'Reset' }));

      await waitFor(() => expect(screen.getAllByTitle('Remove deliverable')).toHaveLength(1));
      expect(api.assembleDeliverables).not.toHaveBeenCalled();
      expect(screen.queryByText('Unsaved changes')).toBeNull();
      confirmSpy.mockRestore();
    });

    it('does not fall back to library defaults when the saved scope cannot be loaded', async () => {
      api.getScopeDeliverables.mockRejectedValue(new Error('scope service down'));
      render(ScopeBuilder, { params: { id: 'fee1' } });

      expect(await screen.findByText(/scope service down/)).toBeInTheDocument();
      expect(api.assembleDeliverables).not.toHaveBeenCalled();
      expect(screen.queryAllByTitle('Remove deliverable')).toHaveLength(0);
    });

    it('Library defaults is a separate action with its own confirm', async () => {
      api.getScopeDeliverables.mockResolvedValue(savedState([{ id: 'd1' }]));
      const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(false);
      await renderLoaded();

      await fireEvent.click(screen.getByRole('button', { name: 'Library defaults' }));
      expect(confirmSpy).toHaveBeenCalledTimes(1);
      expect(api.assembleDeliverables).not.toHaveBeenCalled();
      expect(screen.getAllByTitle('Remove deliverable')).toHaveLength(1);

      confirmSpy.mockReturnValue(true);
      await fireEvent.click(screen.getByRole('button', { name: 'Library defaults' }));
      await waitFor(() => expect(screen.getAllByTitle('Remove deliverable')).toHaveLength(2));
      expect(api.assembleDeliverables).toHaveBeenCalledTimes(1);
      expect(screen.getByText('Unsaved changes')).toBeInTheDocument();
      confirmSpy.mockRestore();
    });
  });
  describe('review round 3', () => {
    it('Save is disabled and sends nothing after a failed load', async () => {
      api.getScopeDeliverables.mockRejectedValue(new Error('scope service down'));
      render(ScopeBuilder, { params: { id: 'fee1' } });
      await screen.findByText(/scope service down/);

      const save = screen.getByRole('button', { name: 'Save' });
      expect(save).toBeDisabled();
      await fireEvent.click(save);
      expect(api.saveScopeBuilder).not.toHaveBeenCalled();
    });

    it('a failed assemble (nothing saved) is an error, not an empty editable scope', async () => {
      api.assembleDeliverables.mockRejectedValue(new Error('assemble down'));
      render(ScopeBuilder, { params: { id: 'fee1' } });

      await screen.findByText(/assemble down/);
      expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled();
    });

    it('shows saved custom stage labels and a save does not overwrite them', async () => {
      api.getScopeDeliverables.mockResolvedValue(
        savedState([{ id: 'd1' }, { id: 'd2' }], { Concept: 'My Concept Label' })
      );
      api.saveScopeBuilder.mockResolvedValueOnce({});
      await renderLoaded();
      expect(screen.getByText('My Concept Label')).toBeInTheDocument();

      await fireEvent.click(screen.getAllByTitle('Remove deliverable')[0]);
      await fireEvent.click(screen.getByRole('button', { name: 'Save' }));
      await screen.findByText('Scope saved successfully');

      const sent = api.saveScopeBuilder.mock.calls[0][0];
      expect('stage_labels' in sent).toBe(false);
    });

    it('keeps a row whose library item was removed, marked, and saves it as a snapshot', async () => {
      api.getScopeDeliverables.mockResolvedValue({
        fee_id: 'fee1',
        deliverables_used: [
          {
            deliverable_id: 'd1',
            stage: 'Concept',
            sort_order: 1,
            wording_snapshot: 'Library body d1'
          },
          {
            deliverable_id: 'gone9',
            stage: 'Concept',
            short_name: 'Old item',
            sort_order: 2,
            wording_snapshot: 'Old wording'
          }
        ],
        stage_labels: null,
        manual_items: []
      });
      api.saveScopeBuilder.mockResolvedValueOnce({});
      await renderLoaded();

      expect(screen.getAllByText('not in active library')).toHaveLength(1);
      expect(screen.getAllByTitle('Remove deliverable')).toHaveLength(2);

      await fireEvent.click(screen.getByRole('button', { name: 'Save' }));
      await screen.findByText('Scope saved successfully');
      expect(api.saveScopeBuilder.mock.calls[0][0].deliverables).toEqual([
        { deliverable_id: 'd1', stage: 'Concept', sort_order: 1, wording_override: null },
        {
          deliverable_id: 'gone9',
          stage: 'Concept',
          sort_order: 2,
          wording_override: 'Old wording',
          short_name: 'Old item'
        }
      ]);
    });

    it('a failed Library defaults shows the error and keeps the editor contents', async () => {
      api.getScopeDeliverables.mockResolvedValue(savedState([{ id: 'd1' }]));
      const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);
      await renderLoaded();
      api.assembleDeliverables.mockRejectedValue(new Error('assemble down'));

      await fireEvent.click(screen.getByRole('button', { name: 'Library defaults' }));

      expect(await screen.findByText(/assemble down/)).toBeInTheDocument();
      expect(screen.getAllByTitle('Remove deliverable')).toHaveLength(1);
      expect(screen.queryByRole('button', { name: 'Retry' })).toBeNull();
      expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled();
      confirmSpy.mockRestore();
    });

    it('a modal save that equals the library body is not an override', async () => {
      api.saveScopeBuilder.mockResolvedValueOnce({});
      await renderLoaded();

      await editFirstDeliverableInModal('Changed');
      await editFirstDeliverableInModal('Library body d1');
      await fireEvent.click(screen.getByRole('button', { name: 'Save' }));
      await screen.findByText('Scope saved successfully');

      expect(api.saveScopeBuilder.mock.calls[0][0].deliverables[0].wording_override).toBeNull();
    });
  });
  describe('orphan rows, review round 4', () => {
    function savedWithOrphan(wording: string | undefined) {
      return {
        fee_id: 'fee1',
        deliverables_used: [
          {
            deliverable_id: 'd1',
            stage: 'Concept',
            sort_order: 1,
            wording_snapshot: 'Library body d1'
          },
          {
            deliverable_id: 'gone9',
            stage: 'Concept',
            short_name: 'Old item',
            sort_order: 2,
            ...(wording === undefined ? {} : { wording_snapshot: wording })
          }
        ],
        stage_labels: null,
        manual_items: []
      };
    }

    it('shows a neutral "not in active library" marker (removed or deprecated)', async () => {
      api.getScopeDeliverables.mockResolvedValue(savedWithOrphan('Old wording'));
      await renderLoaded();

      expect(screen.getByText('not in active library')).toBeInTheDocument();
      expect(screen.queryByText('library item removed')).toBeNull();
      expect(screen.queryByText('wording missing')).toBeNull();
    });

    it.each([[undefined], [''], ['   ']])(
      'an orphan with no wording (%j) shows "wording missing" and blocks the save',
      async wording => {
        api.getScopeDeliverables.mockResolvedValue(savedWithOrphan(wording));
        await renderLoaded();

        expect(screen.getByText('wording missing')).toBeInTheDocument();

        await fireEvent.click(screen.getByRole('button', { name: 'Save' }));

        expect(api.saveScopeBuilder).not.toHaveBeenCalled();
        expect(screen.getByText(/1 row\(s\) marked "wording missing"/)).toBeInTheDocument();
      }
    );

    it('is sent once the orphan has wording', async () => {
      api.getScopeDeliverables.mockResolvedValue(savedWithOrphan(undefined));
      api.saveScopeBuilder.mockResolvedValueOnce({});
      await renderLoaded();

      // Give the orphan (second card) wording through the detail modal.
      const card = screen.getAllByTitle('Remove deliverable')[1].closest('[role="listitem"]')!;
      await fireEvent.dblClick(card);
      const dialog = await screen.findByRole('dialog');
      await fireEvent.click(within(dialog).getByRole('button', { name: 'Edit' }));
      await fireEvent.input(within(dialog).getByRole('textbox'), {
        target: { value: 'Rewritten wording' }
      });
      await fireEvent.click(within(dialog).getByRole('button', { name: 'Save' }));

      expect(screen.queryByText('wording missing')).toBeNull();
      await fireEvent.click(screen.getByRole('button', { name: 'Save' }));
      await screen.findByText('Scope saved successfully');
      expect(api.saveScopeBuilder.mock.calls[0][0].deliverables[1]).toEqual({
        deliverable_id: 'gone9',
        stage: 'Concept',
        sort_order: 2,
        wording_override: 'Rewritten wording',
        short_name: 'Old item'
      });
    });
  });
});
