/** Resolved WASM module reference once initialised. */
export let wasm = null;

/**
 * Load and initialise the WASM crypto module.
 *
 * @throws When the WASM module cannot be loaded.
 */
export async function initWasm() {
    try {
        const mod = await import('../../pasm-wasm/pkg/pasm_wasm.js');
        await mod.default();
        wasm = mod;
    } catch {
        throw new Error('Failed to load WASM crypto module');
    }
}
