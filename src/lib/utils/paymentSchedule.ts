/**
 * Payment schedule generation logic, extracted from PaymentSchedulePanel.svelte
 * for unit testing.
 *
 * Business rule (per Martin, 2026-04-21, ailx hub message:9a3hw83prc2gt5rqpat2):
 * The mobilisation rebate is split EQUALLY across the number of design stages,
 * not pro-rated by stage value. Each stage pays its quoted total minus an equal
 * share of the mobilisation amount.
 *
 *   mobilisation = design_subtotal * (mobilisation_percent / 100)
 *   rebate_per_stage = mobilisation / count(stages)
 *   stage_payment[i] = stage_total[i] - rebate_per_stage
 *
 * Totals invariant: sum(stage_payments) + mobilisation == design_subtotal.
 */

export interface ScheduleStageInput {
  /** Stable stage id used for cross-referencing payment entries back to stages. */
  id: string;
  /** Stage name shown in the description. */
  name: string;
  /** Quoted (rounded) stage total — already-rounded by caller's pricing config. */
  total: number;
}

export interface ScheduleStageOutput {
  stage_id: string;
  stage_name: string;
  /** What the client invoices for this stage after the mobilisation rebate. */
  amount: number;
  /** The stage's quoted total before rebate (kept on the entry for display). */
  quoted_stage_amount: number;
}

export interface ScheduleResult {
  mobilisation: number;
  stages: ScheduleStageOutput[];
  design_subtotal: number;
}

export function buildPaymentSchedule(
  stages: ScheduleStageInput[],
  mobilisationPercent: number
): ScheduleResult {
  const designSubtotal = stages.reduce((sum, s) => sum + s.total, 0);
  const mobilisation = designSubtotal * (mobilisationPercent / 100);
  const rebatePerStage = stages.length > 0 ? mobilisation / stages.length : 0;

  const stageOutputs = stages.map(stage => ({
    stage_id: stage.id,
    stage_name: stage.name,
    amount: stage.total - rebatePerStage,
    quoted_stage_amount: stage.total
  }));

  return {
    mobilisation,
    stages: stageOutputs,
    design_subtotal: designSubtotal
  };
}
