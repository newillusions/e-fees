/**
 * Folder Reconcile API methods
 *
 * On-disk follow-up to a project merge or cascade-delete (DB-only ops -
 * see projects.ts / db/project_lifecycle.rs). Optional, decoupled: a
 * cancelled or failed reconcile never rolls back the DB merge/delete that
 * already committed. See src-tauri/src/commands/folder_reconcile.rs for the
 * backend and its safety invariants (never a silent overwrite, dry-run
 * before any write, size-first conflict detection).
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  FolderReconcilePreview,
  FolderReconcileOutcome,
  ReconcileResolution,
  TrashFolderPreview,
  TrashFolderOutcome
} from '../../types';
import { logApiError } from '../services/logger';

/**
 * Walk `sourcePath` and classify every file against `targetPath` (new /
 * conflict / identical). Read-only.
 *
 * `sourceNumber`/`targetNumber` are the two projects' hyphenated numbers
 * (e.g. "26-97110"). Project folders are created from a template whose
 * files get the project's number substituted into their names
 * (`rename_template_files_cross_platform` in `template_ops.rs`) - so the
 * same logical base file carries a DIFFERENT literal name in each project
 * folder. Passing both numbers lets the backend pair files by that
 * project-number-agnostic identity instead of raw path, so a renamed base
 * file classifies as one real conflict/identical pair rather than two
 * unrelated "new" files. Pass empty strings to disable this and pair
 * strictly by relative path.
 */
export async function previewFolderReconcile(
  sourcePath: string,
  targetPath: string,
  sourceNumber: string,
  targetNumber: string
): Promise<FolderReconcilePreview> {
  try {
    return await invoke<FolderReconcilePreview>('preview_folder_reconcile', {
      sourcePath,
      targetPath,
      sourceNumber,
      targetNumber
    });
  } catch (error) {
    logApiError('previewFolderReconcile', error as Error, { component: 'FolderReconcileApi' });
    throw error;
  }
}

/**
 * Apply (or, with `dryRun: true`, simulate) a set of per-file resolutions
 * from a prior `previewFolderReconcile` call. Call once with `dryRun: true`
 * to render a confirmation summary, then again with `dryRun: false` after
 * the user confirms.
 *
 * `sourceNumber`/`targetNumber` must match what was passed to
 * `previewFolderReconcile` - the backend recomputes each destination path
 * fresh from these rather than trusting a client-supplied path, so a copy
 * of a source-numbered file lands under the target's number, not the
 * source's.
 */
export async function executeFolderReconcile(
  sourcePath: string,
  targetPath: string,
  resolutions: ReconcileResolution[],
  dryRun: boolean,
  sourceNumber: string,
  targetNumber: string
): Promise<FolderReconcileOutcome> {
  try {
    return await invoke<FolderReconcileOutcome>('execute_folder_reconcile', {
      sourcePath,
      targetPath,
      resolutions,
      dryRun,
      sourceNumber,
      targetNumber
    });
  } catch (error) {
    logApiError('executeFolderReconcile', error as Error, { component: 'FolderReconcileApi' });
    throw error;
  }
}

/**
 * Preview what `executeTrashProjectFolder` would move, for the
 * cascade-delete opt-in checkbox. Read-only.
 */
export async function previewTrashProjectFolder(
  projectNumber: string
): Promise<TrashFolderPreview> {
  try {
    return await invoke<TrashFolderPreview>('preview_trash_project_folder', {
      projectNumber
    });
  } catch (error) {
    logApiError('previewTrashProjectFolder', error as Error, { component: 'FolderReconcileApi' });
    throw error;
  }
}

/**
 * Move a deleted project's on-disk folder to
 * `.reconcile-backups/{timestamp}/` rather than leaving an orphan. Opt-in
 * only - call this AFTER `deleteProjectCascade` has already committed, so a
 * failure here never blocks or reverses the DB delete.
 */
export async function executeTrashProjectFolder(
  projectNumber: string
): Promise<TrashFolderOutcome> {
  try {
    return await invoke<TrashFolderOutcome>('execute_trash_project_folder', {
      projectNumber
    });
  } catch (error) {
    logApiError('executeTrashProjectFolder', error as Error, { component: 'FolderReconcileApi' });
    throw error;
  }
}
