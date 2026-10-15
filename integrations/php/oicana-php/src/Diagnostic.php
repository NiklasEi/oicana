<?php

declare(strict_types=1);

namespace Oicana;

/**
 * A compilation diagnostic, like a warning, with its location in the template.
 */
final readonly class Diagnostic implements \Stringable
{
    /**
     * @param string $message The message describing the problem
     * @param SourceLocation|null $location Where in the template the problem is, if it points into a file
     * @param list<string> $hints Hints on how to address the problem
     * @param list<TracePoint> $trace The function calls leading to the problem, innermost first
     * @param string $formatted The diagnostic formatted for humans, with source excerpts and the
     *     configured diagnostic color
     */
    public function __construct(
        public DiagnosticSeverity $severity,
        public string $message,
        public ?SourceLocation $location,
        public array $hints,
        public array $trace,
        public string $formatted
    ) {
    }

    public function __toString(): string
    {
        return $this->formatted;
    }

    /**
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        /** @var string $severity */
        $severity = $data['severity'];
        /** @var string $message */
        $message = $data['message'];
        /** @var array{file: string, line: int, column: int, endLine: int, endColumn: int}|null $location */
        $location = $data['location'] ?? null;
        /** @var list<string> $hints */
        $hints = $data['hints'];
        /** @var list<array<string, mixed>> $trace */
        $trace = $data['trace'];
        /** @var string $formatted */
        $formatted = $data['formatted'];
        return new self(
            DiagnosticSeverity::from($severity),
            $message,
            $location === null ? null : SourceLocation::fromArray($location),
            $hints,
            array_map(TracePoint::fromArray(...), $trace),
            $formatted
        );
    }

    /**
     * @internal
     *
     * @return list<Diagnostic>
     * @throws \JsonException If the diagnostics cannot be decoded
     */
    public static function listFromJson(string $json): array
    {
        /** @var list<array<string, mixed>> $diagnostics */
        $diagnostics = json_decode($json, true, flags: JSON_THROW_ON_ERROR);
        return array_map(self::fromArray(...), $diagnostics);
    }
}
