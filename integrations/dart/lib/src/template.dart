import 'dart:typed_data';

import 'backend.dart';
import 'manifest.dart';
import 'manifest_json.dart';
import 'platform.dart';
import 'types.dart';

/// A packed Oicana template, ready to be compiled repeatedly.
///
/// Load templates once, not for every document. Call [dispose] when the
/// template is no longer needed; templates that are garbage collected are freed
/// as well. Documents compiled from a template stay usable after it is
/// disposed.
///
/// Methods without the `Sync` suffix run on a separate isolate on native
/// targets, so they do not block the calling isolate, for example the UI of a
/// Flutter app. On the web, all methods run on the calling thread.
class Template {
  Template._(this._backend);

  TemplateBackend? _backend;

  /// Loads a packed template and compiles it once to warm up its caches.
  ///
  /// The warm-up compilation uses [jsonInputs], [blobInputs] and [mode]. A
  /// [String] value in [jsonInputs] is taken as a JSON document; other values
  /// are encoded as JSON.
  static Future<Template> load(
    Uint8List template, {
    Map<String, Object?> jsonInputs = const {},
    Map<String, BlobInput> blobInputs = const {},
    CompilationMode mode = CompilationMode.development,
    ZipLimits? limits,
  }) async {
    final inputs = EncodedInputs(jsonInputs, blobInputs);
    return Template._(
      await backend.loadTemplateAsync(template, inputs, mode, limits),
    );
  }

  /// Like [load], but blocks until the template is loaded.
  factory Template.loadSync(
    Uint8List template, {
    Map<String, Object?> jsonInputs = const {},
    Map<String, BlobInput> blobInputs = const {},
    CompilationMode mode = CompilationMode.development,
    ZipLimits? limits,
  }) {
    final inputs = EncodedInputs(jsonInputs, blobInputs);
    return Template._(backend.loadTemplate(template, inputs, mode, limits));
  }

  TemplateBackend get _template =>
      _backend ?? (throw StateError('The template is already disposed'));

  /// The warnings of the warm-up compilation.
  List<Diagnostic> get warnings => _template.warnings;

  /// Compiles the template into a document whose pages can be exported
  /// repeatedly.
  Future<CompiledDocument> compile({
    Map<String, Object?> jsonInputs = const {},
    Map<String, BlobInput> blobInputs = const {},
    CompilationMode mode = CompilationMode.production,
  }) async {
    final inputs = EncodedInputs(jsonInputs, blobInputs);
    return CompiledDocument._(await _template.compileAsync(inputs, mode));
  }

  /// Like [compile], but blocks until the template is compiled.
  CompiledDocument compileSync({
    Map<String, Object?> jsonInputs = const {},
    Map<String, BlobInput> blobInputs = const {},
    CompilationMode mode = CompilationMode.production,
  }) {
    final inputs = EncodedInputs(jsonInputs, blobInputs);
    return CompiledDocument._(_template.compile(inputs, mode));
  }

  /// Compiles the template and exports it to [format].
  ///
  /// [pages] restricts the export to a range of pages.
  Future<Uint8List> export(
    ExportFormat format, {
    Map<String, Object?> jsonInputs = const {},
    Map<String, BlobInput> blobInputs = const {},
    CompilationMode mode = CompilationMode.production,
    PageRange? pages,
  }) {
    final inputs = EncodedInputs(jsonInputs, blobInputs);
    return _template.exportAsync(inputs, mode, format, checkPages(pages));
  }

  /// Like [export], but blocks until the document is exported.
  Uint8List exportSync(
    ExportFormat format, {
    Map<String, Object?> jsonInputs = const {},
    Map<String, BlobInput> blobInputs = const {},
    CompilationMode mode = CompilationMode.production,
    PageRange? pages,
  }) {
    final document = compileSync(
      jsonInputs: jsonInputs,
      blobInputs: blobInputs,
      mode: mode,
    );
    try {
      return document.exportSync(format, pages: pages);
    } finally {
      document.dispose();
    }
  }

  /// Compiles the template and exports it to PDF. See [export].
  Future<Uint8List> exportPdf({
    Map<String, Object?> jsonInputs = const {},
    Map<String, BlobInput> blobInputs = const {},
    CompilationMode mode = CompilationMode.production,
    PageRange? pages,
  }) => export(
    ExportFormat.pdf,
    jsonInputs: jsonInputs,
    blobInputs: blobInputs,
    mode: mode,
    pages: pages,
  );

  /// Like [exportPdf], but blocks until the document is exported.
  Uint8List exportPdfSync({
    Map<String, Object?> jsonInputs = const {},
    Map<String, BlobInput> blobInputs = const {},
    CompilationMode mode = CompilationMode.production,
    PageRange? pages,
  }) => exportSync(
    ExportFormat.pdf,
    jsonInputs: jsonInputs,
    blobInputs: blobInputs,
    mode: mode,
    pages: pages,
  );

