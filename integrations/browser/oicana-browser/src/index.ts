import init, { configure_diagnostic_color } from '@oicana/browser-wasm';

export {
  configure_automatic_cache_eviction as configureAutomaticCacheEviction,
  evict_cache as evictCache,
  set_log_level as setLogLevel,
  set_validate_inputs as setValidateInputs,
} from '@oicana/browser-wasm';
export * from './CompilationMode.js';
export * from './CompiledDocument.js';
export * from './ExportFormat.js';
export * from './ExportOnceResult.js';
export * from './inputs/index.js';
export * from './PageRange.js';
export * from './Template.js';
export * from './ZipLimits.js';

/** Color mode for compilation diagnostics. */
export type DiagnosticColor = 'none' | 'ansi';

/**
 * Configure the coloring of compilation diagnostics like warnings and errors.
 */
export function configureDiagnosticColor(color: DiagnosticColor): void {
  configure_diagnostic_color(color === 'ansi');
}

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
