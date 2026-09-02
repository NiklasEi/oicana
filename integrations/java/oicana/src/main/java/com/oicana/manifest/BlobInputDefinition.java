package com.oicana.manifest;

import com.google.gson.annotations.SerializedName;

/**
 * An input taking arbitrary bytes.
 *
 * @param key Key the input is supplied and used under
 * @param required Whether a value of this input is required for compilation
 * @param defaultValue Blob used when no value is supplied, or {@code null}
 * @param development Blob used in development mode when no value is supplied, or {@code null}
 */
public record BlobInputDefinition(
        String key,
        boolean required,
        @SerializedName("default") BlobFallback defaultValue,
        BlobFallback development)
        implements InputDefinition {}
