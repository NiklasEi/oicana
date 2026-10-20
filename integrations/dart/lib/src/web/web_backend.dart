// Internal: not exported from lib/oicana.dart, so no API docs needed.
// ignore_for_file: public_member_api_docs

import 'dart:convert';
import 'dart:js_interop';
import 'dart:js_interop_unsafe';
import 'dart:typed_data';

import '../backend.dart';
import '../exceptions.dart';
import '../types.dart';
import '../version.dart';

final Backend backend = _WebBackend();

const _flutterModulePath = 'assets/packages/oicana/web/oicana_browser_wasm.js';

_Wasm? _module;
Future<void>? _loading;
var _nextId = 0;

_Wasm get _wasm =>
    _module ??
    (throw StateError(
      'Oicana is not initialized. Await initOicana() before using it on the web.',
    ));

final class _WebBackend implements Backend {
  @override
  Future<void> initialize(Uri? module) async {
    if (_module != null) return;
    final loading = _loading ??= _load(module);
    try {
      await loading;
    } catch (_) {
      // Allow retrying.
      if (identical(_loading, loading)) _loading = null;
      rethrow;
    }
  }

  @override
  ExportOnceResult exportOnce(
    Uint8List files,
    EncodedInputs inputs,
    CompilationMode mode,
    ExportFormat format,
    PageRange? pages,
    ZipLimits? limits,
  ) {
    final result = _guard(
      () => _wasm.exportTemplateOnce(
        files.toJS,
        _jsonInputs(inputs),
        _blobInputs(inputs),
        mode.name.toJS,
        _format(format),
        _pages(pages),
        _limits(limits),
      ),
    );
    return ExportOnceResult(
      document: result.data.toDart,
      warnings: _diagnostics(result.warnings),
    );
  }

  @override
  Future<ExportOnceResult> exportOnceAsync(
    Uint8List files,
    EncodedInputs inputs,
    CompilationMode mode,
    ExportFormat format,
    PageRange? pages,
    ZipLimits? limits,
  ) async {
    await initialize(null);
    return exportOnce(files, inputs, mode, format, pages, limits);
  }

  @override
  TemplateBackend loadTemplate(
    Uint8List files,
    EncodedInputs inputs,
    CompilationMode mode,
    ZipLimits? limits,
  ) {
    final id = 'dart-${_nextId++}';
    final warmUp = _guard(
      () => _wasm.registerTemplate(
        id.toJS,
        files.toJS,
        _jsonInputs(inputs),
        _blobInputs(inputs),
        mode.name.toJS,
        _limits(limits),
      ),
    );
    final warnings = _diagnostics(_wasm.getWarnings(warmUp));
    _wasm.removeDocument(warmUp);
    return _WebTemplate(id, warnings);
  }

  @override
  Future<TemplateBackend> loadTemplateAsync(
    Uint8List files,
    EncodedInputs inputs,
    CompilationMode mode,
    ZipLimits? limits,
  ) async {
    await initialize(null);
    return loadTemplate(files, inputs, mode, limits);
  }

  @override
  int registerFonts(List<Uint8List> fonts) =>
      _wasm.registerFonts([for (final font in fonts) font.toJS].toJS).toDartInt;

  @override
  int registerFontPaths(List<String> paths) => throw UnsupportedError(
    'Fonts cannot be registered from paths on the web, use registerFonts',
  );

  @override
  List<RegisteredFont> registeredFonts() => [
    for (final font in _wasm.registeredFonts().dartify() as List)
      RegisteredFont(family: (font as Map)['family'] as String),
  ];

  @override
  void clearFonts() => _wasm.clearFonts();

  @override
  void configureDiagnosticColor(DiagnosticColor color) =>
      _wasm.configureDiagnosticColor((color == DiagnosticColor.ansi).toJS);

  @override
  void configureAutomaticCacheEviction(int? maxAge) =>
      _wasm.configureAutomaticCacheEviction(maxAge?.toJS);

  @override
  void evictCache(int maxAge) => _wasm.evictCache(maxAge.toJS);

  @override
  String version() => packageVersion;
}

final _templates = Finalizer<String>((id) => _module?.removeWorld(id.toJS));
final _documents = Finalizer<String>((id) => _module?.removeDocument(id.toJS));

