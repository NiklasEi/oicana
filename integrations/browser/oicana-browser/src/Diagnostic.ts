/** A compilation diagnostic, like a warning, with its location in the template. */
export interface Diagnostic {
  severity: DiagnosticSeverity;
  /** The message describing the problem. */
  message: string;
  /** Where in the template the problem is, if it points into a file. */
  location?: SourceLocation;
  /** Hints on how to address the problem. */
  hints: string[];
  /** The function calls leading to the problem, innermost first. */
  trace: TracePoint[];
  /**
   * The diagnostic formatted for humans, with source excerpts and the
   * configured diagnostic color.
   */
  formatted: string;
}

/** The severity of a {@link Diagnostic}. */
export type DiagnosticSeverity = 'error' | 'warning';

/**
 * A range in a file of the template.
 *
 * Lines and columns are 1-based. Columns count characters.
 */
export interface SourceLocation {
  /**
   * Path of the file in the template, like `/main.typ`. Files of packages are
   * prefixed with the package, like `@preview/example:0.1.0/lib.typ`.
   */
  file: string;
  line: number;
  column: number;
  endLine: number;
  /** Column of the end of the range (exclusive). */
  endColumn: number;
}

/** A step in the function calls leading to a {@link Diagnostic}. */
export interface TracePoint {
  /** What happened at this step, like `error occurred in this function call`. */
  message: string;
  /** Where the step happened, if it points into a file. */
  location?: SourceLocation;
}
