/** Current API bearer token. */
export let apiKey = '';

/** Current encryption key derived from master password. */
export let encrKey = '';

/** Cached list of entries fetched from server. */
export let entries = [];

/** Base URL of the pasm server. */
export let serverUrl = localStorage.getItem('pasm_server') || 'http://localhost:3000';

/** Update the API bearer token. */
export function setApiKey(v) { apiKey = v; }

/** Update the encryption key. */
export function setEncrKey(v) { encrKey = v; }

/** Replace the entries list. */
export function setEntries(v) { entries = v; }

/** Update the server URL and persist it to localStorage. */
export function setServerUrl(v) { serverUrl = v; }
