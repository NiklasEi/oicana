import { randomUUID } from 'node:crypto';
import {
  type BlobWithMetadata as BlobWithMetadataNative,
  compileTemplate,
  exportDocument,
  getWarnings,
  CompilationMode as NativeCompilationMode,
  registerTemplate,
  removeDocument,
  removeWorld,
  setValidateInputs,
} from '@oicana/node-native';
import { CompilationMode } from './CompilationMode.js';
import { CompiledDocument } from './CompiledDocument.js';
import { type ExportFormat, Pdf, Png, Svg } from './ExportFormat.js';
import type { BlobWithMetadata } from './inputs/index.js';
import { type PageRange, serializePageRange } from './PageRange.js';

/**
 * A template
 *
 * The zip file is loaded during the instance creation and cached afterward.
 */
export class Template implements Disposable {
  private readonly template: string;
  private lastWarnings: string | undefined;

  /**
   * Register a template with the given template file
   * @param template - the packed Oicana template file
   */
  public constructor(template: Uint8Array);

  /**
   * Register a template with the given template file and inputs
   * @param template - the packed Oicana template file
   * @param jsonInputs for the initial compilation to warm up the cache
   * @param blobInputs for the initial compilation to warm up the cache
   */
  public constructor(
    template: Uint8Array,
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
  );

  /**
   * Register a template with the given template file and inputs
   * @param template - the packed Oicana template file
   * @param jsonInputs for the initial compilation to warm up the cache (defaults to empty map)
   * @param blobInputs for the initial compilation to warm up the cache (defaults to empty map)
   * @param compilationOptions for the initial compilation to warm up the cache (defaults to Development)
   */
  public constructor(
    template: Uint8Array,
    jsonInputs?: Map<string, string>,
    blobInputs?: Map<string, BlobWithMetadata>,
    compilationOptions?: CompilationMode,
  ) {
    this.template = randomUUID();

    const documentId = registerTemplate(
      this.template,
      template,
      Object.fromEntries(jsonInputs ?? new Map<string, string>()),
      this.convertBlobWithMetadata(
        blobInputs ?? new Map<string, BlobWithMetadata>(),
      ),
      this.mapCompilationMode(
        compilationOptions ?? CompilationMode.Development,
      ),
    );
    this.lastWarnings = getWarnings(documentId) ?? undefined;
    removeDocument(documentId);
  }

  /**
   * Compile the template and export it to a PDF file, without inputs, in
   * production mode.
   */
  public export(): Uint8Array;

  /**
   * Compile the template with the given inputs and export it to a PDF file in
   * production mode.
   * @param jsonInputs
   * @param blobInputs
   */
  public export(
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
  ): Uint8Array;

  /**
   * Compile the template with the given inputs and export it in the given format.
   * @param jsonInputs
   * @param blobInputs
   * @param exportOptions
   */
  public export(
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
    exportOptions: ExportFormat,
  ): Uint8Array;

  /**
   * Compile the template with the given inputs and export it in the given format.
   * @param jsonInputs
   * @param blobInputs
   * @param exportOptions
   * @param compilationOptions
   */
  public export(
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
    exportOptions: ExportFormat,
    compilationOptions: CompilationMode,
  ): Uint8Array;

  /**
   * Compile the template with the given inputs and export a range of pages
   * @param jsonInputs
   * @param blobInputs
   * @param exportOptions
   * @param compilationOptions
   * @param pages
   */
  public export(
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
    exportOptions: ExportFormat,
    compilationOptions: CompilationMode,
    pages: PageRange,
  ): Uint8Array;

  /**
   * Compile the template and export it in a single call, then free the document.
   *
   * To export the document
   * several times (multiple formats, page ranges, or individual pages) from a
   * single compilation, use {@link compile} and call `export` on the returned
   * {@link CompiledDocument}.
   * @param jsonInputs - JSON inputs for the template (defaults to empty map)
   * @param blobInputs - Blob inputs for the template (defaults to empty map)
   * @param exportOptions - Export format specification (defaults to PDF)
   * @param compilationOptions - Compilation mode (defaults to Production)
   * @param pages - 0-based, inclusive page range (defaults to the whole document)
   */
  public export(
    jsonInputs?: Map<string, string>,
    blobInputs?: Map<string, BlobWithMetadata>,
    exportOptions?: ExportFormat,
    compilationOptions?: CompilationMode,
    pages?: PageRange,
  ): Uint8Array {
    return this.exportWith(
      exportOptions ?? Pdf,
      jsonInputs,
      blobInputs,
      compilationOptions,
      pages,
    );
  }

  /**
   * Compile the template and export it to PDF in a single call, then free the
   * document.
   * Tagging will be automatically turned off when exporting a subset of pages.
   * @param jsonInputs - JSON inputs for the template (defaults to empty map)
   * @param blobInputs - Blob inputs for the template (defaults to empty map)
   * @param compilationOptions - Compilation mode (defaults to Production)
   * @param pages - 0-based, inclusive page range (defaults to the whole document)
   */
  public exportPdf(
    jsonInputs?: Map<string, string>,
    blobInputs?: Map<string, BlobWithMetadata>,
    compilationOptions?: CompilationMode,
    pages?: PageRange,
  ): Uint8Array {
    return this.exportWith(
      Pdf,
      jsonInputs,
      blobInputs,
      compilationOptions,
      pages,
    );
  }

