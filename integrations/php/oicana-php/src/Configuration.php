<?php

declare(strict_types=1);

namespace Oicana;

/**
 * Global Oicana configuration.
 */
final class Configuration
{
    private function __construct()
    {
    }

    /**
     * Configure the coloring of compilation diagnostics like warnings and errors.
     */
    public static function setDiagnosticColor(DiagnosticColor $color): void
    {
        \OicanaInternal\configure_diagnostic_color($color === DiagnosticColor::Ansi);
    }
}
