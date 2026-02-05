<?php

declare(strict_types=1);

namespace Oicana;

/**
 * Export format configuration for template compilation.
 *
 * This class provides type-safe factory methods for creating export format configurations.
 *
 * Example:
 * ```php
 * // PDF export
 * $pdf = $template->compile(exportFormat: ExportFormat::pdf());
 *
 * // PNG export with custom resolution
 * $png = $template->compile(exportFormat: ExportFormat::png(pixelsPerPt: 3.0));
 *
 * // SVG export
 * $svg = $template->compile(exportFormat: ExportFormat::svg());
 * ```
 */
final readonly class ExportFormat
{
    /**
     * @param array{format: string, pixelsPerPt?: float} $config
     */
    private function __construct(
        private array $config
    ) {
    }

    /**
     * Create PDF export format configuration.
     *
     * PDF is the default and most common export format.
     */
    public static function pdf(): self
    {
        return new self(['format' => 'pdf']);
    }

    /**
     * Create SVG export format configuration.
     *
     * SVG is useful for web embedding and vector graphics.
     */
    public static function svg(): self
    {
        return new self(['format' => 'svg']);
    }

    /**
     * Create PNG export format configuration.
     *
     * @param float $pixelsPerPt Resolution multiplier. Higher values produce larger images.
     *                           - 1.0 = 72 DPI (standard)
     *                           - 2.0 = 144 DPI (retina)
     *                           - 3.0 = 216 DPI (high quality print)
     */
    public static function png(float $pixelsPerPt = 2.0): self
    {
        return new self([
            'format' => 'png',
            'pixelsPerPt' => $pixelsPerPt
        ]);
    }

    /**
     * Get the configuration array for the native extension.
     *
     * @return array{format: string, pixelsPerPt?: float}
     */
    public function toArray(): array
    {
        return $this->config;
    }
}
