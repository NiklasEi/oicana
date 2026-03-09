package com.oicana;

/**
 * Exception thrown when an Oicana operation fails.
 */
public class OicanaException extends RuntimeException {
    /**
     * Create an exception with the given message.
     *
     * @param message the error message
     */
    public OicanaException(String message) {
        super(message);
    }

    /**
     * Create an exception with the given message and cause.
     *
     * @param message the error message
     * @param cause   the underlying cause
     */
    public OicanaException(String message, Throwable cause) {
        super(message, cause);
    }
}
