import {
  compile_template,
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
   * Register a template with the given name and template file
   * @param name of the template
   * @param template - the packed Oicana template file
   */
  public constructor(name: string, template: Uint8Array);

  /**
   * Register a template with the given name, template file, and inputs
   * @param name of the template
   * @param template - the packed Oicana template file
   * @param jsonInputs for the initial compilation to warm up the cache
   * @param blobInputs for the initial compilation to warm up the cache
   */
  public constructor(
    name: string,
    template: Uint8Array,
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
  );

  /**
   * Register a template with the given name, template file, and inputs
   * @param name of the template
   * @param template - the packed Oicana template file
   * @param jsonInputs for the initial compilation to warm up the cache (defaults to empty map)
   * @param blobInputs for the initial compilation to warm up the cache (defaults to empty map)
   * @param compilationMode for the initial compilation to warm up the cache (defaults to Development)
   */
  public constructor(
    name: string,
    template: Uint8Array,
    jsonInputs?: Map<string, string>,
    blobInputs?: Map<string, BlobWithMetadata>,
    compilationMode?: CompilationMode,
  ) {
    this.template = name;
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
      compilationMode ?? CompilationMode.Development,
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
   * @param exportFormat
   */
  public compile(
    jsonInputs: Map<string, string>,
    blobInputs: Map<string, BlobWithMetadata>,
    exportFormat: ExportFormat,
  ): Uint8Array;

  /**
   * Compile the template with the given inputs
   * @param jsonInputs - JSON inputs for the template (defaults to empty map)
   * @param blobInputs - Blob inputs for the template (defaults to empty map)
   * @param exportFormat - Export format specification (defaults to PDF)
   * @param compilationMode - Compilation mode (defaults to Production)
   */
  public compile(
    jsonInputs?: Map<string, string>,
    blobInputs?: Map<string, BlobWithMetadata>,
    exportFormat?: ExportFormat,
    compilationMode?: CompilationMode,
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
      compilationMode ?? CompilationMode.Production,
    );
    const result = export_document(
      documentId,
      this.convertExportFormat(exportFormat),
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
