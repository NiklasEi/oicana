package com.oicana;

/**
 * Limits applied when reading a packed template zip.
 *
 * <p>A {@code null} bound keeps the default (10 000 entries / 512 MiB decompressed).
 *
 * @param maxEntries                 maximum number of zip entries, or {@code null} for the default
 * @param maxTotalDecompressedBytes  maximum total decompressed size in bytes, or {@code null}
 *                                   for the default
 */
public record ZipLimits(Long maxEntries, Long maxTotalDecompressedBytes) {
}
