import { writable } from 'svelte/store';

export const startupError = writable<string | null>(null);
