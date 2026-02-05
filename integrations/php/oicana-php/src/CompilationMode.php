<?php

declare(strict_types=1);

namespace Oicana;

/**
 * Compilation mode for template rendering.
 *
 * - Development: Uses default values for inputs when not provided
 * - Production: Requires all inputs to be explicitly provided
 */
enum CompilationMode: int
{
    case Production = 0;
    case Development = 1;

    /**
     * Convert to native extension value.
     */
    public function toNative(): int
    {
        return $this->value;
    }
}
