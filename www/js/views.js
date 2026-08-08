import { wasm } from './wasm.js';
import { encrKey, entries } from './state.js';

/** Convenience alias for document.getElementById. */
const getEl = id => document.getElementById(id);

const loginScreen = getEl('login-screen');
const appScreen = getEl('app-screen');
const detailScreen = getEl('detail-screen');
const entryList = getEl('entry-list');
const msg = getEl('msg');

export { getEl, loginScreen, appScreen, detailScreen, entryList, msg };

/** Show the login screen, hiding app and detail screens. */
export function showLogin() {
    loginScreen.classList.remove('hidden');
    appScreen.classList.add('hidden');
    detailScreen.classList.add('hidden');
}

/** Show the main app screen and re-render the entry list. */
export function showApp() {
    loginScreen.classList.add('hidden');
    appScreen.classList.remove('hidden');
    detailScreen.classList.add('hidden');
    renderEntries();
}

/** Show the detail screen for a single entry. */
export function showDetail(entry) {
    loginScreen.classList.add('hidden');
    appScreen.classList.add('hidden');
    detailScreen.classList.remove('hidden');
    renderDetail(entry);
}

/** Re-render the entry list from the shared entries array. */
export function renderEntries() {
    if (entries.length === 0) {
        entryList.innerHTML = '<div style="color:var(--text-hint);text-align:center;padding:32px;">No entries yet</div>';
        return;
    }
    entryList.innerHTML = entries.map(e => {
        const name = e.key || '?';
        return `<li class="entry-item" data-name="${name}">
            <span class="entry-name">${escapeHtml(name)}</span>
            <span class="entry-actions">
                <button class="btn btn-sm btn-secondary btn-view" data-name="${name}">View</button>
                <button class="btn btn-sm btn-danger btn-delete" data-name="${name}">Delete</button>
            </span>
        </li>`;
    }).join('');

    entryList.querySelectorAll('.btn-view').forEach(b => {
        b.addEventListener('click', e => {
            e.stopPropagation();
            const name = b.dataset.name;
            const entry = entries.find(e => e.key === name);
            if (entry) showDetail(entry);
        });
    });

    entryList.querySelectorAll('.btn-delete').forEach(b => {
        b.addEventListener('click', e => {
            e.stopPropagation();
            const name = b.dataset.name;
            if (confirm(`Delete "${name}"?`)) {
                import('./crud.js').then(m => m.handleDelete(name));
            }
        });
    });

    entryList.querySelectorAll('.entry-item').forEach(item => {
        item.addEventListener('click', () => {
            const name = item.dataset.name;
            const entry = entries.find(e => e.key === name);
            if (entry) showDetail(entry);
        });
    });
}

/** Render the detail view for a single entry, decrypting its value. */
function renderDetail(entry) {
    let details;
    try {
        const decrypted = wasm.decrypt_entry(entry.value, encrKey);
        details = JSON.parse(decrypted);
    } catch {
        details = { name: entry.key, site: '', uname: '', pword: '', note: '(decryption failed)' };
    }

    getEl('detail-content').innerHTML = `
        <div class="detail-row"><span class="detail-label">Name</span><span class="detail-value">${escapeHtml(details.name)}</span></div>
        <div class="detail-row"><span class="detail-label">Site</span><span class="detail-value">${escapeHtml(details.site)}</span></div>
        <div class="detail-row"><span class="detail-label">Username</span><span class="detail-value">${escapeHtml(details.uname)}</span></div>
        <div class="detail-row"><span class="detail-label">Password</span><span class="detail-value">${escapeHtml(details.pword)}</span></div>
        <div class="detail-row"><span class="detail-label">Note</span><span class="detail-value">${escapeHtml(details.note)}</span></div>
    `;
}

/** Show a temporary status message. */
export function showMsg(text, isError = false) {
    msg.textContent = text;
    msg.className = `msg ${isError ? 'msg-error' : 'msg-success'}`;
    msg.classList.remove('hidden');
    setTimeout(() => msg.classList.add('hidden'), 4000);
}

/** Escape a string for safe HTML injection. */
function escapeHtml(s) {
    const d = document.createElement('div');
    d.textContent = s;
    return d.innerHTML;
}
