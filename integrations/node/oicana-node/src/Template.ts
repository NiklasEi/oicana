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
  private defaultCompilationMode: CompilationMode;

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
   * @param jsonInputs for the initial rendering to warm up the cache
   * @param blobInputs for the initial rendering to warm up the cache
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
   * @param jsonInputs for the initial rendering to warm up the cache
   * @param blobInputs for the initial rendering to warm up the cache
   * @param compilation mode for the initial rendering to warm up the cache
   */
  public constructor(
    name: string,
    template: Uint8Array,
    jsonInputs?: Map<string, string>,
    blobInputs?: Map<string, BlobWithMetadata>,
    compilationMode?: CompilationMode,
  ) {
    this.template = name;
    this.defaultCompilationMode = CompilationMode.Production;

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
   * @param jsonInputs
   * @param blobInputs
   * @param exportFormat
   * @param compilationMode
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

  /**
   * Get the default compilation mode of this template
   */
  public defaultMode(): CompilationMode {
    return this.defaultCompilationMode;
  }

  /**
   * Set the default compilation mode of this template
   * @param compilationMode to use as default when compiling this template
   */
  public setDefaultMode(compilationMode: CompilationMode) {
    this.defaultCompilationMode = compilationMode;
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
