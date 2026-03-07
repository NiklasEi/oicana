<?php

declare(strict_types=1);

namespace Oicana\Installer;

/**
 * Represents a PHP platform configuration.
 */
final readonly class Platform
{
    public function __construct(
        public string $os,
        public string $architecture,
        public string $phpVersion,
        public string $threadSafety
    ) {
    }

    /**
     * Get the binary filename for this platform.
     *
     * Format: oicana-php{VERSION}-{OS}-{ARCH}-{TS}.{EXT}
     * Example: oicana-php8.3-linux-x64-nts.so
     */
    public function getBinaryName(): string
    {
        $ext = match ($this->os) {
            'windows' => 'dll',
            'macos' => 'dylib',
            'linux' => 'so',
        };

        return sprintf(
            'oicana-php%s-%s-%s-%s.%s',
            $this->phpVersion,
            $this->os,
            $this->architecture,
            $this->threadSafety,
            $ext
        );
    }

    /**
     * Get a human-readable description of this platform.
     */
    public function getDescription(): string
    {
        return sprintf(
            '%s %s (PHP %s %s)',
            ucfirst($this->os),
            $this->architecture,
            $this->phpVersion,
            strtoupper($this->threadSafety)
        );
    }
}
