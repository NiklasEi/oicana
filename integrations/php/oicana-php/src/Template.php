<?php

declare(strict_types=1);

namespace Oicana;

use Oicana\Inputs\BlobInput;

/**
 * Oicana template for PDF generation.
 *
 * This class provides an idiomatic PHP interface to the Oicana native extension.
 * Templates are based on Typst and can generate PDFs, PNGs, and SVGs.
 *
 * Example:
 * ```php
 * $template = new Template($templateBytes);
 * try {
 *     $pdf = $template->compile(
 *         jsonInputs: ['name' => '{"value": "Alice"}'],
 *         exportFormat: ExportFormat::pdf()
 *     );
 *     file_put_contents('output.pdf', $pdf);
 * } finally {
 *     $template->cleanup();
 * }
 * ```
 */
class Template
{
    private string $templateId;
    /** @var list<string> */
    private array $documentIds = [];

    /**
     * Initialize template and register it with the native extension.
     *
     * @param string $template Template zip file bytes
     * @param array<string, string> $jsonInputs JSON inputs (key => JSON string)
     * @param array<string, BlobInput> $blobInputs blob inputs
     * @param CompilationMode $mode Compilation mode
     * @throws \Exception If template registration fails
     */
    public function __construct(
        string $template,
        array $jsonInputs = [],
        array $blobInputs = [],
        CompilationMode $mode = CompilationMode::Development
    ) {
        $this->templateId = uniqid('template_', true);

        $nativeBlobs = $this->prepareBlobInputs($blobInputs);
        $templateBytes = $this->stringToBytes($template);

        $docId = \OicanaInternal\register_template(
            $this->templateId,
            $templateBytes,
            $jsonInputs,
            $nativeBlobs,
            $mode->toNative()
        );

        \OicanaInternal\remove_document($docId);
    }

    /**
     * Compile template and export to the given format.
     *
     * @param array<string, string> $jsonInputs JSON inputs (key => JSON string)
     * @param array<string, BlobInput> $blobInputs Blob inputs
     * @param ExportFormat|null $exportFormat Export format configuration (defaults to PDF)
     * @param CompilationMode $mode Compilation mode
     * @return string Compiled document bytes (PDF, PNG, or SVG)
     * @throws \Exception If compilation or export fails
     */
    public function compile(
        array $jsonInputs = [],
        array $blobInputs = [],
        ?ExportFormat $exportFormat = null,
        CompilationMode $mode = CompilationMode::Production
    ): string {
        $formatArray = ($exportFormat ?? ExportFormat::pdf())->toArray();

        $nativeBlobs = $this->prepareBlobInputs($blobInputs);

        $docId = \OicanaInternal\compile_template(
            $this->templateId,
            $jsonInputs,
            $nativeBlobs,
            $mode->toNative()
        );

        $this->documentIds[] = $docId;

        try {
            $bytes = \OicanaInternal\export_document($docId, json_encode($formatArray));
            return pack('C*', ...$bytes);
        } finally {
            \OicanaInternal\remove_document($docId);
            $this->documentIds = array_filter(
                $this->documentIds,
                static fn(string $id): bool => $id !== $docId
            );
        }
    }

    /**
     * Compile the given template once without caching.
     *
     * This is a convenience method for one-off compilations where you don't need
     * to reuse the template. For multiple compilations with the same template,
     * create an instance of Template and use compile() instead.
     *
     * @param string $templateBytes Template zip file bytes
     * @param array<string, string> $jsonInputs JSON inputs (key => JSON string)
     * @param array<string, BlobInput> $blobInputs Blob inputs
     * @param ExportFormat|null $exportFormat Export format configuration (defaults to PDF)
     * @param CompilationMode $mode Compilation mode
     * @return string Compiled document bytes (PDF, PNG, or SVG)
     * @throws \Exception If compilation or export fails
     */
    public static function compileOnce(
        string $templateBytes,
        array $jsonInputs = [],
        array $blobInputs = [],
        ?ExportFormat $exportFormat = null,
        CompilationMode $mode = CompilationMode::Production
    ): string {
        $template = new self($templateBytes, mode: CompilationMode::Development);

        try {
            return $template->compile($jsonInputs, $blobInputs, $exportFormat, $mode);
        } finally {
            $template->cleanup();
        }
    }

    /**
     * Get input definitions from template manifest.
     *
     * Returns the input schema defined in the template's typst.toml file.
     *
     * @return array<string, mixed> Input definitions
     * @throws \Exception If template is not registered
     */
    public function inputs(): array
    {
        $inputsJson = \OicanaInternal\inputs($this->templateId);
        return json_decode($inputsJson, true);
    }

    /**
     * Get source file content from template.
     *
     * @param string $path File path within the template
     * @return string Source code as string
     * @throws \Exception If file not found or template not registered
     */
    public function source(string $path): string
    {
        return \OicanaInternal\get_source($this->templateId, $path);
    }

    /**
     * Get binary file content from template.
     *
     * @param string $path File path within the template
     * @return string Binary file content
     * @throws \Exception If file not found or template not registered
     */
    public function file(string $path): string
    {
        $bytes = \OicanaInternal\get_file($this->templateId, $path);
        return pack('C*', ...$bytes);
    }

    /**
     * Clean up cached resources.
     *
     * This should be called when you're done with the template to free memory.
     * It's automatically called by the destructor, but explicit cleanup is recommended.
     */
    public function cleanup(): void
    {
        foreach ($this->documentIds as $docId) {
            \OicanaInternal\remove_document($docId);
        }
        $this->documentIds = [];

        \OicanaInternal\remove_world($this->templateId);
    }

    /**
     * Destructor ensures cleanup even if not called explicitly.
     */
    public function __destruct()
    {
        try {
            $this->cleanup();
        } catch (\Throwable $e) {
            // Best effort cleanup - don't throw in destructor
        }
    }

    /**
     * Convert BlobInput objects to native BlobWithMetadata objects.
     *
     * @param array<string, BlobInput> $blobInputs
     * @return array<string, \OicanaInternal\BlobWithMetadata>
     */
    private function prepareBlobInputs(array $blobInputs): array
    {
        $nativeBlobs = [];
        foreach ($blobInputs as $key => $blob) {
            $meta = $blob->metadata !== null
                ? json_encode($blob->metadata)
                : '{}';
            $bytes = $this->stringToBytes($blob->data);
            $nativeBlobs[$key] = new \OicanaInternal\BlobWithMetadata($bytes, $meta);
        }
        return $nativeBlobs;
    }

    /**
     * Convert a string to a byte array for the native extension.
     *
     * @param string $data
     * @return list<int>
     */
    private function stringToBytes(string $data): array
    {
        $unpacked = unpack('C*', $data);
        if ($unpacked === false) {
            return [];
        }
        return array_values($unpacked);
    }
}
