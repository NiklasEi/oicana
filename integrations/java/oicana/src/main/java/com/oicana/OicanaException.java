package com.oicana;

/**
 * Exception thrown when an Oicana operation fails.
 */
public class OicanaException extends RuntimeException {
    public OicanaException(String message) {
        super(message);
    }

    public OicanaException(String message, Throwable cause) {
        super(message, cause);
    }
}
