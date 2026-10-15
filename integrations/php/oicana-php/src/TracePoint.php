<?php

declare(strict_types=1);

namespace Oicana;

/**
 * A step in the function calls leading to a diagnostic.
 */
final readonly class TracePoint
{
    /**
     * @param string $message What happened at this step, like `error occurred in this function call`
     * @param SourceLocation|null $location Where the step happened, if it points into a file
     */
    public function __construct(
        public string $message,
        public ?SourceLocation $location
    ) {
    }

    /**
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        /** @var array{file: string, line: int, column: int, endLine: int, endColumn: int}|null $location */
        $location = $data['location'] ?? null;
        /** @var string $message */
        $message = $data['message'];
        return new self(
            $message,
            $location === null ? null : SourceLocation::fromArray($location)
        );
    }
}
