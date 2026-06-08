package com.oicana;

import java.util.HashMap;
import java.util.Map;
import java.util.UUID;

/**
 * A template for generating documents.
 *
 * <p>The template ZIP file is loaded during construction and cached for subsequent compilations.
 * Template instances are thread-safe and can be shared across threads.
 *
 * <p>Use try-with-resources for automatic cleanup:
 * <pre>{@code
 * try (var template = new Template(zipBytes)) {
 *     byte[] pdf = template.export(Map.of("name", "{\"value\": \"World\"}"), Map.of());
 * }
 * }</pre>
 */
public class Template implements AutoCloseable {
    private final String templateId;
    private boolean closed = false;

    /**
     * Register a template with the given template file.
     * The template is compiled once in development mode to warm up the cache.
     *
     * @param templateFile the packed Oicana template ZIP file
     */
    public Template(byte[] templateFile) {
        this(templateFile, Map.of(), Map.of(), CompilationMode.DEVELOPMENT);
    }

    /**
     * Register a template with the given template file and inputs.
     *
     * @param templateFile the packed Oicana template ZIP file
     * @param jsonInputs   JSON inputs for the initial warm-up compilation
     * @param blobInputs   blob inputs for the initial warm-up compilation
     * @param mode         compilation mode for the initial warm-up compilation
     */
    public Template(byte[] templateFile, Map<String, String> jsonInputs,
                    Map<String, BlobInput> blobInputs, CompilationMode mode) {
        this.templateId = UUID.randomUUID().toString();
        String documentId = OicanaNative.registerTemplate(
                this.templateId,
                templateFile,
                jsonInputs,
                convertBlobInputs(blobInputs),
                mode.value
        );
        OicanaNative.removeDocument(documentId);
    }

    /**
     * Compile the template to PDF without any inputs in production mode.
     *
     * @return the compiled document as a byte array
     */
    public byte[] export() {
        return export(Map.of(), Map.of(), ExportFormat.pdf(), CompilationMode.PRODUCTION);
    }

    /**
     * Compile the template without any inputs using the given export format and compilation mode.
     *
     * @param exportFormat the output format
     * @param mode         the compilation mode
     * @return the compiled document as a byte array
     */
    public byte[] export(ExportFormat exportFormat, CompilationMode mode) {
        return export(Map.of(), Map.of(), exportFormat, mode);
    }

    /**
     * Compile the template to PDF with the given inputs in production mode.
     *
     * @param jsonInputs the JSON inputs for the template
     * @param blobInputs the blob inputs for the template
     * @return the compiled document as a byte array
     */
    public byte[] export(Map<String, String> jsonInputs, Map<String, BlobInput> blobInputs) {
        return export(jsonInputs, blobInputs, ExportFormat.pdf(), CompilationMode.PRODUCTION);
    }

    /**
     * Compile the template with the given inputs, export format, and compilation mode.
     *
     * @param jsonInputs   the JSON inputs for the template
     * @param blobInputs   the blob inputs for the template
     * @param exportFormat the output format
     * @param mode         the compilation mode
     * @return the compiled document as a byte array
     */
    public byte[] export(Map<String, String> jsonInputs, Map<String, BlobInput> blobInputs,
                          ExportFormat exportFormat, CompilationMode mode) {
        return export(jsonInputs, blobInputs, exportFormat, mode, null);
    }

    /**
     * Compile the template and export a range of pages.
     *
     * @param jsonInputs   the JSON inputs for the template
     * @param blobInputs   the blob inputs for the template
     * @param exportFormat the output format
     * @param mode         the compilation mode
     * @param pages        the 1-based, inclusive page range to export, or {@code null} for the
     *                     whole document
     * @return the compiled document as a byte array
     */
    public byte[] export(Map<String, String> jsonInputs, Map<String, BlobInput> blobInputs,
                          ExportFormat exportFormat, CompilationMode mode, PageRange pages) {
        ensureNotClosed();
        String documentId = OicanaNative.compileTemplate(
                this.templateId,
                jsonInputs,
                convertBlobInputs(blobInputs),
                mode.value
        );
        try {
            return OicanaNative.exportDocument(
                    documentId,
                    exportFormat.toJsonString(),
                    pages == null ? "" : pages.toJsonString()
            );
        } finally {
            OicanaNative.removeDocument(documentId);
        }
    }

    /**
     * Compile the template in production mode without any inputs.
     *
     * @return a handle to the compiled document
     */
    public CompiledDocument compile() {
        return compile(Map.of(), Map.of(), CompilationMode.PRODUCTION);
    }

    /**
     * Compile the template with the given inputs in production mode.
     *
     * @param jsonInputs the JSON inputs for the template
     * @param blobInputs the blob inputs for the template
     * @return a handle to the compiled document
     */
    public CompiledDocument compile(Map<String, String> jsonInputs, Map<String, BlobInput> blobInputs) {
        return compile(jsonInputs, blobInputs, CompilationMode.PRODUCTION);
    }

    /**
     * Compile the template.
     *
     * <p>Unlike {@link #export(Map, Map, ExportFormat, CompilationMode)}, the document is kept in
     * memory so it can be exported one or more times without re-compiling. Use try-with-resources
     * or call {@link CompiledDocument#close()} to free it.
     *
     * @param jsonInputs the JSON inputs for the template
     * @param blobInputs the blob inputs for the template
     * @param mode       the compilation mode
     * @return a handle to the compiled document
     */
    public CompiledDocument compile(Map<String, String> jsonInputs, Map<String, BlobInput> blobInputs,
                                    CompilationMode mode) {
        ensureNotClosed();
        String documentId = OicanaNative.compileTemplate(
                this.templateId,
                jsonInputs,
                convertBlobInputs(blobInputs),
                mode.value
        );
        return new CompiledDocument(documentId);
    }

