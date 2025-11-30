import {
  compile_template,
  evict_cache,
  export_document,
  get_file,
  get_source,
  register_template,
  remove_document,
  inputs as wasmInputs,
} from '@oicana/browser-wasm';
import { CompilationMode } from './CompilationMode';
import type { ExportFormat } from './ExportFormat';
import type {
  BlobInputDefinition,
  BlobWithMetadata,
  JsonInputDefinition,
} from './inputs';

/**
 * A template
 *
 * The zip file is loaded during the instance creation and cached afterward.
 */
export class Template {
  private readonly template: string;

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
    remove_document(documentId);
  }

  /**
   * Compile the template to a PDF file without any inputs in production mode
   */
  public compile(): Uint8Array;

  /**
   * Compile the template to a PDF file with given inputs in production mode
   * @param jsonInputs
   * @param blobInputs
   */
  public compile(
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
  ): Uint8Array;

  /**
   * Compile the template with the given inputs
   * @param jsonInputs
   * @param blobInputs
   * @param exportOptions
   */
  public compile(
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
    exportOptions: ExportFormat,
  ): Uint8Array;

  /**
   * Compile the template with the given inputs
   * @param jsonInputs
   * @param blobInputs
   * @param exportOptions
   * @param compilationOptions
   */
  public compile(
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
    exportOptions: ExportFormat,
    compilationOptions: CompilationMode,
  ): Uint8Array;

  /**
   * Compile the template with the given inputs
   * @param jsonInputs - JSON inputs for the template (defaults to empty map)
   * @param blobInputs - Blob inputs for the template (defaults to empty map)
   * @param exportOptions - Export format specification (defaults to PDF)
   * @param compilationOptions - Compilation mode (defaults to Production)
   */
  public compile(
    jsonInputs?: Map<string, string>,
    blobInputs?: Map<string, BlobWithMetadata>,
    exportOptions?: ExportFormat,
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
    evict_cache();
    const result = export_document(
      documentId,
      this.convertExportFormat(exportOptions),
    );
    remove_document(documentId);

    return result;
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

  private convertExportFormat(exportFormat?: ExportFormat): InnerExportFormat {
    if (exportFormat === undefined) return { format: 'pdf' };
    let exportFormatInner: InnerExportFormat;
    if (exportFormat.format === 'png') {
      exportFormatInner = {
        format: 'png',
        pixels_per_pt: exportFormat.pixelsPerPt,
      };
    } else {
      exportFormatInner = { format: exportFormat.format };
    }

    return exportFormatInner;
  }
}

type InnerExportFormat =
  | { format: 'pdf' | 'svg' }
  | { format: 'png'; pixels_per_pt: number };
