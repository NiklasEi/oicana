package com.oicana;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

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

    /**
     * Make fonts available to every template registered from now on.
     *
     * @param fonts raw content of the font files; data that holds no font is ignored
     * @return the number of font faces that were added
     */
    public static int registerFonts(byte[]... fonts) {
        int faces = 0;
        for (byte[] font : fonts) {
            faces += OicanaNative.registerFont(font);
        }
        return faces;
    }

    /**
     * Make fonts on disk available to every template registered from now on.
     *
     * @param paths paths to font files
     * @return the number of font faces that were added
     */
    public static int registerFontPaths(String... paths) {
        int faces = 0;
        for (String path : paths) {
            faces += OicanaNative.registerFontPath(path);
        }
        return faces;
    }

    /**
     * All font faces currently registered by the host.
     *
     * @return an unmodifiable list of the registered faces
     */
    public static List<RegisteredFont> registeredFonts() {
        Object[] flat = OicanaNative.registeredFonts();
        List<RegisteredFont> fonts = new ArrayList<>(flat.length / 2);
        for (int index = 0; index + 1 < flat.length; index += 2) {
            fonts.add(new RegisteredFont((String) flat[index], (String) flat[index + 1]));
        }
        return Collections.unmodifiableList(fonts);
    }

    /**
     * Drop all fonts registered by the host.
     *
     * <p>Templates that are already registered keep the fonts they were created with.
     */
    public static void clearFonts() {
        OicanaNative.clearFonts();
    }
}
