package com.oicana;

/**
 * The export format for a compiled template.
 */
public sealed interface ExportFormat {

    /** PDF export format. */
    record Pdf() implements ExportFormat {}

    /**
     * PNG export format with configurable resolution.
     *
     * @param pixelsPerPt the number of pixels per pt; higher values produce sharper but larger images
     */
    record Png(float pixelsPerPt) implements ExportFormat {
        /**
         * @throws IllegalArgumentException if {@code pixelsPerPt} is not a positive, finite number
         */
        public Png {
            if (!Float.isFinite(pixelsPerPt) || pixelsPerPt <= 0) {
                throw new IllegalArgumentException(
                        "pixelsPerPt must be a positive, finite number, got " + pixelsPerPt);
            }
        }
    }

    /** SVG export format. */
    record Svg() implements ExportFormat {}

    /**
     * Create a PDF export format.
     *
     * @return a PDF export format
     */
    static ExportFormat pdf() {
        return new Pdf();
    }

    /**
     * Create a PNG export format with the given resolution.
     *
     * @param pixelsPerPt the number of pixels per point (e.g. 1.0 for 72 DPI, 2.0 for 144 DPI)
     * @return a PNG export format
     */
    static ExportFormat png(float pixelsPerPt) {
        return new Png(pixelsPerPt);
    }

    /**
     * Create an SVG export format.
     *
     * @return an SVG export format
     */
    static ExportFormat svg() {
        return new Svg();
    }

    /**
     * Serialize this export format to a JSON string for the native layer.
     *
     * @return JSON representation of the export format
     */
    default String toJsonString() {
        if (this instanceof Pdf) {
            return "{\"format\":\"pdf\"}";
        } else if (this instanceof Png png) {
            return "{\"format\":\"png\",\"pixelsPerPt\":" + png.pixelsPerPt() + "}";
        } else if (this instanceof Svg) {
            return "{\"format\":\"svg\"}";
        }
        throw new IllegalStateException("Unknown export format");
    }
}
