import { initWasm } from './wasm.js';
import { showLogin } from './views.js';
import { handleLogin, handleCreate } from './crud.js';
import { getEl } from './views.js';

/**
 * Application entry point.
 *
 * Initialises the WASM module, shows the login screen,
 * and wires up all DOM event listeners.
 */
document.addEventListener('DOMContentLoaded', async () => {
    try {
        await initWasm();
    } catch (e) {
        alert(e.message);
        return;
    }

    showLogin();

    getEl('login-btn').addEventListener('click', handleLogin);
    getEl('password-input').addEventListener('keydown', e => {
        if (e.key === 'Enter') handleLogin();
    });

    getEl('create-btn').addEventListener('click', handleCreate);
    getEl('back-to-app').addEventListener('click', () => import('./views.js').then(m => m.showApp()));
    getEl('logout-btn').addEventListener('click', () => {
        import('./state.js').then(m => {
            m.setApiKey('');
            m.setEncrKey('');
            m.setEntries([]);
        });
        showLogin();
    });
});