    /**
     * Compile a template in a single call without caching.
     * Useful for one-off compilations where template reuse is not needed.
     *
     * @param templateFile the packed Oicana template ZIP file
     * @param jsonInputs   the JSON inputs
     * @param blobInputs   the blob inputs
     * @param exportFormat the output format
     * @param mode         the compilation mode
     * @return the compiled document as a byte array
     */
    public static byte[] exportOnce(byte[] templateFile, Map<String, String> jsonInputs,
                                     Map<String, BlobInput> blobInputs,
                                     ExportFormat exportFormat, CompilationMode mode) {
        try (var template = new Template(templateFile, jsonInputs, blobInputs, mode)) {
            return template.export(jsonInputs, blobInputs, exportFormat, mode);
        }
    }

    /**
     * Get the input definitions for this template as a JSON string.
     *
     * @return JSON string describing the template's input definitions
     */
    public String inputs() {
        ensureNotClosed();
        return OicanaNative.inputs(this.templateId);
    }

    /**
     * Get the source code of a file within the template.
     *
     * @param path the file path within the template
     * @return the source code as a string
     */
    public String source(String path) {
        ensureNotClosed();
        return OicanaNative.getSource(this.templateId, path);
    }

    /**
     * Get the binary content of a file within the template.
     *
     * @param path the file path within the template
     * @return the file content as a byte array
     */
    public byte[] file(String path) {
        ensureNotClosed();
        return OicanaNative.getFile(this.templateId, path);
    }

    /**
     * Enable or disable JSON schema validation for this template.
     *
     * <p>When enabled (the default), JSON inputs are validated against their schemas
     * before compilation.
     *
     * @param validate whether to validate inputs against their JSON schemas
     */
    public void setValidateInputs(boolean validate) {
        ensureNotClosed();
        OicanaNative.setValidateInputs(this.templateId, validate);
    }

    /**
     * Configure automatic cache eviction after each compilation.
     *
     * @param maxAge maximum age threshold. Use -1 to disable eviction, 0 to clear all,
     *               or a positive value to keep entries used within the last n evictions.
     *               Default is 10.
     */
    public static void configureAutomaticCacheEviction(int maxAge) {
        OicanaNative.configureAutomaticCacheEviction(maxAge);
    }

    /**
     * Manually evict the cache with the given age threshold.
     *
     * @param maxAge the age threshold for cache eviction
     */
    public static void evictCache(int maxAge) {
        OicanaNative.evictCache(maxAge);
    }

    /**
     * Release native resources associated with this template.
     * After calling close(), this template instance should not be used.
     */
    @Override
    public void close() {
        if (!closed) {
            closed = true;
            OicanaNative.removeWorld(this.templateId);
        }
    }

    private void ensureNotClosed() {
        if (closed) {
            throw new OicanaException("Template has been closed");
        }
    }

    private static Map<String, NativeBlobWithMetadata> convertBlobInputs(Map<String, BlobInput> blobInputs) {
        if (blobInputs == null || blobInputs.isEmpty()) {
            return Map.of();
        }
        Map<String, NativeBlobWithMetadata> result = new HashMap<>(blobInputs.size());
        for (var entry : blobInputs.entrySet()) {
            BlobInput blob = entry.getValue();
            String meta = blob.metadata() == null ? "{}" : toJson(blob.metadata());
            result.put(entry.getKey(), new NativeBlobWithMetadata(blob.data(), meta));
        }
        return result;
    }

    private static String toJson(Map<String, Object> map) {
        StringBuilder sb = new StringBuilder("{");
        boolean first = true;
        for (var entry : map.entrySet()) {
            if (!first) sb.append(",");
            first = false;
            sb.append("\"").append(escapeJson(entry.getKey())).append("\":");
            sb.append(valueToJson(entry.getValue()));
        }
        sb.append("}");
        return sb.toString();
    }

    @SuppressWarnings("unchecked")
    private static String valueToJson(Object value) {
        if (value == null) return "null";
        if (value instanceof String s) return "\"" + escapeJson(s) + "\"";
        if (value instanceof Number n) return n.toString();
        if (value instanceof Boolean b) return b.toString();
        if (value instanceof Map<?, ?> m) return toJson((Map<String, Object>) m);
        if (value instanceof Object[] arr) {
            StringBuilder sb = new StringBuilder("[");
            for (int i = 0; i < arr.length; i++) {
                if (i > 0) sb.append(",");
                sb.append(valueToJson(arr[i]));
            }
            sb.append("]");
            return sb.toString();
        }
        if (value instanceof Iterable<?> iter) {
            StringBuilder sb = new StringBuilder("[");
            boolean first = true;
            for (Object item : iter) {
                if (!first) sb.append(",");
                first = false;
                sb.append(valueToJson(item));
            }
            sb.append("]");
            return sb.toString();
        }
        return "\"" + escapeJson(value.toString()) + "\"";
    }

    private static String escapeJson(String s) {
        return s.replace("\\", "\\\\")
                .replace("\"", "\\\"")
                .replace("\n", "\\n")
                .replace("\r", "\\r")
                .replace("\t", "\\t");
    }
}
