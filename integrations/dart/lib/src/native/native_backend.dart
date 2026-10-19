// Internal: not exported from lib/oicana.dart, so no API docs needed.
// ignore_for_file: public_member_api_docs

import 'dart:ffi';
import 'dart:isolate';
import 'dart:typed_data';

import '../backend.dart';
import '../exceptions.dart';
import '../types.dart';
import '../version.dart';
import 'oicana_uniffi.dart' as ffi;

/// Async calls run on a short-lived isolate. Native objects cannot be sent
/// between isolates, so they cross as the address of a new reference, which the
/// receiving side takes ownership of.
final Backend backend = _NativeBackend();

final class _NativeBackend implements Backend {
  @override
  Future<void> initialize(Uri? module) async {}

  @override
  ExportOnceResult exportOnce(
    Uint8List files,
    EncodedInputs inputs,
    CompilationMode mode,
    ExportFormat format,
    PageRange? pages,
    ZipLimits? limits,
  ) => _exportOnce(files, inputs, mode, format, pages, limits);

  @override
  Future<ExportOnceResult> exportOnceAsync(
    Uint8List files,
    EncodedInputs inputs,
    CompilationMode mode,
    ExportFormat format,
    PageRange? pages,
    ZipLimits? limits,
  ) => _exportOnceOnIsolate(files, inputs, mode, format, pages, limits);

  @override
  TemplateBackend loadTemplate(
    Uint8List files,
    EncodedInputs inputs,
    CompilationMode mode,
    ZipLimits? limits,
  ) => _NativeTemplate(_newTemplate(files, inputs, mode, limits));

  @override
  Future<TemplateBackend> loadTemplateAsync(
    Uint8List files,
    EncodedInputs inputs,
    CompilationMode mode,
    ZipLimits? limits,
  ) async {
    final address = await _newTemplateOnIsolate(files, inputs, mode, limits);
    return _NativeTemplate(ffi.Template.lift(Pointer.fromAddress(address)));
  }

  @override
  int registerFonts(List<Uint8List> fonts) =>
      _guard(() => ffi.registerFonts(fonts: fonts));

  @override
  int registerFontPaths(List<String> paths) =>
      _guard(() => ffi.registerFontPaths(paths: paths));

  @override
  List<RegisteredFont> registeredFonts() => [
    for (final font in _guard(ffi.registeredFonts))
      RegisteredFont(family: font.family, path: font.path),
  ];

  @override
  void clearFonts() => _guard(ffi.clearFonts);

  @override
  void configureDiagnosticColor(DiagnosticColor color) => _guard(
    () => ffi.configureDiagnosticColor(
      color: switch (color) {
        DiagnosticColor.none => ffi.DiagnosticColor.none,
        DiagnosticColor.ansi => ffi.DiagnosticColor.ansi,
      },
    ),
  );

  @override
  void configureAutomaticCacheEviction(int? maxAge) =>
      _guard(() => ffi.configureAutomaticCacheEviction(maxAge: maxAge));

  @override
  void evictCache(int maxAge) => _guard(() => ffi.evictCache(maxAge: maxAge));

  @override
  String version() => _guard(ffi.version);
}

final class _NativeTemplate implements TemplateBackend {
  _NativeTemplate(this._template)
    : warnings = _guard(() => _diagnostics(_template.warnings()));

  final ffi.Template _template;

  @override
  final List<Diagnostic> warnings;

  @override
  DocumentBackend compile(EncodedInputs inputs, CompilationMode mode) =>
      _NativeDocument(_compile(_template, inputs, mode));

  @override
  Future<DocumentBackend> compileAsync(
    EncodedInputs inputs,
    CompilationMode mode,
  ) async {
    final address = await _compileOnIsolate(_share(_template), inputs, mode);
    return _NativeDocument(
      ffi.CompiledDocument.lift(Pointer.fromAddress(address)),
    );
  }

  @override
  Future<Uint8List> exportAsync(
    EncodedInputs inputs,
    CompilationMode mode,
    ExportFormat format,
    PageRange? pages,
  ) => _exportOnIsolate(_share(_template), inputs, mode, format, pages);

