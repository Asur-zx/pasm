import { apiKey, serverUrl } from './state.js';

/**
 * Make an authenticated request to the pasm server.
 *
 * Automatically attaches the Authorization and Content-Type headers.
 * Parses the response as JSON when possible; falls back to raw text.
 *
 * @throws When the server returns a non-2xx status.
 */
export async function api(path, opts = {}) {
    const res = await fetch(`${serverUrl}${path}`, {
        ...opts,
        headers: {
            'Authorization': `Bearer ${apiKey}`,
            'Content-Type': 'application/json',
            ...opts.headers,
        },
    });
    const text = await res.text();
    if (!res.ok) throw new Error(text || `HTTP ${res.status}`);
    try { return JSON.parse(text); } catch { return text; }
}
