namespace Oicana;

/// <summary>
/// A range in a file of the template.
///
/// Lines and columns are 1-based. Columns count characters.
/// </summary>
/// <param name="File">Path of the file in the template, like <c>/main.typ</c>. Files of packages are prefixed with the package, like <c>@preview/example:0.1.0/lib.typ</c>.</param>
/// <param name="Line">Line of the start of the range.</param>
/// <param name="Column">Column of the start of the range.</param>
/// <param name="EndLine">Line of the end of the range.</param>
/// <param name="EndColumn">Column of the end of the range (exclusive).</param>
public sealed record SourceLocation(string File, int Line, int Column, int EndLine, int EndColumn);
