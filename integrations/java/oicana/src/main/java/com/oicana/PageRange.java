package com.oicana;

/**
 * A contiguous, 1-based inclusive range of document pages to export.
 *
 * <p>Both bounds are optional ({@code null} leaves a bound open). For example,
 * {@code PageRange.of(2, null)} selects page 2 to the end of the document.
 *
 * @param start the first page to export (1-based, inclusive), or {@code null} to start at the first
 *     page
 * @param end the last page to export (1-based, inclusive), or {@code null} to go to the last page
 */
public record PageRange(Integer start, Integer end) {

    /**
     * Create a range selecting exactly the given 1-based page.
     *
     * @param page the 1-based page to select
     * @return a single-page range
     */
    public static PageRange single(int page) {
        return new PageRange(page, page);
    }

    /**
     * Create a range with the given (nullable) 1-based, inclusive bounds.
     *
     * @param start the first page to export, or {@code null} to start at the first page
     * @param end the last page to export, or {@code null} to go to the last page
     * @return a page range
     */
    public static PageRange of(Integer start, Integer end) {
        return new PageRange(start, end);
    }

    /**
     * Serialize this page range to a JSON string for the native layer.
     *
     * @return JSON representation of the page range
     */
    public String toJsonString() {
        StringBuilder builder = new StringBuilder("{");
        boolean hasStart = start != null;
        if (hasStart) {
            builder.append("\"start\":").append(start);
        }
        if (end != null) {
            if (hasStart) {
                builder.append(',');
            }
            builder.append("\"end\":").append(end);
        }
        return builder.append('}').toString();
    }
}
