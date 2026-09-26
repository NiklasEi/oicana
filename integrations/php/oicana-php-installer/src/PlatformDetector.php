<?php

declare(strict_types=1);

namespace Oicana\Installer;

/**
 * Detects the current PHP platform configuration.
 */
final class PlatformDetector
{
    /**
     * Detect the current platform.
     *
     * @throws UnsupportedPlatformException If there is no build for this platform
     * @throws \RuntimeException If platform cannot be detected
     */
    public function detect(): Platform
    {
        return new Platform(
            os: $this->detectOS(),
            architecture: $this->detectArchitecture(),
            phpVersion: $this->detectPhpVersion(),
            threadSafety: $this->detectThreadSafety()
        );
    }

    /**
     * Detect operating system.
     *
     * @throws UnsupportedPlatformException If OS is not supported
     */
    private function detectOS(): string
    {
        return match (PHP_OS_FAMILY) {
            'Windows' => 'windows',
            'Darwin' => 'macos',
            'Linux' => $this->detectLinux(),
            default => throw new UnsupportedPlatformException('Unsupported OS: ' . PHP_OS_FAMILY),
        };
    }

    /**
     * Detect Linux, rejecting musl based systems.
     *
     * @throws UnsupportedPlatformException If the system uses musl
     */
    private function detectLinux(): string
    {
        if ($this->isMusl()) {
            throw new UnsupportedPlatformException(
                'Unsupported platform: there is no musl build of the Oicana extension, and this '
                . 'system uses musl (Alpine and other musl based distributions).'
            );
        }

        return 'linux';
    }

    /**
     * Whether the running PHP links against musl rather than glibc.
     */
    private function isMusl(): bool
    {
        $maps = @file_get_contents('/proc/self/maps');
        if ($maps !== false && $maps !== '') {
            return str_contains($maps, '/ld-musl-');
        }

        $header = @file_get_contents(PHP_BINARY, false, null, 0, 4096);
        return $header !== false && str_contains($header, '/ld-musl-');
    }

    /**
     * Detect CPU architecture.
     *
     * @throws UnsupportedPlatformException If architecture is not supported
     */
    private function detectArchitecture(): string
    {
        $arch = php_uname('m');
        return match (true) {
            str_contains($arch, 'x86_64') || str_contains($arch, 'amd64') || str_contains($arch, 'AMD64') => 'x64',
            str_contains($arch, 'aarch64') || str_contains($arch, 'arm64') => 'arm64',
            default => throw new UnsupportedPlatformException('Unsupported architecture: ' . $arch),
        };
    }

    /**
     * Detect PHP major.minor version.
     *
     * @throws \RuntimeException If PHP version cannot be determined
     */
    private function detectPhpVersion(): string
    {
        $version = PHP_VERSION;
        if (preg_match('/^(\d+\.\d+)/', $version, $matches)) {
            return $matches[1];
        }
        throw new \RuntimeException('Could not detect PHP version from: ' . $version);
    }

    /**
     * Detect thread safety mode.
     */
    private function detectThreadSafety(): string
    {
        return ZEND_THREAD_SAFE ? 'zts' : 'nts';
    }
}