final class _WebTemplate implements TemplateBackend {
  _WebTemplate(this._id, this.warnings) {
    _templates.attach(this, _id, detach: this);
  }

  final String _id;

  @override
  final List<Diagnostic> warnings;

  @override
  DocumentBackend compile(EncodedInputs inputs, CompilationMode mode) =>
      _WebDocument(
        _guard(
          () => _wasm
              .compileTemplate(
                _id.toJS,
                _jsonInputs(inputs),
                _blobInputs(inputs),
                mode.name.toJS,
              )
              .toDart,
        ),
      );

  @override
  Future<DocumentBackend> compileAsync(
    EncodedInputs inputs,
    CompilationMode mode,
  ) async => compile(inputs, mode);

  @override
  Future<Uint8List> exportAsync(
    EncodedInputs inputs,
    CompilationMode mode,
    ExportFormat format,
    PageRange? pages,
  ) async {
    final document = compile(inputs, mode);
    try {
      return document.export(format, pages);
    } finally {
      document.dispose();
    }
  }

  @override
  String manifest() => _guard(() => _wasm.manifest(_id.toJS).toDart);

  @override
  String source(String path) =>
      _guard(() => _wasm.getSource(_id.toJS, path.toJS).toDart);

  @override
  Uint8List file(String path) =>
      _guard(() => _wasm.getFile(_id.toJS, path.toJS).toDart);

  @override
  void setValidateInputs(bool validate) =>
      _guard(() => _wasm.setValidateInputs(_id.toJS, validate.toJS));

  @override
  void dispose() {
    _templates.detach(this);
    _wasm.removeWorld(_id.toJS);
  }
}

final class _WebDocument implements DocumentBackend {
  _WebDocument(this._id)
    : pages = List.unmodifiable([
        for (final page
            in jsonDecode(_wasm.documentPages(_id.toJS).toDart) as List)
          PageSize(
            width: ((page as Map)['width'] as num).toDouble(),
            height: (page['height'] as num).toDouble(),
          ),
      ]),
      warnings = _diagnostics(_wasm.getWarnings(_id.toJS)) {
    _documents.attach(this, _id, detach: this);
  }

  final String _id;

  @override
  final List<PageSize> pages;

  @override
  final List<Diagnostic> warnings;

  @override
  Uint8List export(ExportFormat format, PageRange? pages) => _guard(
    () => _wasm.exportDocument(_id.toJS, _format(format), _pages(pages)).toDart,
  );

  @override
  Future<Uint8List> exportAsync(ExportFormat format, PageRange? pages) async =>
      export(format, pages);

  @override
  void dispose() {
    _documents.detach(this);
    _wasm.removeDocument(_id.toJS);
  }
}

Future<void> _load(Uri? module) async {
  // Relative to the page, not to the compiled app script.
  final base = (globalContext['document'] as JSObject?)
      ?.getProperty<JSString?>('baseURI'.toJS)
      ?.toDart;
  final url = (base == null ? Uri.base : Uri.parse(base)).resolveUri(
    module ?? Uri.parse(_flutterModulePath),
  );
  final wasm = await importModule(url.toString().toJS).toDart as _Wasm;
  final options = JSObject()
    ..['module_or_path'] = url
        .resolve('oicana_browser_wasm_bg.wasm')
        .toString()
        .toJS;
  await wasm.init(options).toDart;
  _module = wasm;
}

T _guard<T>(T Function() call) {
  try {
    return call();
  } on OicanaException {
    rethrow;
  } catch (error) {
    if (error is StateError) rethrow;
    throw OicanaException(_message(error));
  }
}

String _message(Object error) => error is String ? error : error.toString();

JSObject _jsonInputs(EncodedInputs inputs) {
  final object = JSObject();
  for (final MapEntry(:key, :value) in inputs.json.entries) {
    object[key] = value.toJS;
  }
  return object;
}

JSObject _blobInputs(EncodedInputs inputs) {
  final json = globalContext['JSON'] as JSObject;
  final object = JSObject();
  for (final MapEntry(:key, :value) in inputs.blobs.entries) {
    object[key] = JSObject()
      ..['data'] = value.data.toJS
      ..['metadata'] = json.callMethod('parse'.toJS, value.metadata.toJS);
  }
  return object;
}

