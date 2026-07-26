package com.oicana;

import java.util.Optional;

/**
 * A font face made available to templates by the host.
 *
 * @param family the family name, as used in Typst's {@code text(font: ...)}
 * @param path   the file the face was read from, or null for fonts registered
 *               from memory
 */
public record RegisteredFont(String family, String path) {
    /**
     * The file the face was read from.
     *
     * @return the path, or an empty Optional for fonts registered from memory
     */
    public Optional<String> file() {
        return Optional.ofNullable(path);
    }
}
