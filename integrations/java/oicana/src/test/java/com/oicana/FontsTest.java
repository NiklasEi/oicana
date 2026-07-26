package com.oicana;

import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests for host fonts registered through {@link Configuration}.
 */
class FontsTest {

    private static String manifestRequiring(String family) {
        return """
                [package]
                name = "font-test"
                version = "0.1.0"
                entrypoint = "main.typ"

                [tool.oicana]
                manifest_version = 1

                [tool.oicana.fonts]
                require = ["%s"]
                """.formatted(family);
    }

    private static final String PLAIN_MANIFEST = """
            [package]
            name = "font-test"
            version = "0.1.0"
            entrypoint = "main.typ"

            [tool.oicana]
            manifest_version = 1
            """;

    private static byte[] packTemplate(String manifest, String mainTypst) throws IOException {
        var stream = new ByteArrayOutputStream();
        try (var zip = new ZipOutputStream(stream)) {
            zip.setMethod(ZipOutputStream.STORED);
            for (var entry : Map.of("typst.toml", manifest, "main.typ", mainTypst).entrySet()) {
                byte[] content = entry.getValue().getBytes(StandardCharsets.UTF_8);
                var zipEntry = new ZipEntry(entry.getKey());
                zipEntry.setSize(content.length);
                var crc = new java.util.zip.CRC32();
                crc.update(content);
                zipEntry.setCrc(crc.getValue());
                zip.putNextEntry(zipEntry);
                zip.write(content);
                zip.closeEntry();
            }
        }
        return stream.toByteArray();
    }

    /**
     * Family the test font provides. No system or Typst-embedded font has it, so a
     * template requiring it can only be registered once the host registers the font.
     */
    private static final String TEST_FAMILY = "Oicana Test";

    /** The test font shipped with the repository. */
    private static Path aFontFile() {
        return Path.of("../../../assets/fonts/oicana-test-font.ttf").toAbsolutePath().normalize();
    }

    // The font registry is process-global, so isolate every test.
    @BeforeEach
    void clearBefore() {
        Configuration.clearFonts();
    }

    @AfterEach
    void clearAfter() {
        Configuration.clearFonts();
    }

    @Test
    void registryStartsEmpty() {
        assertEquals(List.of(), Configuration.registeredFonts());
    }

    @Test
    void registersFontsFromBytesWithoutAPath() throws IOException {
        byte[] data = Files.readAllBytes(aFontFile());

        assertEquals(1, Configuration.registerFonts(data));

        var fonts = Configuration.registeredFonts();
        assertEquals(1, fonts.size());
        assertEquals(TEST_FAMILY, fonts.get(0).family());
        // Registered from memory, so no path is reported.
        assertNull(fonts.get(0).path());
        assertTrue(fonts.get(0).file().isEmpty());
    }

    @Test
    void dataWithoutAFontIsIgnored() {
        assertEquals(0, Configuration.registerFonts("not a font".getBytes(StandardCharsets.UTF_8)));
        assertEquals(List.of(), Configuration.registeredFonts());
    }

    @Test
    void registersFontsByPathAndReportsThePath() {
        Path path = aFontFile();

        assertEquals(1, Configuration.registerFontPaths(path.toString()));

        var fonts = Configuration.registeredFonts();
        assertEquals(1, fonts.size());
        assertEquals(TEST_FAMILY, fonts.get(0).family());
        assertEquals(path.toString(), fonts.get(0).path());
        assertTrue(fonts.get(0).file().isPresent());
    }

    @Test
    void unreadablePathsAreSkipped() {
        assertEquals(0, Configuration.registerFontPaths("/nonexistent/font.ttf"));
        assertEquals(List.of(), Configuration.registeredFonts());
    }

    @Test
    void clearFontsEmptiesTheRegistry() {
        Configuration.registerFontPaths(aFontFile().toString());
        assertFalse(Configuration.registeredFonts().isEmpty());

        Configuration.clearFonts();

        assertEquals(List.of(), Configuration.registeredFonts());
    }

    @Test
    void templateRequiringAnUnavailableFamilyIsRejected() throws IOException {
        byte[] templateFile = packTemplate(manifestRequiring("Nonexistent Host Family"), "Content");

        var error = assertThrows(OicanaException.class, () -> new Template(templateFile));

        assertTrue(error.getMessage().contains("Nonexistent Host Family"));
    }

    @Test
    void testTemplateIsRejectedUntilTheFontIsRegistered() throws IOException {
        byte[] templateFile = packTemplate(manifestRequiring(TEST_FAMILY), "Content");

        // Proves the family really is unavailable without the host font.
        var error = assertThrows(OicanaException.class, () -> new Template(templateFile));

        assertTrue(error.getMessage().contains(TEST_FAMILY));
    }

    @Test
    void templateRequiringARegisteredFamilyCompiles() throws IOException {
        Configuration.registerFontPaths(aFontFile().toString());

        byte[] templateFile = packTemplate(manifestRequiring(TEST_FAMILY), "Content");

        try (var template = new Template(templateFile)) {
            byte[] svg = template.exportSvg(Map.of(), Map.of(), CompilationMode.DEVELOPMENT, null);

            assertTrue(new String(svg, StandardCharsets.UTF_8).contains("<svg"));
        }
    }

    @Test
    void registeredFontRendersWithoutAWarning() throws IOException {
        Configuration.registerFontPaths(aFontFile().toString());

        byte[] templateFile = packTemplate(
                PLAIN_MANIFEST,
                "#set text(font: \"%s\")\nContent".formatted(TEST_FAMILY));

        try (var template = new Template(templateFile)) {
            template.exportSvg(Map.of(), Map.of(), CompilationMode.DEVELOPMENT, null);

            assertTrue(template.warnings().isEmpty());
        }
    }

    /** Without the host font, the same template falls back and warns. */
    @Test
    void unregisteredFamilyWarns() throws IOException {
        byte[] templateFile = packTemplate(
                PLAIN_MANIFEST,
                "#set text(font: \"%s\")\nContent".formatted(TEST_FAMILY));

        try (var template = new Template(templateFile)) {
            template.exportSvg(Map.of(), Map.of(), CompilationMode.DEVELOPMENT, null);

            assertTrue(template.warnings().isPresent());
            assertTrue(template.warnings().get().contains(TEST_FAMILY));
        }
    }
}
