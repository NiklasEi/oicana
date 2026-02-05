<?php

declare(strict_types=1);

namespace Oicana\Installer;

/**
 * Downloads Oicana native extension binaries from GitHub Releases.
 */
final class BinaryDownloader
{
    private const GITHUB_RELEASES_URL = 'https://github.com/oicana/oicana/releases/download';
    private const EXTENSION_DIR_NAME = 'oicana-extensions';

    public function __construct(
        private readonly string $version = '0.1.0-alpha.1'
    ) {
    }

    /**
     * Download the binary for the given platform.
     *
     * @return string Path to the downloaded binary
     * @throws \RuntimeException If download fails
     */
    public function download(Platform $platform): string
    {
        $binaryName = $platform->getBinaryName();
        $url = sprintf(
            '%s/oicana_php-v%s/%s',
            self::GITHUB_RELEASES_URL,
            $this->version,
            $binaryName
        );

        $targetDir = $this->getExtensionDir();
        if (!is_dir($targetDir)) {
            mkdir($targetDir, 0755, true);
        }

        $targetPath = $targetDir . DIRECTORY_SEPARATOR . $binaryName;

        // Return early if already downloaded
        if (file_exists($targetPath)) {
            return $targetPath;
        }

        // Download with proper error handling
        $content = $this->fetchUrl($url);
        if ($content === false) {
            throw new \RuntimeException(sprintf(
                "Failed to download extension from: %s\n" .
                "Please check:\n" .
                "  1. The release exists on GitHub\n" .
                "  2. Your internet connection\n" .
                "  3. The binary is available for your platform: %s",
                $url,
                $platform->getDescription()
            ));
        }

        file_put_contents($targetPath, $content);
        chmod($targetPath, 0755);

        return $targetPath;
    }

    /**
     * Get the directory where extensions should be installed.
     */
    private function getExtensionDir(): string
    {
        // Try to use PHP's extension directory
        $extensionDir = ini_get('extension_dir');
        if ($extensionDir === false || $extensionDir === '') {
            throw new \RuntimeException("Failed to get the php extension directory");
        }
        if (!is_writable($extensionDir)) {
            throw new \RuntimeException(sprintf(
                "Cannot write to extension directory %s",
                $extensionDir
            ));
        }

        return $extensionDir;
    }

    /**
     * Fetch URL content with proper headers.
     *
     * @return string|false Content on success, false on failure
     */
    private function fetchUrl(string $url): string|false
    {
        $context = stream_context_create([
            'http' => [
                'method' => 'GET',
                'header' => sprintf("User-Agent: oicana-installer %s", $this->version),
                'follow_location' => 1,
                'max_redirects' => 5,
                'timeout' => 60,
            ],
        ]);

        return @file_get_contents($url, false, $context);
    }
}
