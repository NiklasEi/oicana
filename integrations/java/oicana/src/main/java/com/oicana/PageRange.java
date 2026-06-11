package com.oicana;

/**
 * A contiguous, 0-based inclusive range of document pages to export.
 *
 * <p>Both bounds are optional ({@code null} leaves a bound open). For example,
 * {@code PageRange.of(1, null)} selects the second page to the end of the document.
 *
 * @param start the first page index to export (0-based, inclusive), or {@code null} to start at the
 *     first page
 * @param end the last page index to export (0-based, inclusive), or {@code null} to go to the last
 *     page
 */
public record PageRange(Integer start, Integer end) {

    /**
     * Create a range selecting exactly the page at the given 0-based index.
     *
     * @param page the 0-based index of the page to select
     * @return a single-page range
     */
    public static PageRange single(int page) {
        return new PageRange(page, page);
    }

    /**
     * Create a range with the given (nullable) 0-based, inclusive bounds.
     *
     * @param start the first page index to export, or {@code null} to start at the first page
     * @param end the last page index to export, or {@code null} to go to the last page
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
