import { writable } from 'svelte/store';

export const currentRoute = writable('dashboard');

export function navigateTo(route: string) {
  currentRoute.set(route);
}