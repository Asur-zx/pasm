import { wasm } from './wasm.js';
import { api } from './api.js';
import { setEntries, setApiKey, setEncrKey, setServerUrl, encrKey } from './state.js';
import { getEl, showApp } from './views.js';

/**
 * Handle the login / unlock flow.
 *
 * Derives API and encryption keys from the master password,
 * authenticates with the server, then fetches the entry list.
 */
export async function handleLogin() {
    const password = getEl('password-input').value;
    if (!password) return;

    try {
        const url = getEl('server-input').value
            || localStorage.getItem('pasm_server')
            || 'http://localhost:3000';
        setApiKey(wasm.derive_api_key(password));
        setEncrKey(wasm.derive_encr_key(password));
        setServerUrl(url);
        localStorage.setItem('pasm_server', url);

        await api('/auth', { method: 'POST' });
    } catch {}

    await fetchEntries();
    showApp();
    getEl('password-input').value = '';
}

/**
 * Handle creating or amending an entry.
 *
 * Encrypts the detail fields with the current encryption key
 * and sends the payload to the server.
 */
export async function handleCreate() {
    const name = getEl('create-name').value.trim();
    const site = getEl('create-site').value.trim();
    const uname = getEl('create-uname').value.trim();
    const pword = getEl('create-pword').value.trim();
    const note = getEl('create-note').value.trim();
    if (!name) return;

    const details = { name, site, uname, pword, note };
    const encrypted = wasm.encrypt_entry(JSON.stringify(details), encrKey);
    const payload = { key: name, value: encrypted };

    await api('/entry/amend', {
        method: 'POST',
        body: JSON.stringify(payload),
    });

    getEl('create-name').value = '';
    getEl('create-site').value = '';
    getEl('create-uname').value = '';
    getEl('create-pword').value = '';
    getEl('create-note').value = '';

    await fetchEntries();
}

/** Handle deleting an entry by name. */
export async function handleDelete(name) {
    await api(`/entry/${encodeURIComponent(name)}`, { method: 'DELETE' });
    await fetchEntries();
    showApp();
}

/** Fetch all entries from the server and update shared state. */
async function fetchEntries() {
    const data = await api('/entries');
    setEntries(Array.isArray(data) ? data : []);
}
