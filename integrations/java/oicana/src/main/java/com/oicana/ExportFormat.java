package com.oicana;

/**
 * The export format for a compiled template.
 */
public sealed interface ExportFormat {

    record Pdf() implements ExportFormat {}

    record Png(float pixelsPerPt) implements ExportFormat {}

    record Svg() implements ExportFormat {}

    static ExportFormat pdf() {
        return new Pdf();
    }

    static ExportFormat png(float pixelsPerPt) {
        return new Png(pixelsPerPt);
    }

    static ExportFormat svg() {
        return new Svg();
    }

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
