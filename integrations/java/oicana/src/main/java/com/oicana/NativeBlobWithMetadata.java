package com.oicana;

/**
 * Internal type for passing blob data to the native layer.
 * Fields are accessed directly by JNI.
 */
class NativeBlobWithMetadata {
    @SuppressWarnings("unused") // accessed by JNI
    final byte[] bytes;
    @SuppressWarnings("unused") // accessed by JNI
    final String meta;

    NativeBlobWithMetadata(byte[] bytes, String meta) {
        this.bytes = bytes;
        this.meta = meta;
    }
}
