import 'dart:typed_data';

/// Decides which fallback values a template uses for inputs without a value.
enum CompilationMode {
  /// Uses the default values of inputs. Required inputs need a value.
  production,

  /// Prefers the development values of inputs over their defaults.
  development,
}

/// A format a document can be exported to: [pdf], [svg] or [ExportFormat.png].
sealed class ExportFormat {
  const ExportFormat();

  /// Exports to PDF. Tagging is turned off when exporting a subset of pages.
  static const ExportFormat pdf = PdfFormat();

  /// Exports to SVG.
  static const ExportFormat svg = SvgFormat();

  /// Exports to PNG with the given resolution in pixels per point.
  ///
  /// Multiple pages are merged into a single, vertically stacked image.
  const factory ExportFormat.png({double pixelsPerPt}) = PngFormat;
}

/// The PDF format, see [ExportFormat.pdf].
final class PdfFormat extends ExportFormat {
  /// The PDF format.
  const PdfFormat();
}

/// The SVG format, see [ExportFormat.svg].
final class SvgFormat extends ExportFormat {
  /// The SVG format.
  const SvgFormat();
}

/// The PNG format, see [ExportFormat.png].
final class PngFormat extends ExportFormat {
  /// The PNG format with the given resolution in pixels per point.
  const PngFormat({this.pixelsPerPt = 1.0});

  /// The resolution in pixels per point.
  final double pixelsPerPt;
}

/// Pages of a document by their 0-based, inclusive indices.
///
/// A missing bound leaves the range open on that side.
final class PageRange {
  /// Selects the pages from [start] to [end], both inclusive.
  const PageRange({this.start, this.end});

  /// Selects only the page at [index].
  const PageRange.single(int index) : start = index, end = index;

  /// The first page, or `null` to start at the first page of the document.
  final int? start;

  /// The last page, or `null` to end at the last page of the document.
  final int? end;
}

/// The size of a page in points.
final class PageSize {
  /// A page size in points.
  const PageSize({required this.width, required this.height});

  /// The width in points.
  final double width;

  /// The height in points.
  final double height;

  @override
  String toString() => 'PageSize($width x $height)';
}

/// Limits for reading a packed template. `null` uses the default of Oicana.
final class ZipLimits {
  /// Limits for reading a packed template.
  const ZipLimits({this.maxEntries, this.maxTotalDecompressedBytes});

  /// The maximum number of files in the template.
  final int? maxEntries;

  /// The maximum size of all files in the template after decompression.
  final int? maxTotalDecompressedBytes;
}

/// The value of a blob input.
final class BlobInput {
  /// A blob input with optional [metadata] for the template.
  const BlobInput(this.data, {this.metadata});

  /// The bytes passed to the template.
  final Uint8List data;

  /// Metadata passed to the template along with the data. It is encoded as JSON
  /// and must encode to an object.
  final Map<String, Object?>? metadata;
}

/// A font face registered by the host.
final class RegisteredFont {
  /// A registered font face.
  const RegisteredFont({required this.family, this.path});

  /// The family name, as used in Typst's `text(font: ...)`.
  final String family;

  /// The file the face was read from, or `null` for fonts registered from memory.
  final String? path;

  @override
  String toString() => path == null ? family : '$family ($path)';
}

/// The coloring of compilation diagnostics in error messages and warnings.
enum DiagnosticColor {
  /// Plain text.
  none,

  /// ANSI escape codes.
  ansi,
}

/// The result of exporting a template once.
final class ExportOnceResult {
  /// The result of exporting a template once.
  const ExportOnceResult({required this.document, required this.warnings});

  /// The exported document.
  final Uint8List document;

  /// The warnings of the compilation.
  final List<Diagnostic> warnings;
}

/// The severity of a [Diagnostic].
enum DiagnosticSeverity {
  /// A fatal error.
  error,

  /// A non-fatal warning.
  warning,
}

/// A compilation diagnostic, like a warning, with its location in the template.
final class Diagnostic {
  /// A compilation diagnostic.
  const Diagnostic({
    required this.severity,
    required this.message,
    required this.location,
    required this.hints,
    required this.trace,
    required this.formatted,
  });

  /// Whether this is an error or a warning.
  final DiagnosticSeverity severity;

  /// Describes the problem.
  final String message;

  /// Where in the template the problem is, or `null` if it does not point into a
  /// file.
  final SourceLocation? location;

  /// Hints on how to address the problem.
  final List<String> hints;

  /// The function calls leading to the problem, innermost first.
  final List<TracePoint> trace;

  /// The diagnostic formatted for humans, with source excerpts and the
  /// configured [DiagnosticColor].
  final String formatted;

  @override
  String toString() => formatted;
}

/// A range in a file of the template.
///
/// Lines and columns are 1-based. Columns count characters.
final class SourceLocation {
  /// A range in a file of the template.
  const SourceLocation({
    required this.file,
    required this.line,
    required this.column,
    required this.endLine,
    required this.endColumn,
  });

  /// The path of the file in the template, like `/main.typ`. Files of packages
  /// are prefixed with the package, like `@preview/example:0.1.0/lib.typ`.
  final String file;

  /// The line the range starts at, 1-based.
  final int line;

  /// The column the range starts at, 1-based.
  final int column;

  /// The line the range ends at, 1-based.
  final int endLine;

  /// The column the range ends at, 1-based and exclusive.
  final int endColumn;

  @override
  String toString() => '$file:$line:$column';
}

/// A step in the function calls leading to a [Diagnostic].
final class TracePoint {
  /// A step in the function calls leading to a diagnostic.
  const TracePoint({required this.message, required this.location});

  /// What happened at this step, like "error occurred in this function call".
  final String message;

  /// Where the step happened, or `null` if it does not point into a file.
  final SourceLocation? location;
}
