package com.oicana;

import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.*;

/**
 * E2E tests for the Oicana Java integration.
 *
 * These tests compile the e2e test template with specific inputs and write
 * PNG outputs to testOutput/. The CI pipeline then uses test_compare to
 * verify these outputs match the reference images pixel-by-pixel.
 */
class E2eTest {

    private static final Path PROJECT_ROOT = Path.of("").toAbsolutePath()
            .getParent().getParent().getParent();

    private static byte[] asset(String file) throws IOException {
        return Files.readAllBytes(PROJECT_ROOT.resolve("assets").resolve(file));
    }

    private static byte[] templateFile() throws IOException {
        return Files.readAllBytes(
                PROJECT_ROOT.resolve("e2e-tests/template/oicana-e2e-test-x.y.z.zip"));
    }

    private static void writeOutput(String filename, byte[] data) throws IOException {
        Path outputDir = Path.of("testOutput");
        Files.createDirectories(outputDir);
        Files.write(outputDir.resolve(filename), data);
    }

    @Test
    void development() throws IOException {
        try (var template = new Template(templateFile())) {
            byte[] image = template.export(
                    Map.of(),
                    Map.of(),
                    ExportFormat.png(1.0f),
                    CompilationMode.DEVELOPMENT
            );
            assertNotNull(image);
            assertTrue(image.length > 0);
            writeOutput("development.png", image);
        }
    }

    @Test
    void production() throws IOException {
        byte[] blob = asset("inputs/input.txt");
        String json = new String(asset("inputs/input.json"));

        try (var template = new Template(templateFile())) {
            byte[] image = template.export(
                    Map.of("development-json", json),
                    Map.of("development-blob", new BlobInput(
                            blob,
                            Map.of("image_format", "jpeg", "foo", 43, "bar", new Object[]{"input", "two"})
                    )),
                    ExportFormat.png(1.0f),
                    CompilationMode.PRODUCTION
            );
            assertNotNull(image);
            assertTrue(image.length > 0);
            writeOutput("production.png", image);
        }
    }

    @Test
    void allInputs() throws IOException {
        byte[] blob = asset("inputs/input.txt");
        String json = new String(asset("inputs/input.json"));

        try (var template = new Template(templateFile())) {
            byte[] image = template.export(
                    Map.of(
                            "default-json", json,
                            "development-json", json,
                            "both-json", json
                    ),
                    Map.of(
                            "default-blob", new BlobInput(
                                    blob,
                                    Map.of("image_format", "jpeg", "foo", 42, "bar", new Object[]{"input", "two"})
                            ),
                            "development-blob", new BlobInput(
                                    blob,
                                    Map.of("image_format", "jpeg", "foo", 43, "bar", new Object[]{"input", "two"})
                            ),
                            "both-blob", new BlobInput(
                                    blob,
                                    Map.of("image_format", "jpeg", "foo", 44, "bar", new Object[]{"input", "two"})
                            )
                    ),
                    ExportFormat.png(1.0f),
                    CompilationMode.PRODUCTION
            );
            assertNotNull(image);
            assertTrue(image.length > 0);
            writeOutput("all-inputs.png", image);
        }
    }

    @Test
    void explicitDevelopmentModeAllowsCompileWithEmptyInputs() throws IOException {
        try (var template = new Template(templateFile())) {
            byte[] image = template.export(
                    Map.of(),
                    Map.of(),
                    ExportFormat.png(1.0f),
                    CompilationMode.DEVELOPMENT
            );
            assertNotNull(image);
            assertTrue(image.length > 0);
        }
    }

    @Test
    void compileDefaultsToProductionMode() throws IOException {
        try (var template = new Template(templateFile())) {
            assertThrows(OicanaException.class, () -> template.export());
        }
    }

    @Test
    void canControlCompilationModeWhenRegistering() throws IOException {
        byte[] bytes = templateFile();
        assertThrows(OicanaException.class, () ->
                new Template(bytes, Map.of(), Map.of(), CompilationMode.PRODUCTION));
    }
}
