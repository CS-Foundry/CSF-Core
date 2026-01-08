import { writable } from 'svelte/store';

export const updateInProgress = writable(false);
export const updateVersion = writable<string | null>(null);
