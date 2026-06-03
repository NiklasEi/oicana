import init from '@oicana/browser-wasm';

export {
  configure_automatic_cache_eviction as configureAutomaticCacheEviction,
  evict_cache as evictCache,
  set_validate_inputs as setValidateInputs,
} from '@oicana/browser-wasm';
export * from './CompilationMode.js';
export * from './ExportFormat.js';
export * from './inputs/index.js';
export * from './Template.js';

const initialized: Set<string> = new Set();

/**
 * Initializes the WASM module from the given URL
 * @param wasmPath URL from which to load the WASM module
 */
export async function initialize(wasmPath: string): Promise<void> {
  if (initialized.has(wasmPath)) return;
  await init({ module_or_path: wasmPath });
  initialized.add(wasmPath);
}
