package com.oicana;

import java.util.Optional;

/**
 * A step in the function calls leading to a {@link Diagnostic}.
 *
 * @param message  what happened at this step, like {@code error occurred in this function call}
 * @param location where the step happened, if it points into a file
 */
public record TracePoint(String message, Optional<SourceLocation> location) {
}
