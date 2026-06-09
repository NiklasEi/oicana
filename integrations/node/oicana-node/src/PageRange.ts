/**
 * A contiguous, 0-based inclusive range of document pages to export.
 *
 * Both bounds are optional; omitting one leaves it open. For example,
 * `{ start: 1 }` selects the second page to the end of the document, and an empty
 * object (or omitting the range entirely) selects every page.
 */
export interface PageRange {
  /** First page to export (0-based, inclusive). Omit to start at the first page. */
  start?: number;
  /** Last page to export (0-based, inclusive). Omit to go to the last page. */
  end?: number;
}

export const PageRange = {
  /** A range selecting exactly the page at the given 0-based index. */
  single: (page: number): PageRange => ({ start: page, end: page }),
  /** A range with the given (optional) 0-based, inclusive bounds. */
  of: (start?: number, end?: number): PageRange => ({ start, end }),
};

/**
 * Serialize a page range for the native `exportDocument` call. A missing range
 * (`undefined` or `null`) becomes an empty string, meaning "the whole document";
 * the native layer only understands the empty-string sentinel, not `"null"`.
 * @internal
 */
export function serializePageRange(pages?: PageRange | null): string {
  return pages === undefined || pages === null ? '' : JSON.stringify(pages);
}
