// Internal: not exported from lib/oicana.dart, so no API docs needed.
// ignore_for_file: public_member_api_docs

import 'dart:convert';

import 'manifest.dart';

TemplateManifest parseManifest(String source) {
  final json = jsonDecode(source) as Map<String, Object?>;
  final package = json['package'] as Map<String, Object?>;
  final oicana = json['oicana'] as Map<String, Object?>;
  final pdf =
      (oicana['export'] as Map<String, Object?>)['pdf'] as Map<String, Object?>;
  final fonts = oicana['fonts'] as Map<String, Object?>;
  return TemplateManifest(
    package: PackageInfo(
      name: package['name'] as String,
      version: package['version'] as String,
      entrypoint: package['entrypoint'] as String,
      authors: (package['authors'] as List).cast<String>(),
      license: package['license'] as String?,
      description: package['description'] as String?,
      homepage: package['homepage'] as String?,
      repository: package['repository'] as String?,
    ),
    oicana: OicanaConfig(
      manifestVersion: oicana['manifestVersion'] as int,
      inputs: [
        for (final input in oicana['inputs'] as List)
          _inputDefinition(input as Map<String, Object?>),
      ],
      validateJsonInputsByDefault:
          oicana['validateJsonInputsByDefault'] as bool,
      export: ExportConfig(
        pdf: PdfExportConfig(
          standards: (pdf['standards'] as List).cast<String>(),
          tagged: pdf['tagged'] as bool,
        ),
      ),
      fonts: FontConfig(require: (fonts['require'] as List).cast<String>()),
    ),
  );
}

InputDefinition _inputDefinition(Map<String, Object?> input) =>
    switch (input['type']) {
      'json' => JsonInputDefinition(
        key: input['key'] as String,
        required: input['required'] as bool,
        defaultValue: input['default'] as String?,
        development: input['development'] as String?,
        schema: input['schema'] as String?,
        validate: input['validate'] as bool,
      ),
      'blob' => BlobInputDefinition(
        key: input['key'] as String,
        required: input['required'] as bool,
        defaultValue: _blobFallback(input['default']),
        development: _blobFallback(input['development']),
      ),
      final type => throw FormatException('Unknown input type $type'),
    };

BlobFallback? _blobFallback(Object? json) {
  if (json is! Map<String, Object?>) return null;
  return BlobFallback(
    file: json['file'] as String,
    meta: json['meta'] as Map<String, Object?>?,
  );
}
