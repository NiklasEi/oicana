// Internal: not exported from lib/oicana.dart, so no API docs needed.
// ignore_for_file: public_member_api_docs

import 'dart:convert';
import 'dart:typed_data';

import 'exceptions.dart';
import 'types.dart';

/// Only holds values that can be sent to another isolate.
final class EncodedInputs {
  factory EncodedInputs(
    Map<String, Object?> jsonInputs,
    Map<String, BlobInput> blobInputs,
  ) {
    return EncodedInputs._(
      {
        for (final MapEntry(:key, :value) in jsonInputs.entries)
          key: value is String
              ? value
              : _encode(value, 'the JSON input "$key"'),
      },
      {
        for (final MapEntry(:key, :value) in blobInputs.entries)
          key: (
            data: value.data,
            metadata: value.metadata == null
                ? '{}'
                : _encode(value.metadata, 'the metadata of the blob "$key"'),
          ),
      },
    );
  }

  EncodedInputs._(this.json, this.blobs);

  final Map<String, String> json;

  final Map<String, ({Uint8List data, String metadata})> blobs;

  static String _encode(Object? value, String what) {
    try {
      return jsonEncode(value);
    } on JsonUnsupportedObjectError catch (error) {
      throw InvalidInputException('Failed to encode $what as JSON: $error');
    }
  }
}

PageRange? checkPages(PageRange? pages) {
  for (final index in [pages?.start, pages?.end]) {
    if (index != null && index < 0) {
      throw InvalidInputException(
        'Page indices must not be negative, got $index',
      );
    }
  }
  return pages;
}

abstract interface class Backend {
  Future<void> initialize(Uri? module);

  ExportOnceResult exportOnce(
    Uint8List files,
    EncodedInputs inputs,
    CompilationMode mode,
    ExportFormat format,
    PageRange? pages,
    ZipLimits? limits,
  );

  Future<ExportOnceResult> exportOnceAsync(
    Uint8List files,
    EncodedInputs inputs,
    CompilationMode mode,
    ExportFormat format,
    PageRange? pages,
    ZipLimits? limits,
  );

  TemplateBackend loadTemplate(
    Uint8List files,
    EncodedInputs inputs,
    CompilationMode mode,
    ZipLimits? limits,
  );

  Future<TemplateBackend> loadTemplateAsync(
    Uint8List files,
    EncodedInputs inputs,
    CompilationMode mode,
    ZipLimits? limits,
  );

  int registerFonts(List<Uint8List> fonts);

  int registerFontPaths(List<String> paths);

  List<RegisteredFont> registeredFonts();

  void clearFonts();

  void configureDiagnosticColor(DiagnosticColor color);

  void configureAutomaticCacheEviction(int? maxAge);

  void evictCache(int maxAge);

  String version();
}

abstract interface class TemplateBackend {
  List<Diagnostic> get warnings;

  DocumentBackend compile(EncodedInputs inputs, CompilationMode mode);

  Future<DocumentBackend> compileAsync(
    EncodedInputs inputs,
    CompilationMode mode,
  );

  /// Compiles and exports in one step, so async exports cross isolates once.
  Future<Uint8List> exportAsync(
    EncodedInputs inputs,
    CompilationMode mode,
    ExportFormat format,
    PageRange? pages,
  );

  String manifest();

  String source(String path);

  Uint8List file(String path);

  void setValidateInputs(bool validate);

  void dispose();
}

abstract interface class DocumentBackend {
  List<PageSize> get pages;

  List<Diagnostic> get warnings;

  Uint8List export(ExportFormat format, PageRange? pages);

  Future<Uint8List> exportAsync(ExportFormat format, PageRange? pages);

  void dispose();
}
