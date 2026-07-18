package com.oicana;

import org.junit.jupiter.api.Test;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.Map;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests for {@link Template#warnings()} on the export convenience path.
 */
class TemplateWarningsTest {

    private static final String MINIMAL_MANIFEST = """
            [package]
            name = "template-warnings-test"
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

    @Test
    void exportSurfacesWarnings() throws IOException {
        byte[] templateFile = packTemplate(
                MINIMAL_MANIFEST,
                "#set text(font: \"NonexistentFontTemplate\")\nContent");

        try (var template = new Template(templateFile)) {
            // Constructor warm-up compile already warns.
            assertTrue(template.warnings().isPresent());

            byte[] svg = template.exportSvg(Map.of(), Map.of(), CompilationMode.DEVELOPMENT, null);

            assertTrue(new String(svg, StandardCharsets.UTF_8).contains("<svg"));
            assertTrue(template.warnings().isPresent());
            assertTrue(template.warnings().get().contains("NonexistentFontTemplate"));
        }
    }

    @Test
    void exportWithoutWarningsIsEmpty() throws IOException {
        byte[] templateFile = packTemplate(MINIMAL_MANIFEST, "Content");

        try (var template = new Template(templateFile)) {
            template.exportSvg(Map.of(), Map.of(), CompilationMode.DEVELOPMENT, null);

            assertTrue(template.warnings().isEmpty());
        }
    }
}
