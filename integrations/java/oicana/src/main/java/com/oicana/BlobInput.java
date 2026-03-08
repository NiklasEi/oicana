package com.oicana;

import java.util.Map;

/**
 * A blob input for a template, consisting of binary data and optional metadata.
 *
 * @param data     the binary content of the blob
 * @param metadata optional metadata as key-value pairs (will be serialized to JSON)
 */
public record BlobInput(byte[] data, Map<String, Object> metadata) {
    public BlobInput(byte[] data) {
        this(data, null);
    }
}
