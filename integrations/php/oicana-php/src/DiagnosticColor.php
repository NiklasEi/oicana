<?php

declare(strict_types=1);

namespace Oicana;

/**
 * Color mode for compilation diagnostics.
 */
enum DiagnosticColor
{
    /** No colors in diagnostic output. */
    case None;
    /** ANSI codes for colors in diagnostic output. */
    case Ansi;
}
