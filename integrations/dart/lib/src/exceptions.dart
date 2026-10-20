/// Thrown when an Oicana call fails.
///
/// On native targets, the exception is one of the subclasses. On the web, it is
/// an [OicanaException] itself.
class OicanaException implements Exception {
  /// An exception with the given message.
  const OicanaException(this.message);

  /// Describes the failure, including rendered compilation diagnostics.
  final String message;

  @override
  String toString() => message;
}

/// The packed template could not be read or its manifest is invalid.
class InvalidTemplateException extends OicanaException {
  /// An exception with the given message.
  const InvalidTemplateException(super.message);
}

/// An input value, its blob metadata, or a page range is invalid.
class InvalidInputException extends OicanaException {
  /// An exception with the given message.
  const InvalidInputException(super.message);
}

/// The template failed to compile.
class CompilationException extends OicanaException {
  /// An exception with the given message.
  const CompilationException(super.message);
}

/// The requested file is not part of the template.
class FileNotFoundException extends OicanaException {
  /// An exception with the given message.
  const FileNotFoundException(super.message);
}

/// The compiled document could not be exported.
class ExportException extends OicanaException {
  /// An exception with the given message.
  const ExportException(super.message);
}

/// A template or document is no longer registered.
class NotFoundException extends OicanaException {
  /// An exception with the given message.
  const NotFoundException(super.message);
}

/// An unexpected failure inside Oicana.
class InternalException extends OicanaException {
  /// An exception with the given message.
  const InternalException(super.message);
}
