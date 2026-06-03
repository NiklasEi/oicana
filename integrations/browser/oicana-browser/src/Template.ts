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
import type { ExportFormat } from './ExportFormat.js';
import type {
  BlobInputDefinition,
  BlobWithMetadata,
  JsonInputDefinition,
} from './inputs/index.js';

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
   * Compile the template to a PDF file without any inputs in production mode
   */
  public compile(): Uint8Array<ArrayBuffer>;

  /**
   * Compile the template to a PDF file with given inputs in production mode
   * @param jsonInputs
   * @param blobInputs
   */
  public compile(
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
  ): Uint8Array<ArrayBuffer>;

  /**
   * Compile the template with the given inputs
   * @param jsonInputs
   * @param blobInputs
   * @param exportFormat
   */
  public compile(
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
    exportFormat: ExportFormat,
  ): Uint8Array<ArrayBuffer>;

  /**
   * Compile the template with the given inputs
   * @param jsonInputs
   * @param blobInputs
   * @param exportFormat
   * @param compilationOptions
   */
  public compile(
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
    exportFormat: ExportFormat,
    compilationOptions: CompilationMode,
  ): Uint8Array<ArrayBuffer>;

  /**
   * Compile the template with the given inputs
   * @param jsonInputs - JSON inputs for the template (defaults to empty map)
   * @param blobInputs - Blob inputs for the template (defaults to empty map)
   * @param exportFormat - Export format specification (defaults to PDF)
   * @param compilationOptions - Compilation mode (defaults to Production)
   */
  public compile(
    jsonInputs?: Map<string, string>,
    blobInputs?: Map<string, BlobWithMetadata>,
    exportFormat?: ExportFormat,
    compilationOptions?: CompilationMode,
  ): Uint8Array {
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
    const result = export_document(
      documentId,
      exportFormat ?? { format: 'pdf' },
    );
    remove_document(documentId);

    return result;
  }

  /**
   * Warnings produced by the most recent compilation (constructor warm-up or
   * `compile()`), or `undefined` if there were none.
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
