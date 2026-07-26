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

    /**
     * Make fonts available to every template registered from now on.
     *
     * @param list<string> $fonts Raw content of the font files. Data that holds no
     *                            font is ignored.
     *
     * @return int The number of font faces that were added.
     */
    public static function registerFonts(array $fonts): int
    {
        $faces = 0;
        foreach ($fonts as $font) {
            $faces += \OicanaInternal\register_font($font);
        }

        return $faces;
    }

    /**
     * Make fonts on disk available to every template registered from now on.
     *
     * @param list<string> $paths Paths to font files.
     *
     * @return int The number of font faces that were added.
     */
    public static function registerFontPaths(array $paths): int
    {
        return \OicanaInternal\register_font_paths($paths);
    }

    /**
     * All font faces currently registered by the host.
     *
     * @return list<RegisteredFont>
     */
    public static function registeredFonts(): array
    {
        /** @var list<array{family: string, path: string|null}> $fonts */
        $fonts = json_decode(\OicanaInternal\registered_fonts(), true, 512, JSON_THROW_ON_ERROR);

        return array_map(
            static fn (array $font): RegisteredFont => new RegisteredFont($font['family'], $font['path']),
            $fonts,
        );
    }

    /**
     * Drop all fonts registered by the host.
     *
     * Templates that are already registered keep the fonts they were created with.
     */
    public static function clearFonts(): void
    {
        \OicanaInternal\clear_fonts();
    }
}
