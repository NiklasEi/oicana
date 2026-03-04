package com.oicana;

/**
 * The compilation mode for a template.
 */
public enum CompilationMode {
    /**
     * Production mode. Use this for generating final documents.
     */
    PRODUCTION(0),
    /**
     * Development mode. Use this for previews and during template development.
     */
    DEVELOPMENT(1);

    final int value;

    CompilationMode(int value) {
        this.value = value;
    }
}
