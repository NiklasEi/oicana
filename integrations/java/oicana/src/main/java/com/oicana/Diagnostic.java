package com.oicana;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Locale;
import java.util.Optional;

/**
 * A compilation diagnostic, like a warning, with its location in the template.
 *
 * @param severity  whether this is an error or a warning
 * @param message   the message describing the problem
 * @param location  where in the template the problem is, if it points into a file
 * @param hints     hints on how to address the problem
 * @param trace     the function calls leading to the problem, innermost first
 * @param formatted the diagnostic formatted for humans, with source excerpts and the configured
 *                  {@link DiagnosticColor}
 */
public record Diagnostic(
        DiagnosticSeverity severity,
        String message,
        Optional<SourceLocation> location,
        List<String> hints,
        List<TracePoint> trace,
        String formatted
) {

    /**
     * The diagnostic formatted for humans.
     *
     * @return the same as {@link #formatted()}
     */
    @Override
    public String toString() {
        return formatted;
    }

    static List<Diagnostic> listFromJson(String json) {
        List<Diagnostic> diagnostics = new ArrayList<>();
        try {
            for (JsonElement element : JsonParser.parseString(json).getAsJsonArray()) {
                diagnostics.add(fromJson(element.getAsJsonObject()));
            }
        } catch (RuntimeException error) {
            throw new OicanaException("Failed to read the compilation warnings", error);
        }
        return Collections.unmodifiableList(diagnostics);
    }

    private static Diagnostic fromJson(JsonObject json) {
        List<String> hints = new ArrayList<>();
        for (JsonElement hint : json.getAsJsonArray("hints")) {
            hints.add(hint.getAsString());
        }
        List<TracePoint> trace = new ArrayList<>();
        for (JsonElement element : json.getAsJsonArray("trace")) {
            JsonObject point = element.getAsJsonObject();
            trace.add(new TracePoint(point.get("message").getAsString(), location(point)));
        }
        return new Diagnostic(
                DiagnosticSeverity.valueOf(
                        json.get("severity").getAsString().toUpperCase(Locale.ROOT)),
                json.get("message").getAsString(),
                location(json),
                Collections.unmodifiableList(hints),
                Collections.unmodifiableList(trace),
                json.get("formatted").getAsString()
        );
    }

    private static Optional<SourceLocation> location(JsonObject parent) {
        JsonObject location = parent.getAsJsonObject("location");
        if (location == null) {
            return Optional.empty();
        }
        return Optional.of(new SourceLocation(
                location.get("file").getAsString(),
                location.get("line").getAsInt(),
                location.get("column").getAsInt(),
                location.get("endLine").getAsInt(),
                location.get("endColumn").getAsInt()
        ));
    }
}
