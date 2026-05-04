<?php

declare(strict_types=1);

namespace Oicana\Installer;

/**
 * Downloads Oicana native extension binaries from GitHub Releases.
 */
final class BinaryDownloader
{
    private const GITHUB_RELEASES_URL = 'https://github.com/oicana/oicana/releases/download';

    public function __construct(
        private readonly string $version = '0.1.0'
    ) {
    }

    /**
     * Download the binary for the given platform into the project's vendor directory.
     *
     * @return string Absolute path to the downloaded binary
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

        $targetDir = $this->getBinDir();
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