  @override
  String manifest() => _guard(_template.manifest);

  @override
  String source(String path) => _guard(() => _template.source(path: path));

  @override
  Uint8List file(String path) => _guard(() => _template.file(path: path));

  @override
  void setValidateInputs(bool validate) =>
      _guard(() => _template.setValidateInputs(validate: validate));

  @override
  void dispose() => _template.dispose();
}

final class _NativeDocument implements DocumentBackend {
  _NativeDocument(this._document)
    : pages = _guard(
        () => List.unmodifiable([
          for (final page in _document.pages())
            PageSize(width: page.width, height: page.height),
        ]),
      ),
      warnings = _guard(() => _diagnostics(_document.warnings()));

  final ffi.CompiledDocument _document;

  @override
  final List<PageSize> pages;

  @override
  final List<Diagnostic> warnings;

  @override
  Uint8List export(ExportFormat format, PageRange? pages) =>
      _export(_document, format, pages);

  @override
  Future<Uint8List> exportAsync(ExportFormat format, PageRange? pages) =>
      _exportDocumentOnIsolate(_share(_document), format, pages);

  @override
  void dispose() => _document.dispose();
}

// Top-level, so the closures capture nothing but the sendable arguments.

Future<ExportOnceResult> _exportOnceOnIsolate(
  Uint8List files,
  EncodedInputs inputs,
  CompilationMode mode,
  ExportFormat format,
  PageRange? pages,
  ZipLimits? limits,
) => Isolate.run(() => _exportOnce(files, inputs, mode, format, pages, limits));

Future<int> _newTemplateOnIsolate(
  Uint8List files,
  EncodedInputs inputs,
  CompilationMode mode,
  ZipLimits? limits,
) => Isolate.run(
  () => _share(_newTemplate(files, inputs, mode, limits), dispose: true),
);

Future<int> _compileOnIsolate(
  int template,
  EncodedInputs inputs,
  CompilationMode mode,
) => Isolate.run(() {
  final owned = ffi.Template.lift(Pointer.fromAddress(template));
  try {
    return _share(_compile(owned, inputs, mode), dispose: true);
  } finally {
    owned.dispose();
  }
});

Future<Uint8List> _exportOnIsolate(
  int template,
  EncodedInputs inputs,
  CompilationMode mode,
  ExportFormat format,
  PageRange? pages,
) => Isolate.run(() {
  final owned = ffi.Template.lift(Pointer.fromAddress(template));
  try {
    final document = _compile(owned, inputs, mode);
    try {
      return _export(document, format, pages);
    } finally {
      document.dispose();
    }
  } finally {
    owned.dispose();
  }
});

Future<Uint8List> _exportDocumentOnIsolate(
  int document,
  ExportFormat format,
  PageRange? pages,
) => Isolate.run(() {
  final owned = ffi.CompiledDocument.lift(Pointer.fromAddress(document));
  try {
    return _export(owned, format, pages);
  } finally {
    owned.dispose();
  }
});

int _share(Object object, {bool dispose = false}) {
  final (pointer, free) = switch (object) {
    ffi.Template() => (object.uniffiClonePointer(), object.dispose),
    ffi.CompiledDocument() => (object.uniffiClonePointer(), object.dispose),
    _ => throw ArgumentError.value(object),
  };
  if (dispose) free();
  return pointer.address;
}

ffi.Template _newTemplate(
  Uint8List files,
  EncodedInputs inputs,
  CompilationMode mode,
  ZipLimits? limits,
) => _guard(
  () => ffi.Template(
    files: files,
    jsonInputs: inputs.json,
    blobInputs: _blobs(inputs),
    mode: _mode(mode),
    limits: _limits(limits),
  ),
);

ffi.CompiledDocument _compile(
  ffi.Template template,
  EncodedInputs inputs,
  CompilationMode mode,
) => _guard(
  () => template.compile(
    jsonInputs: inputs.json,
    blobInputs: _blobs(inputs),
    mode: _mode(mode),
  ),
);

Uint8List _export(
  ffi.CompiledDocument document,
  ExportFormat format,
  PageRange? pages,
) => _guard(
  () => document.export_(format: _format(format), pages: _pages(pages)),
);

