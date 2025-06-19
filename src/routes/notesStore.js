import { writable } from "svelte/store";

// Define the initial empty array with proper typing
/** @type {import('svelte/store').Writable<Array<{id: string, title: string, content: string}>>} */
export const notes = writable([]);

/** @type {import('svelte/store').Writable<Array<{id: string, title: string, content: string}>>} */
export const searchResults = writable([]);

/** @type {import('svelte/store').Writable<string>} */
export const searchQuery = writable("");

/** @type {import('svelte/store').Writable<{id: string, title: string, content: string} | null>} */
export const selectedNote = writable(null);

/** @type {import('svelte/store').Writable<boolean>} */
export const useSemanticSearch = writable(false);

/** @type {import('svelte/store').Writable<number>} */
export const similarityThreshold = writable(0.5); // Default threshold value (0.5)
