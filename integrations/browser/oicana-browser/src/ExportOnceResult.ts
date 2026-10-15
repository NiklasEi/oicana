import type { Diagnostic } from './Diagnostic.js';

/**
 * Result of a one-shot template export.
 */
export interface ExportOnceResult {
  /** The exported document. */
  document: Uint8Array<ArrayBuffer>;
  /** Compilation warnings. Empty if there were none. */
  warnings: Diagnostic[];
}
