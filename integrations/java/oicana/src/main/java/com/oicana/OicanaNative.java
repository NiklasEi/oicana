package com.oicana;

import java.util.Map;

/**
 * JNI bridge to the native Oicana library.
 * This class is not part of the public API.
 */
class OicanaNative {
    static {
        NativeLoader.load();
    }

    static native String registerTemplate(
            String templateId,
            byte[] files,
            Map<String, String> jsonInputs,
            Map<String, NativeBlobWithMetadata> blobInputs,
            int compilationMode
    );

    static native String compileTemplate(
            String templateId,
            Map<String, String> jsonInputs,
            Map<String, NativeBlobWithMetadata> blobInputs,
            int compilationMode
    );

    static native byte[] exportDocument(String documentId, String exportFormat);

    static native void removeDocument(String documentId);

    static native void removeWorld(String templateId);

    static native String inputs(String templateId);

    static native String getSource(String templateId, String file);

    static native byte[] getFile(String templateId, String file);

    static native void setValidateInputs(String templateId, boolean validate);

    static native void configureAutomaticCacheEviction(int maxAge);

    static native void evictCache(int maxAge);
}
