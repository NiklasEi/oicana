<?php

declare(strict_types=1);

namespace Oicana;

/**
 * A contiguous, 1-based inclusive range of document pages to export.
 *
 * Both bounds are optional; leaving one as null keeps it open.
 *
 * Example:
 * ```php
 * // A single page
 * $range = PageRange::single(2);
 *
 * // Pages 2 through 4
 * $range = PageRange::of(2, 4);
 *
 * $pdf = $template->export(exportFormat: ExportFormat::pdf(), pages: $range);
 * ```
 */
final readonly class PageRange
{
    private function __construct(private ?int $start, private ?int $end) {}

    /**
     * A range selecting exactly the given 1-based page.
     */
    public static function single(int $page): self
    {
        return new self($page, $page);
    }

    /**
     * A range with the given (optional) 1-based, inclusive bounds.
     */
    public static function of(?int $start = null, ?int $end = null): self
    {
        return new self($start, $end);
    }

    /**
     * Get the configuration array for the native extension.
     *
     * @return array{start?: int, end?: int}
     */
    public function toArray(): array
    {
        $config = [];
        if ($this->start !== null) {
            $config["start"] = $this->start;
        }
        if ($this->end !== null) {
            $config["end"] = $this->end;
        }

        return $config;
    }
}
