package com.oicana;

/**
 * Global Oicana configuration.
 */
public final class Configuration {
    private Configuration() {
    }

    /**
     * Configure the coloring of compilation diagnostics like warnings and errors.
     *
     * @param color the color mode to use
     */
    public static void setDiagnosticColor(DiagnosticColor color) {
        OicanaNative.configureDiagnosticColor(color == DiagnosticColor.ANSI);
    }
}
