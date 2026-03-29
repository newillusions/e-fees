/**
 * Fee Stages API methods
 *
 * Handles access to pricing stages on fee records. Used by the scope module
 * to link scope sections to design and post-contract pricing stages.
 */

import { invoke } from '@tauri-apps/api/core';
import { logApiError } from '../services/logger';

/**
 * Represents a pricing stage within a fee proposal.
 *
 * Stages are used to organize pricing into design phases (e.g., Concept,
 * Scheme Design) and post-contract phases (e.g., Detailed Design, Construction Admin).
 */
export interface FeeStage {
  /** Unique identifier for the stage */
  id: string;
  /** Display name of the stage (e.g., "Concept Design") */
  name: string;
  /** Short code for the stage (e.g., "CD") */
  code: string;
  /** Percentage of total fee allocated to this stage (0-100) */
  percentage: number;
  /** Display order within the pricing breakdown */
  order: number;
  /** True if this is a post-contract phase; false for design phases */
  is_post_contract: boolean;
}

/**
 * Retrieves all pricing stages for a fee proposal.
 *
 * @param feeId - The fee ID (string, from SurrealDB record)
 * @returns Array of pricing stages, empty array if none configured
 * @throws Error if database query fails
 *
 * @example
 * const stages = await getFeeStages('fee:project-123-1')
 * console.log(stages.length) // 0 or more stages
 */
export async function getFeeStages(feeId: string): Promise<FeeStage[]> {
  try {
    const stages = await invoke<FeeStage[]>('get_fee_stages', { feeId });
    return stages;
  } catch (error) {
    logApiError('getFeeStages', error as Error, { feeId, component: 'FeeStagesApi' });
    throw error;
  }
}

/**
 * Adds a pricing stage to a fee proposal.
 *
 * Idempotent: if a stage with the same id already exists, it will not be duplicated.
 * Creates a minimal pricing structure if none exists.
 *
 * @param feeId - The fee ID (string, from SurrealDB record)
 * @param stage - The stage to add
 * @returns Updated array of stages for the fee
 * @throws Error if database update fails
 *
 * @example
 * const newStage: FeeStage = {
 *   id: 'cd-001',
 *   name: 'Concept Design',
 *   code: 'CD',
 *   percentage: 15,
 *   order: 1,
 *   is_post_contract: false
 * }
 * const stages = await addStageToFee('fee:project-123-1', newStage)
 */
export async function addStageToFee(feeId: string, stage: FeeStage): Promise<FeeStage[]> {
  try {
    const stages = await invoke<FeeStage[]>('add_stage_to_fee', {
      feeId,
      stage
    });
    return stages;
  } catch (error) {
    logApiError('addStageToFee', error as Error, {
      feeId,
      stageId: stage.id,
      component: 'FeeStagesApi'
    });
    throw error;
  }
}
