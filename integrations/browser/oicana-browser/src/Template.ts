import {
  compile_template,
  export_document,
  get_file,
  get_source,
  get_warnings,
  register_template,
  remove_document,
  remove_world,
  set_validate_inputs,
  inputs as wasmInputs,
} from '@oicana/browser-wasm';
import { CompilationMode } from './CompilationMode.js';
import { CompiledDocument } from './CompiledDocument.js';
import { type ExportFormat, Pdf, Png, Svg } from './ExportFormat.js';
import type {
  BlobInputDefinition,
  BlobWithMetadata,
  JsonInputDefinition,
} from './inputs/index.js';
import type { PageRange } from './PageRange.js';

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
    this.template = crypto.randomUUID();
    for (const blob of blobInputs?.entries() ?? []) {
      if (blob[1].meta === undefined) {
        // Otherwise the FFI layer will fail to pass the blobs over to WASM
        blob[1].meta = {};
      }
    }
    const documentId = register_template(
      this.template,
      template,
      jsonInputs ?? new Map(),
      blobInputs ?? new Map(),
      compilationOptions ?? CompilationMode.Development,
    );
    this.lastWarnings = get_warnings(documentId);
    remove_document(documentId);
  }

  /**
   * Compile the template and export it to a PDF file, without inputs, in
   * production mode.
   */
  public export(): Uint8Array<ArrayBuffer>;

  /**
   * Compile the template with the given inputs and export it to a PDF file in
   * production mode.
   * @param jsonInputs
   * @param blobInputs
   */
  public export(
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
  ): Uint8Array<ArrayBuffer>;

  /**
   * Compile the template with the given inputs and export it in the given format.
   * @param jsonInputs
   * @param blobInputs
   * @param exportFormat
   */
  public export(
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
    exportFormat: ExportFormat,
  ): Uint8Array<ArrayBuffer>;

  /**
   * Compile the template with the given inputs and export it in the given format.
   * @param jsonInputs
   * @param blobInputs
   * @param exportFormat
   * @param compilationOptions
   */
  public export(
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
    exportFormat: ExportFormat,
    compilationOptions: CompilationMode,
  ): Uint8Array<ArrayBuffer>;

  /**
   * Compile the template with the given inputs and export a range of pages
   * @param jsonInputs
   * @param blobInputs
   * @param exportFormat
   * @param compilationOptions
   * @param pages
   */
  public export(
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
    exportFormat: ExportFormat,
    compilationOptions: CompilationMode,
    pages: PageRange,
  ): Uint8Array<ArrayBuffer>;

  /**
   * Compile the template and export it.
   *
   * This is the one-shot path: the compiled document is not kept. To export it
   * several times (multiple formats, page ranges, or individual pages) from a
   * single compilation, use {@link compile} and call `export` on the returned
   * {@link CompiledDocument}.
   * @param jsonInputs - JSON inputs for the template (defaults to empty map)
   * @param blobInputs - Blob inputs for the template (defaults to empty map)
   * @param exportFormat - Export format specification (defaults to PDF)
   * @param compilationOptions - Compilation mode (defaults to Production)
   * @param pages - 0-based, inclusive page range (defaults to the whole document)
   */
  public export(
    jsonInputs?: Map<string, string>,
    blobInputs?: Map<string, BlobWithMetadata>,
    exportFormat?: ExportFormat,
    compilationOptions?: CompilationMode,
    pages?: PageRange,
  ): Uint8Array {
    return this.exportWith(
      exportFormat ?? Pdf,
      jsonInputs,
      blobInputs,
      compilationOptions,
      pages,
    );
  }

  /**
   * Compile the template and export it to PDF in a single call, then free the
   * document.
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
    const documentId = this.compileToDocumentId(
      jsonInputs,
      blobInputs,
      compilationOptions,
    );
    try {
      return export_document(documentId, format, pages);
    } finally {
      remove_document(documentId);
    }
  }

  /**
   * Compile the template.
   *
   * The document is kept in memory so it can be exported one or more times
   * (whole document, a page range, or individual pages) without re-compiling.
   * Call `dispose()` on the returned document (or use `using`) to free it.
   * For a single one-shot export, prefer {@link export}.
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
    for (const blob of blobInputs instanceof Map
      ? (blobInputs?.entries() ?? [])
      : Object.entries<BlobWithMetadata>(blobInputs ?? {})) {
      if (blob[1].meta === undefined) {
        // Otherwise the FFI layer will fail to pass the blobs over to WASM
        blob[1].meta = {};
      }
    }
    const documentId = compile_template(
      this.template,
      jsonInputs ?? new Map(),
      blobInputs ?? new Map(),
      compilationOptions ?? CompilationMode.Production,
    );
    this.lastWarnings = get_warnings(documentId);
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
   * Gather all input definitions of this template
   */
  public inputs(): { inputs: (BlobInputDefinition | JsonInputDefinition)[] } {
    return JSON.parse(wasmInputs(this.template));
  }

  /**
   * Get the string content of a file from the template
   */
  public source(path: string): string {
    return get_source(this.template, path);
  }

  /**
   * Get the raw file from the template
   */
  public file(path: string): Uint8Array {
    return get_file(this.template, path);
  }

  /**
   * Enable or disable JSON schema validation for this template.
   *
   * When enabled (the default), JSON inputs are validated against their schemas
   * before compilation.
   * @param validate - whether to validate inputs against their JSON schemas
   */
  public setValidateInputs(validate: boolean): void {
    set_validate_inputs(this.template, validate);
  }

  /**
   * Release resources associated with this template.
   * After calling dispose(), this template instance should not be used.
   */
  public dispose(): void {
    remove_world(this.template);
  }

  /**
   * Enables use with the `using` keyword for automatic resource cleanup.
   */
  [Symbol.dispose](): void {
    this.dispose();
  }
}
