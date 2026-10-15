package com.oicana;

import java.util.List;

/**
 * Result of a one-shot template export.
 *
 * @param document the exported document
 * @param warnings compilation warnings, empty if there were none
 */
public record ExportOnceResult(byte[] document, List<Diagnostic> warnings) {
}
