<?php

declare(strict_types=1);

namespace Oicana;

/**
 * A range in a file of the template.
 *
 * Lines and columns are 1-based. Columns count characters.
 */
final readonly class SourceLocation
{
    /**
     * @param string $file Path of the file in the template, like `/main.typ`. Files of packages
     *     are prefixed with the package, like `@preview/example:0.1.0/lib.typ`.
     * @param int $endColumn Column of the end of the range (exclusive)
     */
    public function __construct(
        public string $file,
        public int $line,
        public int $column,
        public int $endLine,
        public int $endColumn
    ) {
    }

    /**
     * @param array{file: string, line: int, column: int, endLine: int, endColumn: int} $data
     */
    public static function fromArray(array $data): self
    {
        return new self(
            $data['file'],
            $data['line'],
            $data['column'],
            $data['endLine'],
            $data['endColumn']
        );
    }
}
