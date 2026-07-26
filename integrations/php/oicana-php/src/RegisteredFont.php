<?php

declare(strict_types=1);

namespace Oicana;

/**
 * A font face made available to templates by the host.
 */
final class RegisteredFont
{
    public function __construct(
        /** Family name, as used in Typst's `text(font: ...)`. */
        public readonly string $family,
        /** File the face was read from; null for fonts registered from memory. */
        public readonly ?string $path = null,
    ) {
    }
}
