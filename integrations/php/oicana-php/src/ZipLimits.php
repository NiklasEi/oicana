<?php

declare(strict_types=1);

namespace Oicana;

/**
 * Limits applied when reading a packed template zip.
 *
 * A null bound keeps the default (10 000 entries / 512 MiB decompressed).
 */
final class ZipLimits
{
    public function __construct(
        public readonly ?int $maxEntries = null,
        public readonly ?int $maxTotalDecompressedBytes = null,
    ) {
    }
}