JSObject _format(ExportFormat format) {
  final object = JSObject();
  switch (format) {
    case PdfFormat():
      object['format'] = 'pdf'.toJS;
    case SvgFormat():
      object['format'] = 'svg'.toJS;
    case PngFormat(:final pixelsPerPt):
      object['format'] = 'png'.toJS;
      object['pixelsPerPt'] = pixelsPerPt.toJS;
  }
  return object;
}

JSObject? _pages(PageRange? pages) {
  if (pages == null) return null;
  final object = JSObject();
  if (pages.start case final start?) object['start'] = start.toJS;
  if (pages.end case final end?) object['end'] = end.toJS;
  return object;
}

JSObject? _limits(ZipLimits? limits) {
  if (limits == null) return null;
  final object = JSObject();
  if (limits.maxEntries case final value?) object['maxEntries'] = value.toJS;
  if (limits.maxTotalDecompressedBytes case final value?) {
    object['maxTotalDecompressedBytes'] = value.toJS;
  }
  return object;
}

List<Diagnostic> _diagnostics(JSAny? diagnostics) => List.unmodifiable([
  for (final diagnostic in diagnostics.dartify() as List)
    _diagnostic(diagnostic as Map),
]);

Diagnostic _diagnostic(Map json) => Diagnostic(
  severity: json['severity'] == 'error'
      ? DiagnosticSeverity.error
      : DiagnosticSeverity.warning,
  message: json['message'] as String,
  location: _location(json['location']),
  hints: List.unmodifiable((json['hints'] as List).cast<String>()),
  trace: List.unmodifiable([
    for (final point in json['trace'] as List)
      TracePoint(
        message: (point as Map)['message'] as String,
        location: _location(point['location']),
      ),
  ]),
  formatted: json['formatted'] as String,
);

SourceLocation? _location(Object? json) => json is Map
    ? SourceLocation(
        file: json['file'] as String,
        line: (json['line'] as num).toInt(),
        column: (json['column'] as num).toInt(),
        endLine: (json['endLine'] as num).toInt(),
        endColumn: (json['endColumn'] as num).toInt(),
      )
    : null;

extension type _Wasm._(JSObject _) implements JSObject {
  @JS('default')
  external JSPromise<JSAny?> init(JSObject options);

  @JS('export_template_once')
  external _ExportOnceResult exportTemplateOnce(
    JSUint8Array files,
    JSObject jsonInputs,
    JSObject blobInputs,
    JSString mode,
    JSObject format,
    JSObject? pages,
    JSObject? limits,
  );

  @JS('register_template')
  external JSString registerTemplate(
    JSString template,
    JSUint8Array files,
    JSObject jsonInputs,
    JSObject blobInputs,
    JSString mode,
    JSObject? limits,
  );

  @JS('compile_template')
  external JSString compileTemplate(
    JSString template,
    JSObject jsonInputs,
    JSObject blobInputs,
    JSString mode,
  );

  @JS('export_document')
  external JSUint8Array exportDocument(
    JSString document,
    JSObject format,
    JSObject? pages,
  );

  @JS('document_pages')
  external JSString documentPages(JSString document);

  @JS('get_warnings')
  external JSAny? getWarnings(JSString document);

  @JS('remove_document')
  external void removeDocument(JSString document);

  @JS('remove_world')
  external void removeWorld(JSString template);

  external JSString manifest(JSString template);

  @JS('get_source')
  external JSString getSource(JSString template, JSString path);

  @JS('get_file')
  external JSUint8Array getFile(JSString template, JSString path);

  @JS('set_validate_inputs')
  external void setValidateInputs(JSString template, JSBoolean validate);

  @JS('register_fonts')
  external JSNumber registerFonts(JSArray<JSUint8Array> fonts);

  @JS('registered_fonts')
  external JSAny? registeredFonts();

  @JS('clear_fonts')
  external void clearFonts();

  @JS('configure_diagnostic_color')
  external void configureDiagnosticColor(JSBoolean ansi);

  @JS('configure_automatic_cache_eviction')
  external void configureAutomaticCacheEviction(JSNumber? maxAge);

  @JS('evict_cache')
  external void evictCache(JSNumber maxAge);
}

extension type _ExportOnceResult._(JSObject _) implements JSObject {
  external JSUint8Array get data;
  external JSAny? get warnings;
}