ExportOnceResult _exportOnce(
  Uint8List files,
  EncodedInputs inputs,
  CompilationMode mode,
  ExportFormat format,
  PageRange? pages,
  ZipLimits? limits,
) {
  final result = _guard(
    () => ffi.exportOnce(
      files: files,
      jsonInputs: inputs.json,
      blobInputs: _blobs(inputs),
      mode: _mode(mode),
      format: _format(format),
      pages: _pages(pages),
      limits: _limits(limits),
    ),
  );
  return ExportOnceResult(
    document: result.bytes,
    warnings: _diagnostics(result.warnings),
  );
}

var _verified = false;

T _guard<T>(T Function() call) {
  // Checks the API checksums of the library, once per isolate.
  if (!_verified) {
    try {
      ffi.ensureInitialized();
    } on ArgumentError catch (error) {
      throw StateError(
        'The native Oicana library is not bundled with this program. Build it '
        'with `dart build` or Flutter, `dart compile` does not bundle native '
        'libraries ($error)',
      );
    } on Object catch (error) {
      throw StateError(
        'The native Oicana library does not match version $packageVersion of '
        'the oicana package ($error)',
      );
    }
    _verified = true;
  }
  try {
    return call();
  } on ffi.InvalidTemplateOicanaException catch (error) {
    throw InvalidTemplateException(error.message);
  } on ffi.InvalidInputOicanaException catch (error) {
    throw InvalidInputException(error.message);
  } on ffi.CompilationOicanaException catch (error) {
    throw CompilationException(error.message);
  } on ffi.FileNotFoundOicanaException catch (error) {
    throw FileNotFoundException(error.message);
  } on ffi.ExportOicanaException catch (error) {
    throw ExportException(error.message);
  } on ffi.NotFoundOicanaException catch (error) {
    throw NotFoundException(error.message);
  } on ffi.InternalOicanaException catch (error) {
    throw InternalException(error.message);
  }
}

Map<String, ffi.BlobInput> _blobs(EncodedInputs inputs) => {
  for (final MapEntry(:key, :value) in inputs.blobs.entries)
    key: ffi.BlobInput(data: value.data, metadata: value.metadata),
};

ffi.CompilationMode _mode(CompilationMode mode) => switch (mode) {
  CompilationMode.production => ffi.CompilationMode.production,
  CompilationMode.development => ffi.CompilationMode.development,
};

ffi.ExportFormat _format(ExportFormat format) => switch (format) {
  PdfFormat() => ffi.PdfExportFormat(),
  SvgFormat() => ffi.SvgExportFormat(),
  PngFormat(:final pixelsPerPt) => ffi.PngExportFormat(pixelsPerPt),
};

ffi.PageRange? _pages(PageRange? pages) =>
    pages == null ? null : ffi.PageRange(start: pages.start, end: pages.end);

ffi.ZipLimits? _limits(ZipLimits? limits) => limits == null
    ? null
    : ffi.ZipLimits(
        maxEntries: limits.maxEntries,
        maxTotalDecompressedBytes: limits.maxTotalDecompressedBytes,
      );

List<Diagnostic> _diagnostics(List<ffi.Diagnostic> diagnostics) =>
    List.unmodifiable([
      for (final diagnostic in diagnostics)
        Diagnostic(
          severity: diagnostic.severity == ffi.DiagnosticSeverity.error
              ? DiagnosticSeverity.error
              : DiagnosticSeverity.warning,
          message: diagnostic.message,
          location: _location(diagnostic.location),
          hints: List.unmodifiable(diagnostic.hints),
          trace: List.unmodifiable([
            for (final point in diagnostic.trace)
              TracePoint(
                message: point.message,
                location: _location(point.location),
              ),
          ]),
          formatted: diagnostic.formatted,
        ),
    ]);

SourceLocation? _location(ffi.SourceLocation? location) => location == null
    ? null
    : SourceLocation(
        file: location.file,
        line: location.line,
        column: location.column,
        endLine: location.endLine,
        endColumn: location.endColumn,
      );
