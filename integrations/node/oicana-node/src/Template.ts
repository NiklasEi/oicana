import {
  type BlobWithMetadata as BlobWithMetadataNative,
  compileTemplate,
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

    registerTemplate(
      this.template,
      template,
      Object.fromEntries(jsonInputs ?? new Map<string, string>()),
      this.convertBlobWithMetadata(
        blobInputs ?? new Map<string, BlobWithMetadata>(),
      ),
      this.mapCompilationMode(compilationMode ?? CompilationMode.Development),
    );
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
    const format: ExportFormat = exportFormat ?? { format: 'pdf' };

    const document = compileTemplate(
      this.template,
      Object.fromEntries(jsonInputs ?? new Map<string, string>()),
      this.convertBlobWithMetadata(
        blobInputs ?? new Map<string, BlobWithMetadata>(),
      ),
      this.mapCompilationMode(compilationMode ?? CompilationMode.Production),
    );
    try {
      return exportDocument(document, JSON.stringify(format));
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