  /**
   * Compile the template and export it to PNG in a single call, then free the
   * document.
   * Multiple pages are merged into a single, vertically stacked image.
   * @param jsonInputs - JSON inputs for the template (defaults to empty map)
   * @param blobInputs - Blob inputs for the template (defaults to empty map)
   * @param compilationOptions - Compilation mode (defaults to Production)
   * @param pixelsPerPt - resolution in pixels per point (defaults to 1.0)
   * @param pages - 0-based, inclusive page range (defaults to the whole document)
   */
  public exportPng(
    jsonInputs?: Map<string, string>,
    blobInputs?: Map<string, BlobWithMetadata>,
    compilationOptions?: CompilationMode,
    pixelsPerPt = 1.0,
    pages?: PageRange,
  ): Uint8Array {
    return this.exportWith(
      Png(pixelsPerPt),
      jsonInputs,
      blobInputs,
      compilationOptions,
      pages,
    );
  }

  /**
   * Compile the template and export it to SVG in a single call, then free the
   * document.
   * @param jsonInputs - JSON inputs for the template (defaults to empty map)
   * @param blobInputs - Blob inputs for the template (defaults to empty map)
   * @param compilationOptions - Compilation mode (defaults to Production)
   * @param pages - 0-based, inclusive page range (defaults to the whole document)
   */
  public exportSvg(
    jsonInputs?: Map<string, string>,
    blobInputs?: Map<string, BlobWithMetadata>,
    compilationOptions?: CompilationMode,
    pages?: PageRange,
  ): Uint8Array {
    return this.exportWith(
      Svg,
      jsonInputs,
      blobInputs,
      compilationOptions,
      pages,
    );
  }

  private exportWith(
    format: ExportFormat,
    jsonInputs?: Map<string, string>,
    blobInputs?: Map<string, BlobWithMetadata>,
    compilationOptions?: CompilationMode,
    pages?: PageRange,
  ): Uint8Array {
    const document = this.compileToDocumentId(
      jsonInputs,
      blobInputs,
      compilationOptions,
    );
    try {
      return exportDocument(
        document,
        JSON.stringify(format),
        serializePageRange(pages),
      );
    } finally {
      removeDocument(document);
    }
  }

  /**
   * Compile the template and return a handle to the compiled document.
   *
   * The document is kept in memory so it can be exported one or more times
   * (whole document, a page range, or individual pages) without re-compiling.
   * Call `dispose()` on the returned document (or use `using`) to free it. For a
   * single one-shot export, prefer {@link export}.
   * @param jsonInputs - JSON inputs for the template (defaults to empty map)
   * @param blobInputs - Blob inputs for the template (defaults to empty map)
   * @param compilationOptions - Compilation mode (defaults to Production)
   */
  public compile(
    jsonInputs?: Map<string, string>,
    blobInputs?: Map<string, BlobWithMetadata>,
    compilationOptions?: CompilationMode,
  ): CompiledDocument {
    const documentId = this.compileToDocumentId(
      jsonInputs,
      blobInputs,
      compilationOptions,
    );
    return new CompiledDocument(documentId);
  }

  private compileToDocumentId(
    jsonInputs?: Map<string, string>,
    blobInputs?: Map<string, BlobWithMetadata>,
    compilationOptions?: CompilationMode,
  ): string {
    const documentId = compileTemplate(
      this.template,
      Object.fromEntries(jsonInputs ?? new Map<string, string>()),
      this.convertBlobWithMetadata(
        blobInputs ?? new Map<string, BlobWithMetadata>(),
      ),
      this.mapCompilationMode(compilationOptions ?? CompilationMode.Production),
    );
    this.lastWarnings = getWarnings(documentId) ?? undefined;
    return documentId;
  }

  /**
   * Warnings produced by the most recent compilation (constructor warm-up, or a
   * `compile()` / `export()` call), or `undefined` if there were none.
   */
  public warnings(): string | undefined {
    return this.lastWarnings;
  }

  /**
   * Enable or disable JSON schema validation for this template.
   *
   * When enabled (the default), JSON inputs are validated against their schemas
   * before compilation.
   * @param validate - whether to validate inputs against their JSON schemas
   */
  public setValidateInputs(validate: boolean): void {
    setValidateInputs(this.template, validate);
  }

  /**
   * Release resources associated with this template.
   * After calling dispose(), this template instance should not be used.
   */
  public dispose(): void {
    removeWorld(this.template);
  }

  /**
   * Enables use with the `using` keyword for automatic resource cleanup.
   */
  [Symbol.dispose](): void {
    this.dispose();
  }

  private convertBlobWithMetadata(
    blobInputs: Map<string, BlobWithMetadata>,
  ): Record<string, BlobWithMetadataNative> {
    return Object.fromEntries(
      Array.from(blobInputs.entries(), ([key, value]) => {
        const nativeValue = {
          bytes: value.bytes,
          meta: value.meta === undefined ? '{}' : JSON.stringify(value.meta),
        };
        return [key, nativeValue];
      }),
    );
  }

  private mapCompilationMode(mode: CompilationMode): NativeCompilationMode {
    switch (mode) {
      case CompilationMode.Development:
        return NativeCompilationMode.Development;
      case CompilationMode.Production:
        return NativeCompilationMode.Production;
    }
  }
}
