import { randomUUID } from 'node:crypto';
import {
  type BlobWithMetadata as BlobWithMetadataNative,
  compileTemplate,
  evictCache,
  exportDocument,
  CompilationMode as NativeCompilationMode,
  registerTemplate,
  removeDocument,
} from '@oicana/node-native';
import { CompilationMode } from './CompilationMode.js';
import type { ExportFormat } from './ExportFormat.js';
import type { BlobWithMetadata } from './inputs/index.js';

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
    this.template = randomUUID();

    registerTemplate(
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
    evictCache();
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
    const format: ExportFormat = exportOptions ?? { format: 'pdf' };

    const document = compileTemplate(
      this.template,
      Object.fromEntries(jsonInputs ?? new Map<string, string>()),
      this.convertBlobWithMetadata(
        blobInputs ?? new Map<string, BlobWithMetadata>(),
      ),
      this.mapCompilationMode(compilationOptions ?? CompilationMode.Production),
    );
    try {
      const exportedDocument = exportDocument(document, JSON.stringify(format));
      evictCache();
      return exportedDocument;
    } finally {
      removeDocument(document);
    }
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
