package com.oicana;

/**
 * Result of a one-shot template export.
 *
 * @param document the exported document
 * @param warnings compilation warnings, or {@code null} if there were none
 */
public record ExportOnceResult(byte[] document, String warnings) {
}
