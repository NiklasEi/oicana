<?php

declare(strict_types=1);

namespace Oicana;

/**
 * Result of a one-shot template export.
 */
final class ExportOnceResult
{
    /**
     * @param string $document The exported document bytes
     * @param list<Diagnostic> $warnings Compilation warnings, empty if there were none
     */
    public function __construct(
        public readonly string $document,
        public readonly array $warnings,
    ) {
    }
}
