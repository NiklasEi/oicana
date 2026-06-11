<?php

declare(strict_types=1);

namespace Oicana;

/**
 * Size of a single document page, in typographic points (pt).
 */
final readonly class PageSize
{
    public function __construct(
        public float $width,
        public float $height
    ) {
    }
}
