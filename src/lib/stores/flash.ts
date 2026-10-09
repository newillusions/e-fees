/**
 * Flash messages that must survive a navigation or a modal closing.
 *
 * `operationState` messages are wiped when a modal closes, which hid the
 * "proposal created" result and any JSON-export failure. A flash is shown by
 * the page the user lands on and stays until dismissed.
 */
import { writable } from 'svelte/store';

export type FlashTone = 'success' | 'warning' | 'error';

export interface FlashMessage {
  text: string;
  tone: FlashTone;
  /** Route id of a proposal whose edit modal (pricing, scope) should open on arrival. */
  openEditForId?: string;
}

export const flash = writable<FlashMessage | null>(null);

export function setFlash(message: FlashMessage) {
  flash.set(message);
}

export function clearFlash() {
  flash.set(null);
}
