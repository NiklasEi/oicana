package com.oicana;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;

/**
 * Loads the platform-specific native library from the classpath.
 */
class NativeLoader {
    private static boolean loaded = false;

    static synchronized void load() {
        if (loaded) {
            return;
        }

        String os = detectOs();
        String arch = detectArch();
        String libName = System.mapLibraryName("oicana_java_native");
        String resourcePath = "natives/" + os + "-" + arch + "/" + libName;

        try (InputStream is = NativeLoader.class.getClassLoader().getResourceAsStream(resourcePath)) {
            if (is == null) {
                throw new OicanaException(
                        "Native library not found for platform " + os + "-" + arch
                                + ". Looked for resource: " + resourcePath);
            }

            Path tempDir = Files.createTempDirectory("oicana-native");
            Path tempLib = tempDir.resolve(libName);
            Files.copy(is, tempLib, StandardCopyOption.REPLACE_EXISTING);
            tempLib.toFile().deleteOnExit();
            tempDir.toFile().deleteOnExit();

            System.load(tempLib.toAbsolutePath().toString());
            loaded = true;
        } catch (IOException e) {
            throw new OicanaException("Failed to load native library", e);
        }
    }

    private static String detectOs() {
        String os = System.getProperty("os.name").toLowerCase();
        if (os.contains("linux")) return "linux";
        if (os.contains("mac") || os.contains("darwin")) return "macos";
        if (os.contains("win")) return "windows";
        throw new OicanaException("Unsupported OS: " + os);
    }

    private static String detectArch() {
        String arch = System.getProperty("os.arch").toLowerCase();
        if (arch.equals("amd64") || arch.equals("x86_64")) return "x86_64";
        if (arch.equals("aarch64") || arch.equals("arm64")) return "aarch64";
        throw new OicanaException("Unsupported architecture: " + arch);
    }
}
