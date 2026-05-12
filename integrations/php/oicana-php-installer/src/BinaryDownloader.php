<?php

declare(strict_types=1);

namespace Oicana\Installer;

/**
 * Downloads Oicana native extension binaries from GitHub Releases.
 */
final class BinaryDownloader
{
    private const GITHUB_RELEASES_URL = 'https://github.com/oicana/oicana/releases/download';
    private const CHECKSUMS_FILE = __DIR__ . '/../checksums.json';

    public function __construct(
        private readonly string $version
    ) {
    }

    /**
     * Download the binary for the given platform into the project's vendor directory.
     *
     * The downloaded content is verified against the SHA-256 hash recorded for this
     * binary in `checksums.json` before being written to disk.
     *
     * @return string Absolute path to the downloaded binary
     * @throws \RuntimeException If download fails or integrity verification fails
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

        $targetDir = $this->getBinDir();
        $targetPath = $targetDir . DIRECTORY_SEPARATOR . $binaryName;

        // Return early if already downloaded
        if (file_exists($targetPath)) {
            return $targetPath;
        }

        $expectedHash = $this->expectedHash($binaryName);

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

        $actualHash = 'sha256:' . hash('sha256', $content);
        if (!hash_equals($expectedHash, $actualHash)) {
            throw new \RuntimeException(sprintf(
                "Checksum mismatch for %s: expected %s, got %s. Refusing to install.",
                $binaryName,
                $expectedHash,
                $actualHash
            ));
        }

        file_put_contents($targetPath, $content);
        chmod($targetPath, 0755);

        return $targetPath;
    }

    /**
     * Look up the expected SHA-256 for the given binary in `checksums.json`.
     *
     * Throws if the file is missing, malformed, or has no entry for this binary
     * (the source-tree placeholder ships with an empty `binaries` map, so every
     * install attempt against an unbuilt source tree fails here).
     */
    private function expectedHash(string $binaryName): string
    {
        if (!is_file(self::CHECKSUMS_FILE)) {
            throw new \RuntimeException(sprintf(
                'checksums.json not found at %s. Refusing to install an unverified binary.',
                self::CHECKSUMS_FILE
            ));
        }

        $checksums = json_decode(
            (string) file_get_contents(self::CHECKSUMS_FILE),
            true,
            flags: JSON_THROW_ON_ERROR
        );

        $hash = $checksums['binaries'][$binaryName] ?? null;
        if (!is_string($hash) || $hash === '') {
            throw new \RuntimeException(sprintf(
                "No checksum recorded for %s in checksums.json. " .
                "This installer build cannot verify the binary; install via composer.oicana.com or " .
                "download from GitHub Releases manually.",
                $binaryName
            ));
        }

        return $hash;
    }

    /**
     * Get the directory where the downloaded extension binary is stored.
     *
     * The binary is placed inside the project's vendor directory so no elevated
     * permissions are required.
     */
    private function getBinDir(): string
    {
        $targetDir = implode(DIRECTORY_SEPARATOR, [
            getcwd(),
            'vendor',
            'oicana',
            'installer',
            'bin',
        ]);

        if (!is_dir($targetDir)) {
            mkdir($targetDir, 0755, true);
        }

        return $targetDir;
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
