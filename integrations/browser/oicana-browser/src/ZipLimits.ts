/**
 * Limits applied when reading a packed template zip.
 *
 * Missing values keep the defaults (10 000 entries / 512 MiB decompressed).
 */
export interface ZipLimits {
  /** Maximum number of zip entries. */
  maxEntries?: number;
  /** Maximum total decompressed size in bytes. */
  maxTotalDecompressedBytes?: number;
}
