/**
 * FeePricingModal render tests: opening must not loop, edits must persist,
 * and the unsaved-changes guard must prompt only when something changed.
 */
import { describe, it, expect, vi, beforeAll } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/svelte';
import { tick } from 'svelte';
import FeePricingModal from './FeePricingModal.svelte';
import type { Fee } from '../../../types';
import {
  createDefaultDisciplines,
  createDefaultDesignStages,
  DEFAULT_PRICING_CONFIG,
  calculatePricingTotals
} from '../../../types/database';

vi.mock('$lib/api/fees', () => ({
  updateFeePricing: vi.fn()
}));

vi.mock('$lib/services/activityLogger', () => ({
  feeLogger: { onUpdate: vi.fn() }
}));

beforeAll(() => {
  // jsdom has no Web Animations API; Svelte transitions call element.animate on close
  if (!Element.prototype.animate) {
    Element.prototype.animate = function () {
      const animation: Record<string, unknown> = {
        cancel() {},
        finish() {},
        currentTime: 0,
        onfinish: null
      };
      setTimeout(() => (animation.onfinish as (() => void) | null)?.(), 0);
      return animation as unknown as Animation;
    };
  }
});

const fee = { id: 'fee:26_97101_1', number: '26-97101-FP-01', rev: 1 } as unknown as Fee;

function feeWithPricing(): Fee {
  const config = { ...DEFAULT_PRICING_CONFIG, target_fee: 100000, quoted_fee: 100000 };
  const disciplines = createDefaultDisciplines();
  const stages = createDefaultDesignStages();
  const totals = calculatePricingTotals([], [], [], config, stages);
  return {
    ...fee,
    pricing: { config, disciplines, stages, cells: [], costs: [], ...totals }
  } as unknown as Fee;
}

async function settle() {
  // initialisation and the dirty-baseline snapshot both complete after a tick
  await tick();
  await tick();
  await new Promise(r => setTimeout(r, 0));
}

async function openStagesTab() {
  await fireEvent.click(screen.getByRole('button', { name: /^Stages$/ }));
  await settle();
}

describe('FeePricingModal', () => {
  it('opens without an effect loop and shows the pricing tabs', async () => {
    render(FeePricingModal, { isOpen: true, fee });
    await settle();

    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /^Stages$/ })).toBeInTheDocument();
  });

  it('keeps a stage edit after it is made', async () => {
    render(FeePricingModal, { isOpen: true, fee });
    await settle();
    await openStagesTab();

    const percentInputs = screen.getAllByRole('spinbutton') as HTMLInputElement[];
    expect(percentInputs.length).toBeGreaterThan(0);
    const input = percentInputs[0];

    await fireEvent.change(input, { target: { value: '37.5' } });
    await settle();

    const after = screen.getAllByRole('spinbutton') as HTMLInputElement[];
    expect(after[0].value).toBe('37.5');
  });

  it('closes without a prompt when nothing was edited', async () => {
    const onclose = vi.fn();
    render(FeePricingModal, { isOpen: true, fee, onclose });
    await settle();

    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(screen.queryByText('Discard unsaved pricing changes?')).not.toBeInTheDocument();
    expect(onclose).toHaveBeenCalledTimes(1);
  });

  it('prompts to discard when Cancel is pressed after an edit, and keeps the modal open', async () => {
    const onclose = vi.fn();
    render(FeePricingModal, { isOpen: true, fee, onclose });
    await settle();
    await openStagesTab();

    const input = (screen.getAllByRole('spinbutton') as HTMLInputElement[])[0];
    await fireEvent.change(input, { target: { value: '37.5' } });
    await settle();

    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(screen.getByText('Discard unsaved pricing changes?')).toBeInTheDocument();
    expect(onclose).not.toHaveBeenCalled();

    await fireEvent.click(screen.getByRole('button', { name: 'Discard' }));
    expect(onclose).toHaveBeenCalledTimes(1);
  });

  it('opens a fee that already has pricing and closes silently without edits', async () => {
    const onclose = vi.fn();
    render(FeePricingModal, { isOpen: true, fee: feeWithPricing(), onclose });
    await settle();

    expect(screen.getByRole('dialog')).toBeInTheDocument();
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(screen.queryByText('Discard unsaved pricing changes?')).not.toBeInTheDocument();
    expect(onclose).toHaveBeenCalledTimes(1);
  });

  for (const [label, makeFee] of [
    ['without pricing', () => fee],
    ['with pricing', feeWithPricing]
  ] as const) {
    it(`does not prompt after visiting the Calculator tab with no edits (${label})`, async () => {
      const onclose = vi.fn();
      render(FeePricingModal, { isOpen: true, fee: makeFee(), onclose });
      await settle();

      await fireEvent.click(screen.getByRole('button', { name: /^Calculator$/ }));
      await settle();

      await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

      expect(screen.queryByText('Discard unsaved pricing changes?')).not.toBeInTheDocument();
      expect(onclose).toHaveBeenCalledTimes(1);
    });
  }

  it('still prompts when an edit was made before visiting the Calculator tab', async () => {
    const onclose = vi.fn();
    render(FeePricingModal, { isOpen: true, fee, onclose });
    await settle();
    await openStagesTab();

    const input = (screen.getAllByRole('spinbutton') as HTMLInputElement[])[0];
    await fireEvent.change(input, { target: { value: '37.5' } });
    await settle();

    await fireEvent.click(screen.getByRole('button', { name: /^Calculator$/ }));
    await settle();
    await fireEvent.click(screen.getByRole('button', { name: 'Cancel' }));

    expect(screen.getByText('Discard unsaved pricing changes?')).toBeInTheDocument();
    expect(onclose).not.toHaveBeenCalled();
  });
});
