package com.oicana;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * A compiled document kept in memory so it can be exported on demand without re-compiling.
 *
 * Obtain one via {@link Template#compile(java.util.Map, java.util.Map, CompilationMode)}. Use
 * try-with-resources or call {@link #close()} to free the underlying document.
 *
 * <pre>{@code
 * try (var document = template.compile(inputs, blobs)) {
 *     byte[] pdf = document.toPdf();
 *     byte[] firstPage = document.exportPage(0, 2.0f);
 * }
 * }</pre>
 */
public class CompiledDocument implements AutoCloseable {

    // The native document_pages output is a JSON array of {"width": <n>, "height": <n>} objects in
    // field order. The Java integration intentionally has no JSON dependency, so the fixed shape is
    // parsed directly.
    private static final Pattern PAGE_PATTERN = Pattern.compile(
            "\"width\"\\s*:\\s*([-+0-9.eE]+)\\s*,\\s*\"height\"\\s*:\\s*([-+0-9.eE]+)");

    private String documentId;
    private final List<PageSize> pages;

    CompiledDocument(String documentId) {
        this.documentId = documentId;
        this.pages = parsePageSizes(OicanaNative.documentPages(documentId));
    }

    /**
     * The sizes (in points) of every page, in document order.
     *
     * @return an unmodifiable list of page sizes
     */
    public List<PageSize> pages() {
        return pages;
    }

    /**
     * The number of pages in the document.
     *
     * @return the page count
     */
    public int pageCount() {
        return pages.size();
    }

    /**
     * Export the whole document to PDF.
     *
     * @return the PDF document as a byte array
     */
    public byte[] toPdf() {
        return export(ExportFormat.pdf(), null);
    }

    /**
     * Export a range of pages to PDF.
     *
     * @param pages the 0-based, inclusive page range to export, or {@code null} for the whole
     *     document
     * @return the PDF document as a byte array
     */
    public byte[] toPdf(PageRange pages) {
        return export(ExportFormat.pdf(), pages);
    }

    /**
     * Export the whole document in the given format.
     *
     * @param exportFormat the output format
     * @return the exported document as a byte array
     */
    public byte[] export(ExportFormat exportFormat) {
        return export(exportFormat, null);
    }

    /**
     * Export the document in the given format, optionally restricted to a range of pages.
     *
     * @param exportFormat the output format
     * @param pages the 0-based, inclusive page range to export, or {@code null} for the whole
     *     document
     * @return the exported document as a byte array
     */
    public byte[] export(ExportFormat exportFormat, PageRange pages) {
        ensureOpen();
        return OicanaNative.exportDocument(
                documentId,
                exportFormat.toJsonString(),
                pages == null ? "" : pages.toJsonString());
    }

    /**
     * Export a single (zero-based) page of the document to PNG.
     *
     * @param pageIndex zero-based index of the page to export
     * @param pixelsPerPt resolution in pixels per point
     * @return the PNG image as a byte array
     */
    public byte[] exportPage(int pageIndex, float pixelsPerPt) {
        return export(ExportFormat.png(pixelsPerPt), PageRange.single(pageIndex));
    }

    /** Release the cached document. The instance must not be used after calling close(). */
    @Override
    public void close() {
        if (documentId != null) {
            OicanaNative.removeDocument(documentId);
            documentId = null;
        }
    }

    private void ensureOpen() {
        if (documentId == null) {
            throw new IllegalStateException("CompiledDocument has already been closed");
        }
    }

    private static List<PageSize> parsePageSizes(String json) {
        List<PageSize> sizes = new ArrayList<>();
        Matcher matcher = PAGE_PATTERN.matcher(json);
        while (matcher.find()) {
            double width = Double.parseDouble(matcher.group(1));
            double height = Double.parseDouble(matcher.group(2));
            sizes.add(new PageSize(width, height));
        }
        return Collections.unmodifiableList(sizes);
    }
}
