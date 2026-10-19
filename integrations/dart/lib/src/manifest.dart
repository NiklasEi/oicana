/// The manifest of a template.
final class TemplateManifest {
  /// A template manifest.
  const TemplateManifest({required this.package, required this.oicana});

  /// The Typst package section of the manifest.
  final PackageInfo package;

  /// The Oicana section of the manifest.
  final OicanaConfig oicana;
}

/// The Typst package a template is.
final class PackageInfo {
  /// The package section of a manifest.
  const PackageInfo({
    required this.name,
    required this.version,
    required this.entrypoint,
    required this.authors,
    this.license,
    this.description,
    this.homepage,
    this.repository,
  });

  /// The name of the package.
  final String name;

  /// The version of the package.
  final String version;

  /// The file the compilation starts at.
  final String entrypoint;

  /// The authors of the package.
  final List<String> authors;

  /// The license of the package, as an SPDX expression.
  final String? license;

  /// A short description of the package.
  final String? description;

  /// The homepage of the package.
  final String? homepage;

  /// The repository of the package.
  final String? repository;
}

/// The Oicana configuration of a template.
final class OicanaConfig {
  /// The Oicana section of a manifest.
  const OicanaConfig({
    required this.manifestVersion,
    required this.inputs,
    required this.validateJsonInputsByDefault,
    required this.export,
    required this.fonts,
  });

  /// The version of the manifest format.
  final int manifestVersion;

  /// The inputs the template declares, in manifest order.
  final List<InputDefinition> inputs;

  /// Whether JSON inputs are validated against their schemas by default.
  final bool validateJsonInputsByDefault;

  /// How compiled documents are exported.
  final ExportConfig export;

  /// The fonts the template expects from its host.
  final FontConfig fonts;
}

/// An input a template declares: a [JsonInputDefinition] or a
/// [BlobInputDefinition].
sealed class InputDefinition {
  const InputDefinition({required this.key, required this.required});

  /// The key the input is supplied and used under.
  final String key;

  /// Whether a value of this input is required for compilation.
  final bool required;
}

/// An input taking a JSON value.
final class JsonInputDefinition extends InputDefinition {
  /// A JSON input definition.
  const JsonInputDefinition({
    required super.key,
    required super.required,
    this.defaultValue,
    this.development,
    this.schema,
    required this.validate,
  });

  /// The file in the template holding the value used when none is supplied. In
  /// development mode, [development] takes precedence.
  final String? defaultValue;

  /// The file in the template holding the value used in development mode when
  /// none is supplied.
  final String? development;

  /// The file in the template holding the JSON schema of this input.
  final String? schema;

  /// Whether values are validated against the [schema].
  final bool validate;
}

/// An input taking arbitrary bytes.
final class BlobInputDefinition extends InputDefinition {
  /// A blob input definition.
  const BlobInputDefinition({
    required super.key,
    required super.required,
    this.defaultValue,
    this.development,
  });

  /// The blob used when no value is supplied. In development mode,
  /// [development] takes precedence.
  final BlobFallback? defaultValue;

  /// The blob used in development mode when no value is supplied.
  final BlobFallback? development;
}

/// A blob from the template, used when no value is supplied.
final class BlobFallback {
  /// A blob fallback.
  const BlobFallback({required this.file, this.meta});

  /// The file in the template holding the blob.
  final String file;

  /// The metadata passed to the template along with the blob.
  final Map<String, Object?>? meta;
}

/// How compiled documents are exported.
final class ExportConfig {
  /// An export configuration.
  const ExportConfig({required this.pdf});

  /// How documents are exported to PDF.
  final PdfExportConfig pdf;
}

/// How documents are exported to PDF.
final class PdfExportConfig {
  /// A PDF export configuration.
  const PdfExportConfig({required this.standards, required this.tagged});

  /// The PDF standards the export conforms to, for example `a-3b`.
  final List<String> standards;

  /// Whether the PDF is tagged for accessibility.
  final bool tagged;
}

/// The fonts a template expects from its host.
final class FontConfig {
  /// A font configuration.
  const FontConfig({required this.require});

  /// The font families the host has to register for this template.
  final List<String> require;
}
