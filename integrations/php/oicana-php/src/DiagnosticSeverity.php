<?php

declare(strict_types=1);

namespace Oicana;

/**
 * The severity of a compilation diagnostic.
 */
enum DiagnosticSeverity: string
{
    /** A fatal error. */
    case Error = 'error';
    /** A non-fatal warning. */
    case Warning = 'warning';
}
