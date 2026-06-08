/**
 * A contiguous, 1-based inclusive range of document pages to export.
 *
 * Both bounds are optional; omitting one leaves it open. For example,
 * `{ start: 2 }` selects page 2 to the end of the document, and an empty object
 * (or omitting the range entirely) selects every page.
 */
export interface PageRange {
  /** First page to export (1-based, inclusive). Omit to start at the first page. */
  start?: number;
  /** Last page to export (1-based, inclusive). Omit to go to the last page. */
  end?: number;
}

export const PageRange = {
  /** A range selecting exactly the given 1-based page. */
  single: (page: number): PageRange => ({ start: page, end: page }),
  /** A range with the given (optional) 1-based, inclusive bounds. */
  of: (start?: number, end?: number): PageRange => ({ start, end }),
};
