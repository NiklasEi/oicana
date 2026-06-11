<?php

declare(strict_types=1);

namespace Oicana;

/**
 * A contiguous, 0-based inclusive range of document pages to export.
 *
 * Both bounds are optional; leaving one as null keeps it open.
 *
 * Example:
 * ```php
 * // Just the first page
 * $range = PageRange::single(0);
 *
 * // The second through fourth pages (indices 1 to 3)
 * $range = PageRange::of(1, 3);
 *
 * $pdf = $template->export(exportFormat: ExportFormat::pdf(), pages: $range);
 * ```
 */
final readonly class PageRange
{
    private function __construct(private ?int $start, private ?int $end)
    {
    }

    /**
     * A range selecting exactly the page at the given 0-based index.
     */
    public static function single(int $page): self
    {
        return new self($page, $page);
    }

    /**
     * A range with the given (optional) 0-based, inclusive bounds.
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

    /**
     * Serialize this range to the JSON object string the native extension expects.
     *
     * @throws \JsonException If encoding fails
     */
    public function toNative(): string
    {
        return json_encode((object) $this->toArray(), JSON_THROW_ON_ERROR);
    }
}
