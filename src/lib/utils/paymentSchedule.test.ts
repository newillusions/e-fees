import { describe, it, expect } from 'vitest';
import { buildPaymentSchedule, type ScheduleStageInput } from './paymentSchedule';

describe('buildPaymentSchedule', () => {
  // ailx hub message:9a3hw83prc2gt5rqpat2 — exact business rule from Martin (2026-04-21):
  // The mobilisation rebate is split EQUALLY across the number of stages, not by value.
  it('splits mobilisation rebate equally across stages (ailx test case)', () => {
    const stages: ScheduleStageInput[] = [
      { id: 'cd', name: 'CD', total: 36000 }, // 30%
      { id: 'sd', name: 'SD', total: 36000 }, // 30%
      { id: 'dd', name: 'DD', total: 48000 }  // 40%
    ];
    const result = buildPaymentSchedule(stages, 30);

    expect(result.mobilisation).toBe(36000);

    const cd = result.stages.find(s => s.stage_id === 'cd')!;
    const sd = result.stages.find(s => s.stage_id === 'sd')!;
    const dd = result.stages.find(s => s.stage_id === 'dd')!;

    // Per ailx: rebate_per_stage = mobilisation_amount / count(stages) = 36000 / 3 = 12000
    // CD: 36000 - 12000 = 24000
    // SD: 36000 - 12000 = 24000
    // DD: 48000 - 12000 = 36000
    expect(cd.amount).toBe(24000);
    expect(sd.amount).toBe(24000);
    expect(dd.amount).toBe(36000);
  });

  it('preserves the totals invariant: sum(stage_payments) + mobilisation == design_subtotal', () => {
    const stages: ScheduleStageInput[] = [
      { id: 'cd', name: 'CD', total: 36000 },
      { id: 'sd', name: 'SD', total: 36000 },
      { id: 'dd', name: 'DD', total: 48000 }
    ];
    const result = buildPaymentSchedule(stages, 30);
    const stageSum = result.stages.reduce((s, x) => s + x.amount, 0);
    expect(stageSum + result.mobilisation).toBe(120000);
  });

  it('handles zero mobilisation as identity (each stage pays its own value)', () => {
    const stages: ScheduleStageInput[] = [
      { id: 'cd', name: 'CD', total: 50000 },
      { id: 'dd', name: 'DD', total: 50000 }
    ];
    const result = buildPaymentSchedule(stages, 0);
    expect(result.mobilisation).toBe(0);
    expect(result.stages.find(s => s.stage_id === 'cd')!.amount).toBe(50000);
    expect(result.stages.find(s => s.stage_id === 'dd')!.amount).toBe(50000);
  });

  it('returns zero entries when there are no stages', () => {
    const result = buildPaymentSchedule([], 30);
    expect(result.mobilisation).toBe(0);
    expect(result.stages).toEqual([]);
  });

  it('handles a single-stage proposal (rebate = full mobilisation)', () => {
    const stages: ScheduleStageInput[] = [{ id: 'cd', name: 'CD', total: 100000 }];
    const result = buildPaymentSchedule(stages, 25);
    expect(result.mobilisation).toBe(25000);
    // Single stage: rebate_per_stage = 25000 / 1 = 25000; CD pays 100000 - 25000 = 75000
    expect(result.stages[0].amount).toBe(75000);
  });

  it('does NOT use the legacy value-proportion formula', () => {
    // Same input as ailx test; legacy buggy formula would give CD 25200 / SD 25200 / DD 33600.
    const stages: ScheduleStageInput[] = [
      { id: 'cd', name: 'CD', total: 36000 },
      { id: 'sd', name: 'SD', total: 36000 },
      { id: 'dd', name: 'DD', total: 48000 }
    ];
    const result = buildPaymentSchedule(stages, 30);
    const cdAmount = result.stages.find(s => s.stage_id === 'cd')!.amount;
    expect(cdAmount).not.toBe(25200);
    expect(cdAmount).toBe(24000);
  });
});