  /// Compiles the template and exports it to PNG. See [export] and
  /// [ExportFormat.png].
  Future<Uint8List> exportPng({
    Map<String, Object?> jsonInputs = const {},
    Map<String, BlobInput> blobInputs = const {},
    CompilationMode mode = CompilationMode.production,
    double pixelsPerPt = 1.0,
    PageRange? pages,
  }) => export(
    ExportFormat.png(pixelsPerPt: pixelsPerPt),
    jsonInputs: jsonInputs,
    blobInputs: blobInputs,
    mode: mode,
    pages: pages,
  );

  /// Like [exportPng], but blocks until the document is exported.
  Uint8List exportPngSync({
    Map<String, Object?> jsonInputs = const {},
    Map<String, BlobInput> blobInputs = const {},
    CompilationMode mode = CompilationMode.production,
    double pixelsPerPt = 1.0,
    PageRange? pages,
  }) => exportSync(
    ExportFormat.png(pixelsPerPt: pixelsPerPt),
    jsonInputs: jsonInputs,
    blobInputs: blobInputs,
    mode: mode,
    pages: pages,
  );

  /// Compiles the template and exports it to SVG. See [export].
  Future<Uint8List> exportSvg({
    Map<String, Object?> jsonInputs = const {},
    Map<String, BlobInput> blobInputs = const {},
    CompilationMode mode = CompilationMode.production,
    PageRange? pages,
  }) => export(
    ExportFormat.svg,
    jsonInputs: jsonInputs,
    blobInputs: blobInputs,
    mode: mode,
    pages: pages,
  );

  /// Like [exportSvg], but blocks until the document is exported.
  Uint8List exportSvgSync({
    Map<String, Object?> jsonInputs = const {},
    Map<String, BlobInput> blobInputs = const {},
    CompilationMode mode = CompilationMode.production,
    PageRange? pages,
  }) => exportSync(
    ExportFormat.svg,
    jsonInputs: jsonInputs,
    blobInputs: blobInputs,
    mode: mode,
    pages: pages,
  );

  /// The manifest of the template.
  TemplateManifest manifest() => parseManifest(_template.manifest());

  /// The content of a text file inside the template.
  String source(String path) => _template.source(path);

  /// The content of a file inside the template.
  Uint8List file(String path) => _template.file(path);

  /// Enables or disables the validation of JSON inputs against their schemas.
  /// Validation is enabled by default.
  void setValidateInputs(bool validate) =>
      _template.setValidateInputs(validate);

  /// Frees the template. Further calls throw a [StateError].
  ///
  /// Calls that are still running finish normally.
  void dispose() {
    _backend?.dispose();
    _backend = null;
  }
}

/// A compiled document that can be exported repeatedly without compiling again.
///
/// Call [dispose] when the document is no longer needed; documents that are
/// garbage collected are freed as well.
class CompiledDocument {
  CompiledDocument._(this._backend);

  DocumentBackend? _backend;

  DocumentBackend get _document =>
      _backend ?? (throw StateError('The document is already disposed'));

  /// The size of every page, in document order.
  List<PageSize> get pages => _document.pages;

  /// The number of pages.
  int get pageCount => _document.pages.length;

  /// The warnings of the compilation.
  List<Diagnostic> get warnings => _document.warnings;

  /// Exports the document to [format].
  ///
  /// [pages] restricts the export to a range of pages.
  Future<Uint8List> export(ExportFormat format, {PageRange? pages}) =>
      _document.exportAsync(format, checkPages(pages));

  /// Like [export], but blocks until the document is exported.
  Uint8List exportSync(ExportFormat format, {PageRange? pages}) =>
      _document.export(format, checkPages(pages));

  /// Exports the document to PDF. See [export].
  Future<Uint8List> exportPdf({PageRange? pages}) =>
      export(ExportFormat.pdf, pages: pages);

  /// Like [exportPdf], but blocks until the document is exported.
  Uint8List exportPdfSync({PageRange? pages}) =>
      exportSync(ExportFormat.pdf, pages: pages);

  /// Exports the document to PNG. See [export] and [ExportFormat.png].
  Future<Uint8List> exportPng({double pixelsPerPt = 1.0, PageRange? pages}) =>
      export(ExportFormat.png(pixelsPerPt: pixelsPerPt), pages: pages);

  /// Like [exportPng], but blocks until the document is exported.
  Uint8List exportPngSync({double pixelsPerPt = 1.0, PageRange? pages}) =>
      exportSync(ExportFormat.png(pixelsPerPt: pixelsPerPt), pages: pages);

  /// Exports the document to SVG. See [export].
  Future<Uint8List> exportSvg({PageRange? pages}) =>
      export(ExportFormat.svg, pages: pages);

  /// Like [exportSvg], but blocks until the document is exported.
  Uint8List exportSvgSync({PageRange? pages}) =>
      exportSync(ExportFormat.svg, pages: pages);

  /// Frees the document. Further calls throw a [StateError].
  ///
  /// Calls that are still running finish normally.
  void dispose() {
    _backend?.dispose();
    _backend = null;
  }
}
