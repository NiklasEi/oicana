package com.oicana;

/**
 * A range in a file of the template.
 *
 * <p>Lines and columns are 1-based. Columns count characters.
 *
 * @param file      the path of the file in the template, like {@code /main.typ}. Files of
 *                  packages are prefixed with the package, like
 *                  {@code @preview/example:0.1.0/lib.typ}
 * @param line      the line of the start of the range
 * @param column    the column of the start of the range
 * @param endLine   the line of the end of the range
 * @param endColumn the column of the end of the range (exclusive)
 */
public record SourceLocation(String file, int line, int column, int endLine, int endColumn) {
}
