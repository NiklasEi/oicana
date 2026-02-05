<?php

declare(strict_types=1);

namespace Oicana\Inputs;

/**
 * Binary blob input with optional metadata.
 *
 * Used for passing images or other binary assets to templates.
 */
final readonly class BlobInput
{
    /**
     * @param string $data Binary data
     * @param array<string, mixed>|null $metadata Optional metadata as associative array
     */
    public function __construct(
        public string $data,
        public ?array $metadata = null
    ) {
    }
}
