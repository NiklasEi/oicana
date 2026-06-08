import {
  document_pages,
  export_document,
  remove_document,
} from '@oicana/browser-wasm';
import { type ExportFormat, Png } from './ExportFormat.js';
import type { PageRange } from './PageRange.js';

/**
 * Size of a single document page, in typographic points (pt).
 */
export interface PageSize {
  /** Page width in points. */
  width: number;
  /** Page height in points. */
  height: number;
}

/**
 * A compiled document. Its pages can be exported on
 * demand without re-compiling.
 *
 * Obtain one via {@link Template.compile}. Call {@link dispose} (or
 * use the `using` keyword) to release the underlying document.
 */
export class CompiledDocument implements Disposable {
  private documentId: string | undefined;

  /** Sizes (in points) of every page, in document order. */
  public readonly pages: ReadonlyArray<PageSize>;

  /**
   * @internal Construct via {@link Template.compile}.
   */
  public constructor(documentId: string) {
    this.documentId = documentId;
    this.pages = JSON.parse(document_pages(documentId)) as PageSize[];
  }

  /** Number of pages in the document. */
  public get pageCount(): number {
    return this.pages.length;
  }

  /**
   * Export a single page of the document to PNG.
   * @param pageIndex - zero-based index of the page to export
   * @param pixelsPerPt - resolution in pixels per point
   */
  public exportPage(pageIndex: number, pixelsPerPt: number): Uint8Array {
    return this.export(Png(pixelsPerPt), {
      start: pageIndex + 1,
      end: pageIndex + 1,
    });
  }

  /**
   * Export the document in the given format (defaults to PDF), optionally
   * restricted to a range of pages.
   * @param format - export format specification
   * @param pages - 1-based, inclusive page range (defaults to the whole document)
   */
  public export(
    format: ExportFormat = { format: 'pdf' },
    pages?: PageRange,
  ): Uint8Array {
    if (this.documentId === undefined) {
      throw new Error('CompiledDocument has already been disposed');
    }
    return export_document(this.documentId, format, pages);
  }

  /**
   * Export the document to a PDF file, optionally restricted to a range of pages.
   * @param pages - 1-based, inclusive page range (defaults to the whole document)
   */
  public toPdf(pages?: PageRange): Uint8Array {
    return this.export({ format: 'pdf' }, pages);
  }

  /**
   * Release the cached document. After calling dispose() this instance must not
   * be used anymore.
   */
  public dispose(): void {
    if (this.documentId !== undefined) {
      remove_document(this.documentId);
      this.documentId = undefined;
    }
  }

  /**
   * Enables use with the `using` keyword for automatic resource cleanup.
   */
  [Symbol.dispose](): void {
    this.dispose();
  }
}
