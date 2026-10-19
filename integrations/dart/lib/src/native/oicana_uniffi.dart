// ignore_for_file: unused_import

library oicana;

import "dart:async";
import "dart:convert";
import "dart:ffi";
import "dart:io" show Platform, File, Directory;
import "dart:isolate";
import "dart:typed_data";
import "package:ffi/ffi.dart";
import "uniffi_runtime.dart";
export "uniffi_runtime.dart";

class BlobInput {
  final Uint8List data;
  final String metadata;
  BlobInput({required this.data, this.metadata = '{}'});
}

class FfiConverterBlobInput {
  static BlobInput lift(RustBuffer buf) {
    return FfiConverterBlobInput.read(buf.asUint8List()).value;
  }

  static LiftRetVal<BlobInput> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final data_lifted = FfiConverterUint8List.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final data = data_lifted.value;
    new_offset += data_lifted.bytesRead;
    final metadata_lifted = FfiConverterString.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final metadata = metadata_lifted.value;
    new_offset += metadata_lifted.bytesRead;
    return LiftRetVal(
      BlobInput(data: data, metadata: metadata),
      new_offset - buf.offsetInBytes,
    );
  }

  static RustBuffer lower(BlobInput value) {
    final total_length =
        FfiConverterUint8List.allocationSize(value.data) +
        FfiConverterString.allocationSize(value.metadata) +
        0;
    final buf = Uint8List(total_length);
    write(value, buf);
    return toRustBuffer(buf);
  }

  static int write(BlobInput value, Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    new_offset += FfiConverterUint8List.write(
      value.data,
      Uint8List.view(buf.buffer, new_offset),
    );
    new_offset += FfiConverterString.write(
      value.metadata,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset - buf.offsetInBytes;
  }

  static int allocationSize(BlobInput value) {
    return FfiConverterUint8List.allocationSize(value.data) +
        FfiConverterString.allocationSize(value.metadata) +
        0;
  }
}

class Diagnostic {
  final DiagnosticSeverity severity;
  final String message;
  final SourceLocation? location;
  final List<String> hints;
  final List<TracePoint> trace;
  final String formatted;
  Diagnostic({
    required this.severity,
    required this.message,
    this.location,
    required this.hints,
    required this.trace,
    required this.formatted,
  });
}

class FfiConverterDiagnostic {
  static Diagnostic lift(RustBuffer buf) {
    return FfiConverterDiagnostic.read(buf.asUint8List()).value;
  }

  static LiftRetVal<Diagnostic> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final severity_lifted = FfiConverterDiagnosticSeverity.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final severity = severity_lifted.value;
    new_offset += severity_lifted.bytesRead;
    final message_lifted = FfiConverterString.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final message = message_lifted.value;
    new_offset += message_lifted.bytesRead;
    final location_lifted = FfiConverterOptionalSourceLocation.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final location = location_lifted.value;
    new_offset += location_lifted.bytesRead;
    final hints_lifted = FfiConverterSequenceString.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final hints = hints_lifted.value;
    new_offset += hints_lifted.bytesRead;
    final trace_lifted = FfiConverterSequenceTracePoint.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final trace = trace_lifted.value;
    new_offset += trace_lifted.bytesRead;
    final formatted_lifted = FfiConverterString.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final formatted = formatted_lifted.value;
    new_offset += formatted_lifted.bytesRead;
    return LiftRetVal(
      Diagnostic(
        severity: severity,
        message: message,
        location: location,
        hints: hints,
        trace: trace,
        formatted: formatted,
      ),
      new_offset - buf.offsetInBytes,
    );
  }

  static RustBuffer lower(Diagnostic value) {
    final total_length =
        FfiConverterDiagnosticSeverity.allocationSize(value.severity) +
        FfiConverterString.allocationSize(value.message) +
        FfiConverterOptionalSourceLocation.allocationSize(value.location) +
        FfiConverterSequenceString.allocationSize(value.hints) +
        FfiConverterSequenceTracePoint.allocationSize(value.trace) +
        FfiConverterString.allocationSize(value.formatted) +
        0;
    final buf = Uint8List(total_length);
    write(value, buf);
    return toRustBuffer(buf);
  }

  static int write(Diagnostic value, Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    new_offset += FfiConverterDiagnosticSeverity.write(
      value.severity,
      Uint8List.view(buf.buffer, new_offset),
    );
    new_offset += FfiConverterString.write(
      value.message,
      Uint8List.view(buf.buffer, new_offset),
    );
    new_offset += FfiConverterOptionalSourceLocation.write(
      value.location,
      Uint8List.view(buf.buffer, new_offset),
    );
    new_offset += FfiConverterSequenceString.write(
      value.hints,
      Uint8List.view(buf.buffer, new_offset),
    );
    new_offset += FfiConverterSequenceTracePoint.write(
      value.trace,
      Uint8List.view(buf.buffer, new_offset),
    );
    new_offset += FfiConverterString.write(
      value.formatted,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset - buf.offsetInBytes;
  }

  static int allocationSize(Diagnostic value) {
    return FfiConverterDiagnosticSeverity.allocationSize(value.severity) +
        FfiConverterString.allocationSize(value.message) +
        FfiConverterOptionalSourceLocation.allocationSize(value.location) +
        FfiConverterSequenceString.allocationSize(value.hints) +
        FfiConverterSequenceTracePoint.allocationSize(value.trace) +
        FfiConverterString.allocationSize(value.formatted) +
        0;
  }
}

class ExportOnceResult {
  final Uint8List bytes;
  final List<Diagnostic> warnings;
  ExportOnceResult({required this.bytes, required this.warnings});
}

class FfiConverterExportOnceResult {
  static ExportOnceResult lift(RustBuffer buf) {
    return FfiConverterExportOnceResult.read(buf.asUint8List()).value;
  }

  static LiftRetVal<ExportOnceResult> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final bytes_lifted = FfiConverterUint8List.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final bytes = bytes_lifted.value;
    new_offset += bytes_lifted.bytesRead;
    final warnings_lifted = FfiConverterSequenceDiagnostic.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final warnings = warnings_lifted.value;
    new_offset += warnings_lifted.bytesRead;
    return LiftRetVal(
      ExportOnceResult(bytes: bytes, warnings: warnings),
      new_offset - buf.offsetInBytes,
    );
  }

  static RustBuffer lower(ExportOnceResult value) {
    final total_length =
        FfiConverterUint8List.allocationSize(value.bytes) +
        FfiConverterSequenceDiagnostic.allocationSize(value.warnings) +
        0;
    final buf = Uint8List(total_length);
    write(value, buf);
    return toRustBuffer(buf);
  }

  static int write(ExportOnceResult value, Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    new_offset += FfiConverterUint8List.write(
      value.bytes,
      Uint8List.view(buf.buffer, new_offset),
    );
    new_offset += FfiConverterSequenceDiagnostic.write(
      value.warnings,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset - buf.offsetInBytes;
  }

  static int allocationSize(ExportOnceResult value) {
    return FfiConverterUint8List.allocationSize(value.bytes) +
        FfiConverterSequenceDiagnostic.allocationSize(value.warnings) +
        0;
  }
}

class PageRange {
  final int? start;
  final int? end;
  PageRange({this.start = null, this.end = null});
}

class FfiConverterPageRange {
  static PageRange lift(RustBuffer buf) {
    return FfiConverterPageRange.read(buf.asUint8List()).value;
  }

  static LiftRetVal<PageRange> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final start_lifted = FfiConverterOptionalUInt64.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final start = start_lifted.value;
    new_offset += start_lifted.bytesRead;
    final end_lifted = FfiConverterOptionalUInt64.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final end = end_lifted.value;
    new_offset += end_lifted.bytesRead;
    return LiftRetVal(
      PageRange(start: start, end: end),
      new_offset - buf.offsetInBytes,
    );
  }

  static RustBuffer lower(PageRange value) {
    final total_length =
        FfiConverterOptionalUInt64.allocationSize(value.start) +
        FfiConverterOptionalUInt64.allocationSize(value.end) +
        0;
    final buf = Uint8List(total_length);
    write(value, buf);
    return toRustBuffer(buf);
  }

  static int write(PageRange value, Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    new_offset += FfiConverterOptionalUInt64.write(
      value.start,
      Uint8List.view(buf.buffer, new_offset),
    );
    new_offset += FfiConverterOptionalUInt64.write(
      value.end,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset - buf.offsetInBytes;
  }

  static int allocationSize(PageRange value) {
    return FfiConverterOptionalUInt64.allocationSize(value.start) +
        FfiConverterOptionalUInt64.allocationSize(value.end) +
        0;
  }
}

class PageSize {
  final double width;
  final double height;
  PageSize({required this.width, required this.height});
}

class FfiConverterPageSize {
  static PageSize lift(RustBuffer buf) {
    return FfiConverterPageSize.read(buf.asUint8List()).value;
  }

  static LiftRetVal<PageSize> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final width_lifted = FfiConverterDouble64.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final width = width_lifted.value;
    new_offset += width_lifted.bytesRead;
    final height_lifted = FfiConverterDouble64.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final height = height_lifted.value;
    new_offset += height_lifted.bytesRead;
    return LiftRetVal(
      PageSize(width: width, height: height),
      new_offset - buf.offsetInBytes,
    );
  }

  static RustBuffer lower(PageSize value) {
    final total_length =
        FfiConverterDouble64.allocationSize(value.width) +
        FfiConverterDouble64.allocationSize(value.height) +
        0;
    final buf = Uint8List(total_length);
    write(value, buf);
    return toRustBuffer(buf);
  }

  static int write(PageSize value, Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    new_offset += FfiConverterDouble64.write(
      value.width,
      Uint8List.view(buf.buffer, new_offset),
    );
    new_offset += FfiConverterDouble64.write(
      value.height,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset - buf.offsetInBytes;
  }

  static int allocationSize(PageSize value) {
    return FfiConverterDouble64.allocationSize(value.width) +
        FfiConverterDouble64.allocationSize(value.height) +
        0;
  }
}

class RegisteredFont {
  final String family;
  final String? path;
  RegisteredFont({required this.family, this.path});
}

class FfiConverterRegisteredFont {
  static RegisteredFont lift(RustBuffer buf) {
    return FfiConverterRegisteredFont.read(buf.asUint8List()).value;
  }

  static LiftRetVal<RegisteredFont> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final family_lifted = FfiConverterString.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final family = family_lifted.value;
    new_offset += family_lifted.bytesRead;
    final path_lifted = FfiConverterOptionalString.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final path = path_lifted.value;
    new_offset += path_lifted.bytesRead;
    return LiftRetVal(
      RegisteredFont(family: family, path: path),
      new_offset - buf.offsetInBytes,
    );
  }

  static RustBuffer lower(RegisteredFont value) {
    final total_length =
        FfiConverterString.allocationSize(value.family) +
        FfiConverterOptionalString.allocationSize(value.path) +
        0;
    final buf = Uint8List(total_length);
    write(value, buf);
    return toRustBuffer(buf);
  }

  static int write(RegisteredFont value, Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    new_offset += FfiConverterString.write(
      value.family,
      Uint8List.view(buf.buffer, new_offset),
    );
    new_offset += FfiConverterOptionalString.write(
      value.path,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset - buf.offsetInBytes;
  }

  static int allocationSize(RegisteredFont value) {
    return FfiConverterString.allocationSize(value.family) +
        FfiConverterOptionalString.allocationSize(value.path) +
        0;
  }
}

class SourceLocation {
  final String file;
  final int line;
  final int column;
  final int endLine;
  final int endColumn;
  SourceLocation({
    required this.file,
    required this.line,
    required this.column,
    required this.endLine,
    required this.endColumn,
  });
}

class FfiConverterSourceLocation {
  static SourceLocation lift(RustBuffer buf) {
    return FfiConverterSourceLocation.read(buf.asUint8List()).value;
  }

  static LiftRetVal<SourceLocation> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final file_lifted = FfiConverterString.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final file = file_lifted.value;
    new_offset += file_lifted.bytesRead;
    final line_lifted = FfiConverterUInt32.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final line = line_lifted.value;
    new_offset += line_lifted.bytesRead;
    final column_lifted = FfiConverterUInt32.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final column = column_lifted.value;
    new_offset += column_lifted.bytesRead;
    final endLine_lifted = FfiConverterUInt32.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final endLine = endLine_lifted.value;
    new_offset += endLine_lifted.bytesRead;
    final endColumn_lifted = FfiConverterUInt32.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final endColumn = endColumn_lifted.value;
    new_offset += endColumn_lifted.bytesRead;
    return LiftRetVal(
      SourceLocation(
        file: file,
        line: line,
        column: column,
        endLine: endLine,
        endColumn: endColumn,
      ),
      new_offset - buf.offsetInBytes,
    );
  }

  static RustBuffer lower(SourceLocation value) {
    final total_length =
        FfiConverterString.allocationSize(value.file) +
        FfiConverterUInt32.allocationSize(value.line) +
        FfiConverterUInt32.allocationSize(value.column) +
        FfiConverterUInt32.allocationSize(value.endLine) +
        FfiConverterUInt32.allocationSize(value.endColumn) +
        0;
    final buf = Uint8List(total_length);
    write(value, buf);
    return toRustBuffer(buf);
  }

  static int write(SourceLocation value, Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    new_offset += FfiConverterString.write(
      value.file,
      Uint8List.view(buf.buffer, new_offset),
    );
    new_offset += FfiConverterUInt32.write(
      value.line,
      Uint8List.view(buf.buffer, new_offset),
    );
    new_offset += FfiConverterUInt32.write(
      value.column,
      Uint8List.view(buf.buffer, new_offset),
    );
    new_offset += FfiConverterUInt32.write(
      value.endLine,
      Uint8List.view(buf.buffer, new_offset),
    );
    new_offset += FfiConverterUInt32.write(
      value.endColumn,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset - buf.offsetInBytes;
  }

  static int allocationSize(SourceLocation value) {
    return FfiConverterString.allocationSize(value.file) +
        FfiConverterUInt32.allocationSize(value.line) +
        FfiConverterUInt32.allocationSize(value.column) +
        FfiConverterUInt32.allocationSize(value.endLine) +
        FfiConverterUInt32.allocationSize(value.endColumn) +
        0;
  }
}

class TracePoint {
  final String message;
  final SourceLocation? location;
  TracePoint({required this.message, this.location});
}

class FfiConverterTracePoint {
  static TracePoint lift(RustBuffer buf) {
    return FfiConverterTracePoint.read(buf.asUint8List()).value;
  }

  static LiftRetVal<TracePoint> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final message_lifted = FfiConverterString.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final message = message_lifted.value;
    new_offset += message_lifted.bytesRead;
    final location_lifted = FfiConverterOptionalSourceLocation.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final location = location_lifted.value;
    new_offset += location_lifted.bytesRead;
    return LiftRetVal(
      TracePoint(message: message, location: location),
      new_offset - buf.offsetInBytes,
    );
  }

  static RustBuffer lower(TracePoint value) {
    final total_length =
        FfiConverterString.allocationSize(value.message) +
        FfiConverterOptionalSourceLocation.allocationSize(value.location) +
        0;
    final buf = Uint8List(total_length);
    write(value, buf);
    return toRustBuffer(buf);
  }

  static int write(TracePoint value, Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    new_offset += FfiConverterString.write(
      value.message,
      Uint8List.view(buf.buffer, new_offset),
    );
    new_offset += FfiConverterOptionalSourceLocation.write(
      value.location,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset - buf.offsetInBytes;
  }

  static int allocationSize(TracePoint value) {
    return FfiConverterString.allocationSize(value.message) +
        FfiConverterOptionalSourceLocation.allocationSize(value.location) +
        0;
  }
}

class ZipLimits {
  final int? maxEntries;
  final int? maxTotalDecompressedBytes;
  ZipLimits({this.maxEntries = null, this.maxTotalDecompressedBytes = null});
}

class FfiConverterZipLimits {
  static ZipLimits lift(RustBuffer buf) {
    return FfiConverterZipLimits.read(buf.asUint8List()).value;
  }

  static LiftRetVal<ZipLimits> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final maxEntries_lifted = FfiConverterOptionalUInt64.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final maxEntries = maxEntries_lifted.value;
    new_offset += maxEntries_lifted.bytesRead;
    final maxTotalDecompressedBytes_lifted = FfiConverterOptionalUInt64.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final maxTotalDecompressedBytes = maxTotalDecompressedBytes_lifted.value;
    new_offset += maxTotalDecompressedBytes_lifted.bytesRead;
    return LiftRetVal(
      ZipLimits(
        maxEntries: maxEntries,
        maxTotalDecompressedBytes: maxTotalDecompressedBytes,
      ),
      new_offset - buf.offsetInBytes,
    );
  }

  static RustBuffer lower(ZipLimits value) {
    final total_length =
        FfiConverterOptionalUInt64.allocationSize(value.maxEntries) +
        FfiConverterOptionalUInt64.allocationSize(
          value.maxTotalDecompressedBytes,
        ) +
        0;
    final buf = Uint8List(total_length);
    write(value, buf);
    return toRustBuffer(buf);
  }

  static int write(ZipLimits value, Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    new_offset += FfiConverterOptionalUInt64.write(
      value.maxEntries,
      Uint8List.view(buf.buffer, new_offset),
    );
    new_offset += FfiConverterOptionalUInt64.write(
      value.maxTotalDecompressedBytes,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset - buf.offsetInBytes;
  }

  static int allocationSize(ZipLimits value) {
    return FfiConverterOptionalUInt64.allocationSize(value.maxEntries) +
        FfiConverterOptionalUInt64.allocationSize(
          value.maxTotalDecompressedBytes,
        ) +
        0;
  }
}

abstract class OicanaException implements Exception {
  RustBuffer lower();
  int allocationSize();
  int write(Uint8List buf);
}

class FfiConverterOicanaException {
  static OicanaException lift(RustBuffer buffer) {
    return FfiConverterOicanaException.read(buffer.asUint8List()).value;
  }

  static LiftRetVal<OicanaException> read(Uint8List buf) {
    final index = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
    final subview = Uint8List.view(buf.buffer, buf.offsetInBytes + 4);
    switch (index) {
      case 1:
        final lifted = InvalidTemplateOicanaException.read(subview);
        return LiftRetVal<OicanaException>(
          lifted.value,
          lifted.bytesRead - subview.offsetInBytes + 4,
        );
      case 2:
        final lifted = InvalidInputOicanaException.read(subview);
        return LiftRetVal<OicanaException>(
          lifted.value,
          lifted.bytesRead - subview.offsetInBytes + 4,
        );
      case 3:
        final lifted = CompilationOicanaException.read(subview);
        return LiftRetVal<OicanaException>(
          lifted.value,
          lifted.bytesRead - subview.offsetInBytes + 4,
        );
      case 4:
        final lifted = FileNotFoundOicanaException.read(subview);
        return LiftRetVal<OicanaException>(
          lifted.value,
          lifted.bytesRead - subview.offsetInBytes + 4,
        );
      case 5:
        final lifted = ExportOicanaException.read(subview);
        return LiftRetVal<OicanaException>(
          lifted.value,
          lifted.bytesRead - subview.offsetInBytes + 4,
        );
      case 6:
        final lifted = NotFoundOicanaException.read(subview);
        return LiftRetVal<OicanaException>(
          lifted.value,
          lifted.bytesRead - subview.offsetInBytes + 4,
        );
      case 7:
        final lifted = InternalOicanaException.read(subview);
        return LiftRetVal<OicanaException>(
          lifted.value,
          lifted.bytesRead - subview.offsetInBytes + 4,
        );
      default:
        throw UniffiInternalError(
          UniffiInternalError.unexpectedEnumCase,
          "Unable to determine enum variant",
        );
    }
  }

  static RustBuffer lower(OicanaException value) {
    return value.lower();
  }

  static int allocationSize(OicanaException value) {
    return value.allocationSize();
  }

  static int write(OicanaException value, Uint8List buf) {
    return value.write(buf) - buf.offsetInBytes;
  }
}

class InvalidTemplateOicanaException extends OicanaException {
  final String message;
  InvalidTemplateOicanaException(String this.message);
  InvalidTemplateOicanaException._(String this.message);
  static LiftRetVal<InvalidTemplateOicanaException> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final message_lifted = FfiConverterString.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final message = message_lifted.value;
    new_offset += message_lifted.bytesRead;
    return LiftRetVal(InvalidTemplateOicanaException._(message), new_offset);
  }

  @override
  RustBuffer lower() {
    final buf = Uint8List(allocationSize());
    write(buf);
    return toRustBuffer(buf);
  }

  @override
  int allocationSize() {
    return FfiConverterString.allocationSize(message) + 4;
  }

  @override
  int write(Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, 1);
    int new_offset = buf.offsetInBytes + 4;
    new_offset += FfiConverterString.write(
      message,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset;
  }

  @override
  String toString() {
    return "InvalidTemplateOicanaException($message)";
  }
}

class InvalidInputOicanaException extends OicanaException {
  final String message;
  InvalidInputOicanaException(String this.message);
  InvalidInputOicanaException._(String this.message);
  static LiftRetVal<InvalidInputOicanaException> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final message_lifted = FfiConverterString.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final message = message_lifted.value;
    new_offset += message_lifted.bytesRead;
    return LiftRetVal(InvalidInputOicanaException._(message), new_offset);
  }

  @override
  RustBuffer lower() {
    final buf = Uint8List(allocationSize());
    write(buf);
    return toRustBuffer(buf);
  }

  @override
  int allocationSize() {
    return FfiConverterString.allocationSize(message) + 4;
  }

  @override
  int write(Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, 2);
    int new_offset = buf.offsetInBytes + 4;
    new_offset += FfiConverterString.write(
      message,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset;
  }

  @override
  String toString() {
    return "InvalidInputOicanaException($message)";
  }
}

class CompilationOicanaException extends OicanaException {
  final String message;
  CompilationOicanaException(String this.message);
  CompilationOicanaException._(String this.message);
  static LiftRetVal<CompilationOicanaException> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final message_lifted = FfiConverterString.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final message = message_lifted.value;
    new_offset += message_lifted.bytesRead;
    return LiftRetVal(CompilationOicanaException._(message), new_offset);
  }

  @override
  RustBuffer lower() {
    final buf = Uint8List(allocationSize());
    write(buf);
    return toRustBuffer(buf);
  }

  @override
  int allocationSize() {
    return FfiConverterString.allocationSize(message) + 4;
  }

  @override
  int write(Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, 3);
    int new_offset = buf.offsetInBytes + 4;
    new_offset += FfiConverterString.write(
      message,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset;
  }

  @override
  String toString() {
    return "CompilationOicanaException($message)";
  }
}

class FileNotFoundOicanaException extends OicanaException {
  final String message;
  FileNotFoundOicanaException(String this.message);
  FileNotFoundOicanaException._(String this.message);
  static LiftRetVal<FileNotFoundOicanaException> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final message_lifted = FfiConverterString.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final message = message_lifted.value;
    new_offset += message_lifted.bytesRead;
    return LiftRetVal(FileNotFoundOicanaException._(message), new_offset);
  }

  @override
  RustBuffer lower() {
    final buf = Uint8List(allocationSize());
    write(buf);
    return toRustBuffer(buf);
  }

  @override
  int allocationSize() {
    return FfiConverterString.allocationSize(message) + 4;
  }

  @override
  int write(Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, 4);
    int new_offset = buf.offsetInBytes + 4;
    new_offset += FfiConverterString.write(
      message,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset;
  }

  @override
  String toString() {
    return "FileNotFoundOicanaException($message)";
  }
}

class ExportOicanaException extends OicanaException {
  final String message;
  ExportOicanaException(String this.message);
  ExportOicanaException._(String this.message);
  static LiftRetVal<ExportOicanaException> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final message_lifted = FfiConverterString.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final message = message_lifted.value;
    new_offset += message_lifted.bytesRead;
    return LiftRetVal(ExportOicanaException._(message), new_offset);
  }

  @override
  RustBuffer lower() {
    final buf = Uint8List(allocationSize());
    write(buf);
    return toRustBuffer(buf);
  }

  @override
  int allocationSize() {
    return FfiConverterString.allocationSize(message) + 4;
  }

  @override
  int write(Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, 5);
    int new_offset = buf.offsetInBytes + 4;
    new_offset += FfiConverterString.write(
      message,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset;
  }

  @override
  String toString() {
    return "ExportOicanaException($message)";
  }
}

class NotFoundOicanaException extends OicanaException {
  final String message;
  NotFoundOicanaException(String this.message);
  NotFoundOicanaException._(String this.message);
  static LiftRetVal<NotFoundOicanaException> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final message_lifted = FfiConverterString.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final message = message_lifted.value;
    new_offset += message_lifted.bytesRead;
    return LiftRetVal(NotFoundOicanaException._(message), new_offset);
  }

  @override
  RustBuffer lower() {
    final buf = Uint8List(allocationSize());
    write(buf);
    return toRustBuffer(buf);
  }

  @override
  int allocationSize() {
    return FfiConverterString.allocationSize(message) + 4;
  }

  @override
  int write(Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, 6);
    int new_offset = buf.offsetInBytes + 4;
    new_offset += FfiConverterString.write(
      message,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset;
  }

  @override
  String toString() {
    return "NotFoundOicanaException($message)";
  }
}

class InternalOicanaException extends OicanaException {
  final String message;
  InternalOicanaException(String this.message);
  InternalOicanaException._(String this.message);
  static LiftRetVal<InternalOicanaException> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final message_lifted = FfiConverterString.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final message = message_lifted.value;
    new_offset += message_lifted.bytesRead;
    return LiftRetVal(InternalOicanaException._(message), new_offset);
  }

  @override
  RustBuffer lower() {
    final buf = Uint8List(allocationSize());
    write(buf);
    return toRustBuffer(buf);
  }

  @override
  int allocationSize() {
    return FfiConverterString.allocationSize(message) + 4;
  }

  @override
  int write(Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, 7);
    int new_offset = buf.offsetInBytes + 4;
    new_offset += FfiConverterString.write(
      message,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset;
  }

  @override
  String toString() {
    return "InternalOicanaException($message)";
  }
}

class OicanaExceptionErrorHandler extends UniffiRustCallStatusErrorHandler {
  @override
  Exception lift(RustBuffer errorBuf) {
    return FfiConverterOicanaException.lift(errorBuf);
  }
}

final OicanaExceptionErrorHandler oicanaExceptionErrorHandler =
    OicanaExceptionErrorHandler();

enum CompilationMode { development, production }

class FfiConverterCompilationMode {
  static LiftRetVal<CompilationMode> read(Uint8List buf) {
    final index = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
    switch (index) {
      case 1:
        return LiftRetVal(CompilationMode.development, 4);
      case 2:
        return LiftRetVal(CompilationMode.production, 4);
      default:
        throw UniffiInternalError(
          UniffiInternalError.unexpectedEnumCase,
          "Unable to determine enum variant",
        );
    }
  }

  static CompilationMode lift(RustBuffer buffer) {
    return FfiConverterCompilationMode.read(buffer.asUint8List()).value;
  }

  static RustBuffer lower(CompilationMode input) {
    return toRustBuffer(createUint8ListFromInt(input.index + 1));
  }

  static int allocationSize(CompilationMode _value) {
    return 4;
  }

  static int write(CompilationMode value, Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, value.index + 1);
    return 4;
  }
}

enum DiagnosticColor { none, ansi }

class FfiConverterDiagnosticColor {
  static LiftRetVal<DiagnosticColor> read(Uint8List buf) {
    final index = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
    switch (index) {
      case 1:
        return LiftRetVal(DiagnosticColor.none, 4);
      case 2:
        return LiftRetVal(DiagnosticColor.ansi, 4);
      default:
        throw UniffiInternalError(
          UniffiInternalError.unexpectedEnumCase,
          "Unable to determine enum variant",
        );
    }
  }

  static DiagnosticColor lift(RustBuffer buffer) {
    return FfiConverterDiagnosticColor.read(buffer.asUint8List()).value;
  }

  static RustBuffer lower(DiagnosticColor input) {
    return toRustBuffer(createUint8ListFromInt(input.index + 1));
  }

  static int allocationSize(DiagnosticColor _value) {
    return 4;
  }

  static int write(DiagnosticColor value, Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, value.index + 1);
    return 4;
  }
}

enum DiagnosticSeverity { error, warning }

class FfiConverterDiagnosticSeverity {
  static LiftRetVal<DiagnosticSeverity> read(Uint8List buf) {
    final index = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
    switch (index) {
      case 1:
        return LiftRetVal(DiagnosticSeverity.error, 4);
      case 2:
        return LiftRetVal(DiagnosticSeverity.warning, 4);
      default:
        throw UniffiInternalError(
          UniffiInternalError.unexpectedEnumCase,
          "Unable to determine enum variant",
        );
    }
  }

  static DiagnosticSeverity lift(RustBuffer buffer) {
    return FfiConverterDiagnosticSeverity.read(buffer.asUint8List()).value;
  }

  static RustBuffer lower(DiagnosticSeverity input) {
    return toRustBuffer(createUint8ListFromInt(input.index + 1));
  }

  static int allocationSize(DiagnosticSeverity _value) {
    return 4;
  }

  static int write(DiagnosticSeverity value, Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, value.index + 1);
    return 4;
  }
}

abstract class ExportFormat {
  RustBuffer lower();
  int allocationSize();
  int write(Uint8List buf);
}

class FfiConverterExportFormat {
  static ExportFormat lift(RustBuffer buffer) {
    return FfiConverterExportFormat.read(buffer.asUint8List()).value;
  }

  static LiftRetVal<ExportFormat> read(Uint8List buf) {
    final index = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
    final subview = Uint8List.view(buf.buffer, buf.offsetInBytes + 4);
    switch (index) {
      case 1:
        final lifted = PdfExportFormat.read(subview);
        return LiftRetVal<ExportFormat>(
          lifted.value,
          lifted.bytesRead - subview.offsetInBytes + 4,
        );
      case 2:
        final lifted = PngExportFormat.read(subview);
        return LiftRetVal<ExportFormat>(
          lifted.value,
          lifted.bytesRead - subview.offsetInBytes + 4,
        );
      case 3:
        final lifted = SvgExportFormat.read(subview);
        return LiftRetVal<ExportFormat>(
          lifted.value,
          lifted.bytesRead - subview.offsetInBytes + 4,
        );
      default:
        throw UniffiInternalError(
          UniffiInternalError.unexpectedEnumCase,
          "Unable to determine enum variant",
        );
    }
  }

  static RustBuffer lower(ExportFormat value) {
    return value.lower();
  }

  static int allocationSize(ExportFormat value) {
    return value.allocationSize();
  }

  static int write(ExportFormat value, Uint8List buf) {
    return value.write(buf) - buf.offsetInBytes;
  }
}

class PdfExportFormat extends ExportFormat {
  PdfExportFormat();
  PdfExportFormat._();
  static LiftRetVal<PdfExportFormat> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    return LiftRetVal(PdfExportFormat._(), new_offset);
  }

  @override
  RustBuffer lower() {
    final buf = Uint8List(allocationSize());
    write(buf);
    return toRustBuffer(buf);
  }

  @override
  int allocationSize() {
    return 4;
  }

  @override
  int write(Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, 1);
    int new_offset = buf.offsetInBytes + 4;
    return new_offset;
  }
}

class PngExportFormat extends ExportFormat {
  final double pixelsPerPt;
  PngExportFormat(double this.pixelsPerPt);
  PngExportFormat._(double this.pixelsPerPt);
  static LiftRetVal<PngExportFormat> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    final pixelsPerPt_lifted = FfiConverterDouble32.read(
      Uint8List.view(buf.buffer, new_offset),
    );
    final pixelsPerPt = pixelsPerPt_lifted.value;
    new_offset += pixelsPerPt_lifted.bytesRead;
    return LiftRetVal(PngExportFormat._(pixelsPerPt), new_offset);
  }

  @override
  RustBuffer lower() {
    final buf = Uint8List(allocationSize());
    write(buf);
    return toRustBuffer(buf);
  }

  @override
  int allocationSize() {
    return FfiConverterDouble32.allocationSize(pixelsPerPt) + 4;
  }

  @override
  int write(Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, 2);
    int new_offset = buf.offsetInBytes + 4;
    new_offset += FfiConverterDouble32.write(
      pixelsPerPt,
      Uint8List.view(buf.buffer, new_offset),
    );
    return new_offset;
  }
}

class SvgExportFormat extends ExportFormat {
  SvgExportFormat();
  SvgExportFormat._();
  static LiftRetVal<SvgExportFormat> read(Uint8List buf) {
    int new_offset = buf.offsetInBytes;
    return LiftRetVal(SvgExportFormat._(), new_offset);
  }

  @override
  RustBuffer lower() {
    final buf = Uint8List(allocationSize());
    write(buf);
    return toRustBuffer(buf);
  }

  @override
  int allocationSize() {
    return 4;
  }

  @override
  int write(Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, 3);
    int new_offset = buf.offsetInBytes + 4;
    return new_offset;
  }
}

abstract class CompiledDocumentInterface {
  Uint8List export_({required ExportFormat format, PageRange? pages = null});
  List<PageSize> pages();
  List<Diagnostic> warnings();
}

final _CompiledDocumentFinalizer = Finalizer<Pointer<Void>>((ptr) {
  rustCall(
    (status) => uniffi_oicana_uniffi_fn_free_compileddocument(ptr, status),
  );
});

class CompiledDocument implements CompiledDocumentInterface {
  late final Pointer<Void> _ptr;
  CompiledDocument._(this._ptr) {
    _CompiledDocumentFinalizer.attach(this, _ptr, detach: this);
  }
  factory CompiledDocument.lift(Pointer<Void> ptr) {
    return CompiledDocument._(ptr);
  }
  Pointer<Void> uniffiClonePointer() {
    return rustCall(
      (status) => uniffi_oicana_uniffi_fn_clone_compileddocument(_ptr, status),
    );
  }

  void dispose() {
    _CompiledDocumentFinalizer.detach(this);
    rustCall(
      (status) => uniffi_oicana_uniffi_fn_free_compileddocument(_ptr, status),
    );
  }

  Uint8List export_({required ExportFormat format, PageRange? pages = null}) {
    return rustCallWithLifter(
      (status) => uniffi_oicana_uniffi_fn_method_compileddocument_export(
        uniffiClonePointer(),
        FfiConverterExportFormat.lower(format),
        FfiConverterOptionalPageRange.lower(pages),
        status,
      ),
      FfiConverterUint8List.lift,
      oicanaExceptionErrorHandler,
    );
  }

  List<PageSize> pages() {
    return rustCallWithLifter(
      (status) => uniffi_oicana_uniffi_fn_method_compileddocument_pages(
        uniffiClonePointer(),
        status,
      ),
      FfiConverterSequencePageSize.lift,
      null,
    );
  }

  List<Diagnostic> warnings() {
    return rustCallWithLifter(
      (status) => uniffi_oicana_uniffi_fn_method_compileddocument_warnings(
        uniffiClonePointer(),
        status,
      ),
      FfiConverterSequenceDiagnostic.lift,
      null,
    );
  }
}

class FfiConverterCompiledDocument {
  static CompiledDocument lift(Pointer<Void> ptr) {
    return CompiledDocument.lift(ptr);
  }

  static Pointer<Void> lower(CompiledDocument value) {
    return value.uniffiClonePointer();
  }

  static int allocationSize(CompiledDocument value) {
    return 8;
  }

  static LiftRetVal<CompiledDocument> read(Uint8List buf) {
    final handle = buf.buffer.asByteData(buf.offsetInBytes).getInt64(0);
    final pointer = Pointer<Void>.fromAddress(handle);
    return LiftRetVal(CompiledDocument.lift(pointer), 8);
  }

  static int write(CompiledDocument value, Uint8List buf) {
    final handle = lower(value);
    buf.buffer.asByteData(buf.offsetInBytes).setInt64(0, handle.address);
    return 8;
  }
}

abstract class TemplateInterface {
  CompiledDocument compile({
    required Map<String, String> jsonInputs,
    required Map<String, BlobInput> blobInputs,
    required CompilationMode mode,
  });
  Uint8List file({required String path});
  String manifest();
  void setValidateInputs({required bool validate});
  String source({required String path});
  List<Diagnostic> warnings();
}

final _TemplateFinalizer = Finalizer<Pointer<Void>>((ptr) {
  rustCall((status) => uniffi_oicana_uniffi_fn_free_template(ptr, status));
});

class Template implements TemplateInterface {
  late final Pointer<Void> _ptr;
  Template._(this._ptr) {
    _TemplateFinalizer.attach(this, _ptr, detach: this);
  }
  Template({
    required Uint8List files,
    required Map<String, String> jsonInputs,
    required Map<String, BlobInput> blobInputs,
    required CompilationMode mode,
    ZipLimits? limits = null,
  }) : _ptr = rustCall(
         (status) => uniffi_oicana_uniffi_fn_constructor_template_new(
           FfiConverterUint8List.lower(files),
           FfiConverterMapStringToString.lower(jsonInputs),
           FfiConverterMapStringToBlobInput.lower(blobInputs),
           FfiConverterCompilationMode.lower(mode),
           FfiConverterOptionalZipLimits.lower(limits),
           status,
         ),
         oicanaExceptionErrorHandler,
       ) {
    _TemplateFinalizer.attach(this, _ptr, detach: this);
  }
  factory Template.lift(Pointer<Void> ptr) {
    return Template._(ptr);
  }
  Pointer<Void> uniffiClonePointer() {
    return rustCall(
      (status) => uniffi_oicana_uniffi_fn_clone_template(_ptr, status),
    );
  }

  void dispose() {
    _TemplateFinalizer.detach(this);
    rustCall((status) => uniffi_oicana_uniffi_fn_free_template(_ptr, status));
  }

  CompiledDocument compile({
    required Map<String, String> jsonInputs,
    required Map<String, BlobInput> blobInputs,
    required CompilationMode mode,
  }) {
    return rustCallWithLifter(
      (status) => uniffi_oicana_uniffi_fn_method_template_compile(
        uniffiClonePointer(),
        FfiConverterMapStringToString.lower(jsonInputs),
        FfiConverterMapStringToBlobInput.lower(blobInputs),
        FfiConverterCompilationMode.lower(mode),
        status,
      ),
      FfiConverterCompiledDocument.lift,
      oicanaExceptionErrorHandler,
    );
  }

  Uint8List file({required String path}) {
    return rustCallWithLifter(
      (status) => uniffi_oicana_uniffi_fn_method_template_file(
        uniffiClonePointer(),
        FfiConverterString.lower(path),
        status,
      ),
      FfiConverterUint8List.lift,
      oicanaExceptionErrorHandler,
    );
  }

  String manifest() {
    return rustCallWithLifter(
      (status) => uniffi_oicana_uniffi_fn_method_template_manifest(
        uniffiClonePointer(),
        status,
      ),
      FfiConverterString.lift,
      oicanaExceptionErrorHandler,
    );
  }

  void setValidateInputs({required bool validate}) {
    return rustCall((status) {
      uniffi_oicana_uniffi_fn_method_template_set_validate_inputs(
        uniffiClonePointer(),
        FfiConverterBool.lower(validate),
        status,
      );
    }, oicanaExceptionErrorHandler);
  }

  String source({required String path}) {
    return rustCallWithLifter(
      (status) => uniffi_oicana_uniffi_fn_method_template_source(
        uniffiClonePointer(),
        FfiConverterString.lower(path),
        status,
      ),
      FfiConverterString.lift,
      oicanaExceptionErrorHandler,
    );
  }

  List<Diagnostic> warnings() {
    return rustCallWithLifter(
      (status) => uniffi_oicana_uniffi_fn_method_template_warnings(
        uniffiClonePointer(),
        status,
      ),
      FfiConverterSequenceDiagnostic.lift,
      null,
    );
  }
}

class FfiConverterTemplate {
  static Template lift(Pointer<Void> ptr) {
    return Template.lift(ptr);
  }

  static Pointer<Void> lower(Template value) {
    return value.uniffiClonePointer();
  }

  static int allocationSize(Template value) {
    return 8;
  }

  static LiftRetVal<Template> read(Uint8List buf) {
    final handle = buf.buffer.asByteData(buf.offsetInBytes).getInt64(0);
    final pointer = Pointer<Void>.fromAddress(handle);
    return LiftRetVal(Template.lift(pointer), 8);
  }

  static int write(Template value, Uint8List buf) {
    final handle = lower(value);
    buf.buffer.asByteData(buf.offsetInBytes).setInt64(0, handle.address);
    return 8;
  }
}

class FfiConverterBool {
  static bool lift(int value) {
    return value == 1;
  }

  static int lower(bool value) {
    return value ? 1 : 0;
  }

  static LiftRetVal<bool> read(Uint8List buf) {
    return LiftRetVal(FfiConverterBool.lift(buf.first), 1);
  }

  static RustBuffer lowerIntoRustBuffer(bool value) {
    return toRustBuffer(Uint8List.fromList([FfiConverterBool.lower(value)]));
  }

  static int allocationSize([bool value = false]) {
    return 1;
  }

  static int write(bool value, Uint8List buf) {
    buf.setAll(0, [value ? 1 : 0]);
    return allocationSize();
  }
}

class FfiConverterDouble32 {
  static double lift(double value) => value;
  static LiftRetVal<double> read(Uint8List buf) {
    return LiftRetVal(
      buf.buffer.asByteData(buf.offsetInBytes).getFloat32(0),
      4,
    );
  }

  static double lower(double value) => value;
  static int allocationSize([double value = 0]) {
    return 4;
  }

  static int write(double value, Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setFloat32(0, value);
    return FfiConverterDouble32.allocationSize();
  }
}

class FfiConverterDouble64 {
  static double lift(double value) => value;
  static LiftRetVal<double> read(Uint8List buf) {
    return LiftRetVal(
      buf.buffer.asByteData(buf.offsetInBytes).getFloat64(0),
      8,
    );
  }

  static double lower(double value) => value;
  static int allocationSize([double value = 0]) {
    return 8;
  }

  static int write(double value, Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setFloat64(0, value);
    return FfiConverterDouble64.allocationSize();
  }
}

class FfiConverterMapStringToBlobInput {
  static Map<String, BlobInput> lift(RustBuffer buf) {
    return FfiConverterMapStringToBlobInput.read(buf.asUint8List()).value;
  }

  static LiftRetVal<Map<String, BlobInput>> read(Uint8List buf) {
    final map = <String, BlobInput>{};
    final length = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
    int offset = buf.offsetInBytes + 4;
    for (var i = 0; i < length; i++) {
      final k = FfiConverterString.read(Uint8List.view(buf.buffer, offset));
      offset += k.bytesRead;
      final v = FfiConverterBlobInput.read(Uint8List.view(buf.buffer, offset));
      offset += v.bytesRead;
      map[k.value] = v.value;
    }
    return LiftRetVal(map, offset - buf.offsetInBytes);
  }

  static int write(Map<String, BlobInput> value, Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, value.length);
    int offset = buf.offsetInBytes + 4;
    for (final entry in value.entries) {
      offset += FfiConverterString.write(
        entry.key,
        Uint8List.view(buf.buffer, offset),
      );
      offset += FfiConverterBlobInput.write(
        entry.value,
        Uint8List.view(buf.buffer, offset),
      );
    }
    return offset - buf.offsetInBytes;
  }

  static int allocationSize(Map<String, BlobInput> value) {
    return value.entries
        .map(
          (e) =>
              FfiConverterString.allocationSize(e.key) +
              FfiConverterBlobInput.allocationSize(e.value),
        )
        .fold(4, (a, b) => a + b);
  }

  static RustBuffer lower(Map<String, BlobInput> value) {
    final buf = Uint8List(allocationSize(value));
    write(value, buf);
    return toRustBuffer(buf);
  }
}

class FfiConverterMapStringToString {
  static Map<String, String> lift(RustBuffer buf) {
    return FfiConverterMapStringToString.read(buf.asUint8List()).value;
  }

  static LiftRetVal<Map<String, String>> read(Uint8List buf) {
    final map = <String, String>{};
    final length = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
    int offset = buf.offsetInBytes + 4;
    for (var i = 0; i < length; i++) {
      final k = FfiConverterString.read(Uint8List.view(buf.buffer, offset));
      offset += k.bytesRead;
      final v = FfiConverterString.read(Uint8List.view(buf.buffer, offset));
      offset += v.bytesRead;
      map[k.value] = v.value;
    }
    return LiftRetVal(map, offset - buf.offsetInBytes);
  }

  static int write(Map<String, String> value, Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, value.length);
    int offset = buf.offsetInBytes + 4;
    for (final entry in value.entries) {
      offset += FfiConverterString.write(
        entry.key,
        Uint8List.view(buf.buffer, offset),
      );
      offset += FfiConverterString.write(
        entry.value,
        Uint8List.view(buf.buffer, offset),
      );
    }
    return offset - buf.offsetInBytes;
  }

  static int allocationSize(Map<String, String> value) {
    return value.entries
        .map(
          (e) =>
              FfiConverterString.allocationSize(e.key) +
              FfiConverterString.allocationSize(e.value),
        )
        .fold(4, (a, b) => a + b);
  }

  static RustBuffer lower(Map<String, String> value) {
    final buf = Uint8List(allocationSize(value));
    write(value, buf);
    return toRustBuffer(buf);
  }
}

class FfiConverterOptionalPageRange {
  static PageRange? lift(RustBuffer buf) {
    return FfiConverterOptionalPageRange.read(buf.asUint8List()).value;
  }

  static LiftRetVal<PageRange?> read(Uint8List buf) {
    if (ByteData.view(buf.buffer, buf.offsetInBytes).getInt8(0) == 0) {
      return LiftRetVal(null, 1);
    }
    final result = FfiConverterPageRange.read(
      Uint8List.view(buf.buffer, buf.offsetInBytes + 1),
    );
    return LiftRetVal<PageRange?>(result.value, result.bytesRead + 1);
  }

  static int allocationSize([PageRange? value]) {
    if (value == null) {
      return 1;
    }
    return FfiConverterPageRange.allocationSize(value) + 1;
  }

  static RustBuffer lower(PageRange? value) {
    if (value == null) {
      return toRustBuffer(Uint8List.fromList([0]));
    }
    final length = FfiConverterOptionalPageRange.allocationSize(value);
    final Pointer<Uint8> frameData = calloc<Uint8>(length);
    final buf = frameData.asTypedList(length);
    FfiConverterOptionalPageRange.write(value, buf);
    final bytes = calloc<ForeignBytes>();
    bytes.ref.len = length;
    bytes.ref.data = frameData;
    return RustBuffer.fromBytes(bytes.ref);
  }

  static int write(PageRange? value, Uint8List buf) {
    if (value == null) {
      buf[0] = 0;
      return 1;
    }
    buf[0] = 1;
    return FfiConverterPageRange.write(
          value,
          Uint8List.view(buf.buffer, buf.offsetInBytes + 1),
        ) +
        1;
  }
}

class FfiConverterOptionalSourceLocation {
  static SourceLocation? lift(RustBuffer buf) {
    return FfiConverterOptionalSourceLocation.read(buf.asUint8List()).value;
  }

  static LiftRetVal<SourceLocation?> read(Uint8List buf) {
    if (ByteData.view(buf.buffer, buf.offsetInBytes).getInt8(0) == 0) {
      return LiftRetVal(null, 1);
    }
    final result = FfiConverterSourceLocation.read(
      Uint8List.view(buf.buffer, buf.offsetInBytes + 1),
    );
    return LiftRetVal<SourceLocation?>(result.value, result.bytesRead + 1);
  }

  static int allocationSize([SourceLocation? value]) {
    if (value == null) {
      return 1;
    }
    return FfiConverterSourceLocation.allocationSize(value) + 1;
  }

  static RustBuffer lower(SourceLocation? value) {
    if (value == null) {
      return toRustBuffer(Uint8List.fromList([0]));
    }
    final length = FfiConverterOptionalSourceLocation.allocationSize(value);
    final Pointer<Uint8> frameData = calloc<Uint8>(length);
    final buf = frameData.asTypedList(length);
    FfiConverterOptionalSourceLocation.write(value, buf);
    final bytes = calloc<ForeignBytes>();
    bytes.ref.len = length;
    bytes.ref.data = frameData;
    return RustBuffer.fromBytes(bytes.ref);
  }

  static int write(SourceLocation? value, Uint8List buf) {
    if (value == null) {
      buf[0] = 0;
      return 1;
    }
    buf[0] = 1;
    return FfiConverterSourceLocation.write(
          value,
          Uint8List.view(buf.buffer, buf.offsetInBytes + 1),
        ) +
        1;
  }
}

class FfiConverterOptionalString {
  static String? lift(RustBuffer buf) {
    return FfiConverterOptionalString.read(buf.asUint8List()).value;
  }

  static LiftRetVal<String?> read(Uint8List buf) {
    if (ByteData.view(buf.buffer, buf.offsetInBytes).getInt8(0) == 0) {
      return LiftRetVal(null, 1);
    }
    final result = FfiConverterString.read(
      Uint8List.view(buf.buffer, buf.offsetInBytes + 1),
    );
    return LiftRetVal<String?>(result.value, result.bytesRead + 1);
  }

  static int allocationSize([String? value]) {
    if (value == null) {
      return 1;
    }
    return FfiConverterString.allocationSize(value) + 1;
  }

  static RustBuffer lower(String? value) {
    if (value == null) {
      return toRustBuffer(Uint8List.fromList([0]));
    }
    final length = FfiConverterOptionalString.allocationSize(value);
    final Pointer<Uint8> frameData = calloc<Uint8>(length);
    final buf = frameData.asTypedList(length);
    FfiConverterOptionalString.write(value, buf);
    final bytes = calloc<ForeignBytes>();
    bytes.ref.len = length;
    bytes.ref.data = frameData;
    return RustBuffer.fromBytes(bytes.ref);
  }

  static int write(String? value, Uint8List buf) {
    if (value == null) {
      buf[0] = 0;
      return 1;
    }
    buf[0] = 1;
    return FfiConverterString.write(
          value,
          Uint8List.view(buf.buffer, buf.offsetInBytes + 1),
        ) +
        1;
  }
}

class FfiConverterOptionalUInt64 {
  static int? lift(RustBuffer buf) {
    return FfiConverterOptionalUInt64.read(buf.asUint8List()).value;
  }

  static LiftRetVal<int?> read(Uint8List buf) {
    if (ByteData.view(buf.buffer, buf.offsetInBytes).getInt8(0) == 0) {
      return LiftRetVal(null, 1);
    }
    final result = FfiConverterUInt64.read(
      Uint8List.view(buf.buffer, buf.offsetInBytes + 1),
    );
    return LiftRetVal<int?>(result.value, result.bytesRead + 1);
  }

  static int allocationSize([int? value]) {
    if (value == null) {
      return 1;
    }
    return FfiConverterUInt64.allocationSize(value) + 1;
  }

  static RustBuffer lower(int? value) {
    if (value == null) {
      return toRustBuffer(Uint8List.fromList([0]));
    }
    final length = FfiConverterOptionalUInt64.allocationSize(value);
    final Pointer<Uint8> frameData = calloc<Uint8>(length);
    final buf = frameData.asTypedList(length);
    FfiConverterOptionalUInt64.write(value, buf);
    final bytes = calloc<ForeignBytes>();
    bytes.ref.len = length;
    bytes.ref.data = frameData;
    return RustBuffer.fromBytes(bytes.ref);
  }

  static int write(int? value, Uint8List buf) {
    if (value == null) {
      buf[0] = 0;
      return 1;
    }
    buf[0] = 1;
    return FfiConverterUInt64.write(
          value,
          Uint8List.view(buf.buffer, buf.offsetInBytes + 1),
        ) +
        1;
  }
}

class FfiConverterOptionalZipLimits {
  static ZipLimits? lift(RustBuffer buf) {
    return FfiConverterOptionalZipLimits.read(buf.asUint8List()).value;
  }

  static LiftRetVal<ZipLimits?> read(Uint8List buf) {
    if (ByteData.view(buf.buffer, buf.offsetInBytes).getInt8(0) == 0) {
      return LiftRetVal(null, 1);
    }
    final result = FfiConverterZipLimits.read(
      Uint8List.view(buf.buffer, buf.offsetInBytes + 1),
    );
    return LiftRetVal<ZipLimits?>(result.value, result.bytesRead + 1);
  }

  static int allocationSize([ZipLimits? value]) {
    if (value == null) {
      return 1;
    }
    return FfiConverterZipLimits.allocationSize(value) + 1;
  }

  static RustBuffer lower(ZipLimits? value) {
    if (value == null) {
      return toRustBuffer(Uint8List.fromList([0]));
    }
    final length = FfiConverterOptionalZipLimits.allocationSize(value);
    final Pointer<Uint8> frameData = calloc<Uint8>(length);
    final buf = frameData.asTypedList(length);
    FfiConverterOptionalZipLimits.write(value, buf);
    final bytes = calloc<ForeignBytes>();
    bytes.ref.len = length;
    bytes.ref.data = frameData;
    return RustBuffer.fromBytes(bytes.ref);
  }

  static int write(ZipLimits? value, Uint8List buf) {
    if (value == null) {
      buf[0] = 0;
      return 1;
    }
    buf[0] = 1;
    return FfiConverterZipLimits.write(
          value,
          Uint8List.view(buf.buffer, buf.offsetInBytes + 1),
        ) +
        1;
  }
}

class FfiConverterSequenceDiagnostic {
  static List<Diagnostic> lift(RustBuffer buf) {
    return FfiConverterSequenceDiagnostic.read(buf.asUint8List()).value;
  }

  static LiftRetVal<List<Diagnostic>> read(Uint8List buf) {
    List<Diagnostic> res = [];
    final length = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
    int offset = buf.offsetInBytes + 4;
    for (var i = 0; i < length; i++) {
      final ret = FfiConverterDiagnostic.read(
        Uint8List.view(buf.buffer, offset),
      );
      offset += ret.bytesRead;
      res.add(ret.value);
    }
    return LiftRetVal(res, offset - buf.offsetInBytes);
  }

  static int write(List<Diagnostic> value, Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, value.length);
    int offset = buf.offsetInBytes + 4;
    for (var i = 0; i < value.length; i++) {
      offset += FfiConverterDiagnostic.write(
        value[i],
        Uint8List.view(buf.buffer, offset),
      );
    }
    return offset - buf.offsetInBytes;
  }

  static int allocationSize(List<Diagnostic> value) {
    return value
            .map((l) => FfiConverterDiagnostic.allocationSize(l))
            .fold(0, (a, b) => a + b) +
        4;
  }

  static RustBuffer lower(List<Diagnostic> value) {
    final buf = Uint8List(allocationSize(value));
    write(value, buf);
    return toRustBuffer(buf);
  }
}

class FfiConverterSequencePageSize {
  static List<PageSize> lift(RustBuffer buf) {
    return FfiConverterSequencePageSize.read(buf.asUint8List()).value;
  }

  static LiftRetVal<List<PageSize>> read(Uint8List buf) {
    List<PageSize> res = [];
    final length = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
    int offset = buf.offsetInBytes + 4;
    for (var i = 0; i < length; i++) {
      final ret = FfiConverterPageSize.read(Uint8List.view(buf.buffer, offset));
      offset += ret.bytesRead;
      res.add(ret.value);
    }
    return LiftRetVal(res, offset - buf.offsetInBytes);
  }

  static int write(List<PageSize> value, Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, value.length);
    int offset = buf.offsetInBytes + 4;
    for (var i = 0; i < value.length; i++) {
      offset += FfiConverterPageSize.write(
        value[i],
        Uint8List.view(buf.buffer, offset),
      );
    }
    return offset - buf.offsetInBytes;
  }

  static int allocationSize(List<PageSize> value) {
    return value
            .map((l) => FfiConverterPageSize.allocationSize(l))
            .fold(0, (a, b) => a + b) +
        4;
  }

  static RustBuffer lower(List<PageSize> value) {
    final buf = Uint8List(allocationSize(value));
    write(value, buf);
    return toRustBuffer(buf);
  }
}

class FfiConverterSequenceRegisteredFont {
  static List<RegisteredFont> lift(RustBuffer buf) {
    return FfiConverterSequenceRegisteredFont.read(buf.asUint8List()).value;
  }

  static LiftRetVal<List<RegisteredFont>> read(Uint8List buf) {
    List<RegisteredFont> res = [];
    final length = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
    int offset = buf.offsetInBytes + 4;
    for (var i = 0; i < length; i++) {
      final ret = FfiConverterRegisteredFont.read(
        Uint8List.view(buf.buffer, offset),
      );
      offset += ret.bytesRead;
      res.add(ret.value);
    }
    return LiftRetVal(res, offset - buf.offsetInBytes);
  }

  static int write(List<RegisteredFont> value, Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, value.length);
    int offset = buf.offsetInBytes + 4;
    for (var i = 0; i < value.length; i++) {
      offset += FfiConverterRegisteredFont.write(
        value[i],
        Uint8List.view(buf.buffer, offset),
      );
    }
    return offset - buf.offsetInBytes;
  }

  static int allocationSize(List<RegisteredFont> value) {
    return value
            .map((l) => FfiConverterRegisteredFont.allocationSize(l))
            .fold(0, (a, b) => a + b) +
        4;
  }

  static RustBuffer lower(List<RegisteredFont> value) {
    final buf = Uint8List(allocationSize(value));
    write(value, buf);
    return toRustBuffer(buf);
  }
}

class FfiConverterSequenceString {
  static List<String> lift(RustBuffer buf) {
    return FfiConverterSequenceString.read(buf.asUint8List()).value;
  }

  static LiftRetVal<List<String>> read(Uint8List buf) {
    List<String> res = [];
    final length = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
    int offset = buf.offsetInBytes + 4;
    for (var i = 0; i < length; i++) {
      final ret = FfiConverterString.read(Uint8List.view(buf.buffer, offset));
      offset += ret.bytesRead;
      res.add(ret.value);
    }
    return LiftRetVal(res, offset - buf.offsetInBytes);
  }

  static int write(List<String> value, Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, value.length);
    int offset = buf.offsetInBytes + 4;
    for (var i = 0; i < value.length; i++) {
      offset += FfiConverterString.write(
        value[i],
        Uint8List.view(buf.buffer, offset),
      );
    }
    return offset - buf.offsetInBytes;
  }

  static int allocationSize(List<String> value) {
    return value
            .map((l) => FfiConverterString.allocationSize(l))
            .fold(0, (a, b) => a + b) +
        4;
  }

  static RustBuffer lower(List<String> value) {
    final buf = Uint8List(allocationSize(value));
    write(value, buf);
    return toRustBuffer(buf);
  }
}

class FfiConverterSequenceTracePoint {
  static List<TracePoint> lift(RustBuffer buf) {
    return FfiConverterSequenceTracePoint.read(buf.asUint8List()).value;
  }

  static LiftRetVal<List<TracePoint>> read(Uint8List buf) {
    List<TracePoint> res = [];
    final length = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
    int offset = buf.offsetInBytes + 4;
    for (var i = 0; i < length; i++) {
      final ret = FfiConverterTracePoint.read(
        Uint8List.view(buf.buffer, offset),
      );
      offset += ret.bytesRead;
      res.add(ret.value);
    }
    return LiftRetVal(res, offset - buf.offsetInBytes);
  }

  static int write(List<TracePoint> value, Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, value.length);
    int offset = buf.offsetInBytes + 4;
    for (var i = 0; i < value.length; i++) {
      offset += FfiConverterTracePoint.write(
        value[i],
        Uint8List.view(buf.buffer, offset),
      );
    }
    return offset - buf.offsetInBytes;
  }

  static int allocationSize(List<TracePoint> value) {
    return value
            .map((l) => FfiConverterTracePoint.allocationSize(l))
            .fold(0, (a, b) => a + b) +
        4;
  }

  static RustBuffer lower(List<TracePoint> value) {
    final buf = Uint8List(allocationSize(value));
    write(value, buf);
    return toRustBuffer(buf);
  }
}

class FfiConverterSequenceUint8List {
  static List<Uint8List> lift(RustBuffer buf) {
    return FfiConverterSequenceUint8List.read(buf.asUint8List()).value;
  }

  static LiftRetVal<List<Uint8List>> read(Uint8List buf) {
    List<Uint8List> res = [];
    final length = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
    int offset = buf.offsetInBytes + 4;
    for (var i = 0; i < length; i++) {
      final ret = FfiConverterUint8List.read(
        Uint8List.view(buf.buffer, offset),
      );
      offset += ret.bytesRead;
      res.add(ret.value);
    }
    return LiftRetVal(res, offset - buf.offsetInBytes);
  }

  static int write(List<Uint8List> value, Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, value.length);
    int offset = buf.offsetInBytes + 4;
    for (var i = 0; i < value.length; i++) {
      offset += FfiConverterUint8List.write(
        value[i],
        Uint8List.view(buf.buffer, offset),
      );
    }
    return offset - buf.offsetInBytes;
  }

  static int allocationSize(List<Uint8List> value) {
    return value
            .map((l) => FfiConverterUint8List.allocationSize(l))
            .fold(0, (a, b) => a + b) +
        4;
  }

  static RustBuffer lower(List<Uint8List> value) {
    final buf = Uint8List(allocationSize(value));
    write(value, buf);
    return toRustBuffer(buf);
  }
}

class FfiConverterUInt32 {
  static int lift(int value) => value;
  static LiftRetVal<int> read(Uint8List buf) {
    return LiftRetVal(buf.buffer.asByteData(buf.offsetInBytes).getUint32(0), 4);
  }

  static int lower(int value) {
    if (value < 0 || value > 4294967295) {
      throw ArgumentError("Value out of range for u32: " + value.toString());
    }
    return value;
  }

  static int allocationSize([int value = 0]) {
    return 4;
  }

  static int write(int value, Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setUint32(0, lower(value));
    return 4;
  }
}

class FfiConverterUInt64 {
  static int lift(int value) => value;
  static LiftRetVal<int> read(Uint8List buf) {
    return LiftRetVal(buf.buffer.asByteData(buf.offsetInBytes).getUint64(0), 8);
  }

  static int lower(int value) {
    if (value < 0) {
      throw ArgumentError("Value out of range for u64: " + value.toString());
    }
    return value;
  }

  static int allocationSize([int value = 0]) {
    return 8;
  }

  static int write(int value, Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setUint64(0, lower(value));
    return 8;
  }
}

class FfiConverterUint8List {
  static Uint8List lift(RustBuffer value) {
    return FfiConverterUint8List.read(value.asUint8List()).value;
  }

  static LiftRetVal<Uint8List> read(Uint8List buf) {
    final length = buf.buffer.asByteData(buf.offsetInBytes).getInt32(0);
    final bytes = Uint8List.view(buf.buffer, buf.offsetInBytes + 4, length);
    return LiftRetVal(bytes, length + 4);
  }

  static RustBuffer lower(Uint8List value) {
    final buf = Uint8List(allocationSize(value));
    write(value, buf);
    return toRustBuffer(buf);
  }

  static int allocationSize([Uint8List? value]) {
    if (value == null) {
      return 4;
    }
    return 4 + value.length;
  }

  static int write(Uint8List value, Uint8List buf) {
    buf.buffer.asByteData(buf.offsetInBytes).setInt32(0, value.length);
    buf.setRange(4, 4 + value.length, value);
    return 4 + value.length;
  }
}

const _uniffiAssetId = "package:oicana/uniffi:oicana_uniffi";
void clearFonts() {
  return rustCall((status) {
    uniffi_oicana_uniffi_fn_func_clear_fonts(status);
  }, null);
}

void configureAutomaticCacheEviction({required int? maxAge}) {
  return rustCall((status) {
    uniffi_oicana_uniffi_fn_func_configure_automatic_cache_eviction(
      FfiConverterOptionalUInt64.lower(maxAge),
      status,
    );
  }, null);
}

void configureDiagnosticColor({required DiagnosticColor color}) {
  return rustCall((status) {
    uniffi_oicana_uniffi_fn_func_configure_diagnostic_color(
      FfiConverterDiagnosticColor.lower(color),
      status,
    );
  }, null);
}

void evictCache({required int maxAge}) {
  return rustCall((status) {
    uniffi_oicana_uniffi_fn_func_evict_cache(
      FfiConverterUInt64.lower(maxAge),
      status,
    );
  }, null);
}

ExportOnceResult exportOnce({
  required Uint8List files,
  required Map<String, String> jsonInputs,
  required Map<String, BlobInput> blobInputs,
  required CompilationMode mode,
  required ExportFormat format,
  PageRange? pages = null,
  ZipLimits? limits = null,
}) {
  return rustCallWithLifter(
    (status) => uniffi_oicana_uniffi_fn_func_export_once(
      FfiConverterUint8List.lower(files),
      FfiConverterMapStringToString.lower(jsonInputs),
      FfiConverterMapStringToBlobInput.lower(blobInputs),
      FfiConverterCompilationMode.lower(mode),
      FfiConverterExportFormat.lower(format),
      FfiConverterOptionalPageRange.lower(pages),
      FfiConverterOptionalZipLimits.lower(limits),
      status,
    ),
    FfiConverterExportOnceResult.lift,
    oicanaExceptionErrorHandler,
  );
}

int registerFontPaths({required List<String> paths}) {
  return rustCallWithLifter(
    (status) => uniffi_oicana_uniffi_fn_func_register_font_paths(
      FfiConverterSequenceString.lower(paths),
      status,
    ),
    FfiConverterUInt64.lift,
    null,
  );
}

int registerFonts({required List<Uint8List> fonts}) {
  return rustCallWithLifter(
    (status) => uniffi_oicana_uniffi_fn_func_register_fonts(
      FfiConverterSequenceUint8List.lower(fonts),
      status,
    ),
    FfiConverterUInt64.lift,
    null,
  );
}

List<RegisteredFont> registeredFonts() {
  return rustCallWithLifter(
    (status) => uniffi_oicana_uniffi_fn_func_registered_fonts(status),
    FfiConverterSequenceRegisteredFont.lift,
    null,
  );
}

String version() {
  return rustCallWithLifter(
    (status) => uniffi_oicana_uniffi_fn_func_version(status),
    FfiConverterString.lift,
    null,
  );
}

@Native<Pointer<Void> Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external Pointer<Void> uniffi_oicana_uniffi_fn_clone_compileddocument(
  Pointer<Void> handle,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<Void Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external void uniffi_oicana_uniffi_fn_free_compileddocument(
  Pointer<Void> handle,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  RustBuffer Function(
    Pointer<Void>,
    RustBuffer,
    RustBuffer,
    Pointer<RustCallStatus>,
  )
>(assetId: _uniffiAssetId)
external RustBuffer uniffi_oicana_uniffi_fn_method_compileddocument_export(
  Pointer<Void> ptr,
  RustBuffer format,
  RustBuffer pages,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<RustBuffer Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external RustBuffer uniffi_oicana_uniffi_fn_method_compileddocument_pages(
  Pointer<Void> ptr,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<RustBuffer Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external RustBuffer uniffi_oicana_uniffi_fn_method_compileddocument_warnings(
  Pointer<Void> ptr,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<Pointer<Void> Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external Pointer<Void> uniffi_oicana_uniffi_fn_clone_template(
  Pointer<Void> handle,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<Void Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external void uniffi_oicana_uniffi_fn_free_template(
  Pointer<Void> handle,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  Pointer<Void> Function(
    RustBuffer,
    RustBuffer,
    RustBuffer,
    RustBuffer,
    RustBuffer,
    Pointer<RustCallStatus>,
  )
>(assetId: _uniffiAssetId)
external Pointer<Void> uniffi_oicana_uniffi_fn_constructor_template_new(
  RustBuffer files,
  RustBuffer json_inputs,
  RustBuffer blob_inputs,
  RustBuffer mode,
  RustBuffer limits,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  Pointer<Void> Function(
    Pointer<Void>,
    RustBuffer,
    RustBuffer,
    RustBuffer,
    Pointer<RustCallStatus>,
  )
>(assetId: _uniffiAssetId)
external Pointer<Void> uniffi_oicana_uniffi_fn_method_template_compile(
  Pointer<Void> ptr,
  RustBuffer json_inputs,
  RustBuffer blob_inputs,
  RustBuffer mode,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  RustBuffer Function(Pointer<Void>, RustBuffer, Pointer<RustCallStatus>)
>(assetId: _uniffiAssetId)
external RustBuffer uniffi_oicana_uniffi_fn_method_template_file(
  Pointer<Void> ptr,
  RustBuffer path,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<RustBuffer Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external RustBuffer uniffi_oicana_uniffi_fn_method_template_manifest(
  Pointer<Void> ptr,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<Void Function(Pointer<Void>, Int8, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external void uniffi_oicana_uniffi_fn_method_template_set_validate_inputs(
  Pointer<Void> ptr,
  int validate,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  RustBuffer Function(Pointer<Void>, RustBuffer, Pointer<RustCallStatus>)
>(assetId: _uniffiAssetId)
external RustBuffer uniffi_oicana_uniffi_fn_method_template_source(
  Pointer<Void> ptr,
  RustBuffer path,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<RustBuffer Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external RustBuffer uniffi_oicana_uniffi_fn_method_template_warnings(
  Pointer<Void> ptr,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<Void Function(Pointer<RustCallStatus>)>(assetId: _uniffiAssetId)
external void uniffi_oicana_uniffi_fn_func_clear_fonts(
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<Void Function(RustBuffer, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external void uniffi_oicana_uniffi_fn_func_configure_automatic_cache_eviction(
  RustBuffer max_age,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<Void Function(RustBuffer, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external void uniffi_oicana_uniffi_fn_func_configure_diagnostic_color(
  RustBuffer color,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<Void Function(Uint64, Pointer<RustCallStatus>)>(assetId: _uniffiAssetId)
external void uniffi_oicana_uniffi_fn_func_evict_cache(
  int max_age,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  RustBuffer Function(
    RustBuffer,
    RustBuffer,
    RustBuffer,
    RustBuffer,
    RustBuffer,
    RustBuffer,
    RustBuffer,
    Pointer<RustCallStatus>,
  )
>(assetId: _uniffiAssetId)
external RustBuffer uniffi_oicana_uniffi_fn_func_export_once(
  RustBuffer files,
  RustBuffer json_inputs,
  RustBuffer blob_inputs,
  RustBuffer mode,
  RustBuffer format,
  RustBuffer pages,
  RustBuffer limits,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<Uint64 Function(RustBuffer, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external int uniffi_oicana_uniffi_fn_func_register_font_paths(
  RustBuffer paths,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<Uint64 Function(RustBuffer, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external int uniffi_oicana_uniffi_fn_func_register_fonts(
  RustBuffer fonts,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<RustBuffer Function(Pointer<RustCallStatus>)>(assetId: _uniffiAssetId)
external RustBuffer uniffi_oicana_uniffi_fn_func_registered_fonts(
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<RustBuffer Function(Pointer<RustCallStatus>)>(assetId: _uniffiAssetId)
external RustBuffer uniffi_oicana_uniffi_fn_func_version(
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  Void Function(
    Pointer<Void>,
    Pointer<NativeFunction<UniffiRustFutureContinuationCallback>>,
    Pointer<Void>,
  )
>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_poll_u8(
  Pointer<Void> handle,
  Pointer<NativeFunction<UniffiRustFutureContinuationCallback>> callback,
  Pointer<Void> callback_data,
);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_cancel_u8(Pointer<Void> handle);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_free_u8(Pointer<Void> handle);

@Native<Uint8 Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external int ffi_oicana_uniffi_rust_future_complete_u8(
  Pointer<Void> handle,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  Void Function(
    Pointer<Void>,
    Pointer<NativeFunction<UniffiRustFutureContinuationCallback>>,
    Pointer<Void>,
  )
>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_poll_i8(
  Pointer<Void> handle,
  Pointer<NativeFunction<UniffiRustFutureContinuationCallback>> callback,
  Pointer<Void> callback_data,
);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_cancel_i8(Pointer<Void> handle);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_free_i8(Pointer<Void> handle);

@Native<Int8 Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external int ffi_oicana_uniffi_rust_future_complete_i8(
  Pointer<Void> handle,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  Void Function(
    Pointer<Void>,
    Pointer<NativeFunction<UniffiRustFutureContinuationCallback>>,
    Pointer<Void>,
  )
>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_poll_u16(
  Pointer<Void> handle,
  Pointer<NativeFunction<UniffiRustFutureContinuationCallback>> callback,
  Pointer<Void> callback_data,
);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_cancel_u16(Pointer<Void> handle);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_free_u16(Pointer<Void> handle);

@Native<Uint16 Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external int ffi_oicana_uniffi_rust_future_complete_u16(
  Pointer<Void> handle,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  Void Function(
    Pointer<Void>,
    Pointer<NativeFunction<UniffiRustFutureContinuationCallback>>,
    Pointer<Void>,
  )
>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_poll_i16(
  Pointer<Void> handle,
  Pointer<NativeFunction<UniffiRustFutureContinuationCallback>> callback,
  Pointer<Void> callback_data,
);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_cancel_i16(Pointer<Void> handle);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_free_i16(Pointer<Void> handle);

@Native<Int16 Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external int ffi_oicana_uniffi_rust_future_complete_i16(
  Pointer<Void> handle,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  Void Function(
    Pointer<Void>,
    Pointer<NativeFunction<UniffiRustFutureContinuationCallback>>,
    Pointer<Void>,
  )
>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_poll_u32(
  Pointer<Void> handle,
  Pointer<NativeFunction<UniffiRustFutureContinuationCallback>> callback,
  Pointer<Void> callback_data,
);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_cancel_u32(Pointer<Void> handle);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_free_u32(Pointer<Void> handle);

@Native<Uint32 Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external int ffi_oicana_uniffi_rust_future_complete_u32(
  Pointer<Void> handle,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  Void Function(
    Pointer<Void>,
    Pointer<NativeFunction<UniffiRustFutureContinuationCallback>>,
    Pointer<Void>,
  )
>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_poll_i32(
  Pointer<Void> handle,
  Pointer<NativeFunction<UniffiRustFutureContinuationCallback>> callback,
  Pointer<Void> callback_data,
);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_cancel_i32(Pointer<Void> handle);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_free_i32(Pointer<Void> handle);

@Native<Int32 Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external int ffi_oicana_uniffi_rust_future_complete_i32(
  Pointer<Void> handle,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  Void Function(
    Pointer<Void>,
    Pointer<NativeFunction<UniffiRustFutureContinuationCallback>>,
    Pointer<Void>,
  )
>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_poll_u64(
  Pointer<Void> handle,
  Pointer<NativeFunction<UniffiRustFutureContinuationCallback>> callback,
  Pointer<Void> callback_data,
);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_cancel_u64(Pointer<Void> handle);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_free_u64(Pointer<Void> handle);

@Native<Uint64 Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external int ffi_oicana_uniffi_rust_future_complete_u64(
  Pointer<Void> handle,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  Void Function(
    Pointer<Void>,
    Pointer<NativeFunction<UniffiRustFutureContinuationCallback>>,
    Pointer<Void>,
  )
>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_poll_i64(
  Pointer<Void> handle,
  Pointer<NativeFunction<UniffiRustFutureContinuationCallback>> callback,
  Pointer<Void> callback_data,
);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_cancel_i64(Pointer<Void> handle);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_free_i64(Pointer<Void> handle);

@Native<Int64 Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external int ffi_oicana_uniffi_rust_future_complete_i64(
  Pointer<Void> handle,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  Void Function(
    Pointer<Void>,
    Pointer<NativeFunction<UniffiRustFutureContinuationCallback>>,
    Pointer<Void>,
  )
>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_poll_f32(
  Pointer<Void> handle,
  Pointer<NativeFunction<UniffiRustFutureContinuationCallback>> callback,
  Pointer<Void> callback_data,
);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_cancel_f32(Pointer<Void> handle);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_free_f32(Pointer<Void> handle);

@Native<Float Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external double ffi_oicana_uniffi_rust_future_complete_f32(
  Pointer<Void> handle,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  Void Function(
    Pointer<Void>,
    Pointer<NativeFunction<UniffiRustFutureContinuationCallback>>,
    Pointer<Void>,
  )
>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_poll_f64(
  Pointer<Void> handle,
  Pointer<NativeFunction<UniffiRustFutureContinuationCallback>> callback,
  Pointer<Void> callback_data,
);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_cancel_f64(Pointer<Void> handle);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_free_f64(Pointer<Void> handle);

@Native<Double Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external double ffi_oicana_uniffi_rust_future_complete_f64(
  Pointer<Void> handle,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  Void Function(
    Pointer<Void>,
    Pointer<NativeFunction<UniffiRustFutureContinuationCallback>>,
    Pointer<Void>,
  )
>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_poll_rust_buffer(
  Pointer<Void> handle,
  Pointer<NativeFunction<UniffiRustFutureContinuationCallback>> callback,
  Pointer<Void> callback_data,
);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_cancel_rust_buffer(
  Pointer<Void> handle,
);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_free_rust_buffer(
  Pointer<Void> handle,
);

@Native<RustBuffer Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external RustBuffer ffi_oicana_uniffi_rust_future_complete_rust_buffer(
  Pointer<Void> handle,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<
  Void Function(
    Pointer<Void>,
    Pointer<NativeFunction<UniffiRustFutureContinuationCallback>>,
    Pointer<Void>,
  )
>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_poll_void(
  Pointer<Void> handle,
  Pointer<NativeFunction<UniffiRustFutureContinuationCallback>> callback,
  Pointer<Void> callback_data,
);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_cancel_void(Pointer<Void> handle);

@Native<Void Function(Pointer<Void>)>(assetId: _uniffiAssetId)
external void ffi_oicana_uniffi_rust_future_free_void(Pointer<Void> handle);

@Native<Void Function(Pointer<Void>, Pointer<RustCallStatus>)>(
  assetId: _uniffiAssetId,
)
external void ffi_oicana_uniffi_rust_future_complete_void(
  Pointer<Void> handle,
  Pointer<RustCallStatus> uniffiStatus,
);

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_func_clear_fonts();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int
uniffi_oicana_uniffi_checksum_func_configure_automatic_cache_eviction();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_func_configure_diagnostic_color();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_func_evict_cache();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_func_export_once();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_func_register_font_paths();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_func_register_fonts();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_func_registered_fonts();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_func_version();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_method_compileddocument_export();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_method_compileddocument_pages();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_method_compileddocument_warnings();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_method_template_compile();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_method_template_file();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_method_template_manifest();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int
uniffi_oicana_uniffi_checksum_method_template_set_validate_inputs();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_method_template_source();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_method_template_warnings();

@Native<Uint16 Function()>(assetId: _uniffiAssetId)
external int uniffi_oicana_uniffi_checksum_constructor_template_new();

@Native<Uint32 Function()>(assetId: _uniffiAssetId)
external int ffi_oicana_uniffi_uniffi_contract_version();

void _checkApiVersion() {
  final bindingsVersion = 30;
  final scaffoldingVersion = ffi_oicana_uniffi_uniffi_contract_version();
  if (bindingsVersion != scaffoldingVersion) {
    throw UniffiInternalError.panicked(
      "UniFFI contract version mismatch: bindings version \$bindingsVersion, scaffolding version \$scaffoldingVersion",
    );
  }
}

void _checkApiChecksums() {
  if (uniffi_oicana_uniffi_checksum_func_clear_fonts() != 18964) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_func_configure_automatic_cache_eviction() !=
      37373) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_func_configure_diagnostic_color() !=
      47395) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_func_evict_cache() != 29494) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_func_export_once() != 59019) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_func_register_font_paths() != 61184) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_func_register_fonts() != 42024) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_func_registered_fonts() != 17013) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_func_version() != 58374) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_method_compileddocument_export() != 55777) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_method_compileddocument_pages() != 1025) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_method_compileddocument_warnings() !=
      4493) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_method_template_compile() != 35206) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_method_template_file() != 23146) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_method_template_manifest() != 52272) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_method_template_set_validate_inputs() !=
      6499) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_method_template_source() != 24155) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_method_template_warnings() != 11791) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
  if (uniffi_oicana_uniffi_checksum_constructor_template_new() != 60852) {
    throw UniffiInternalError.panicked("UniFFI API checksum mismatch");
  }
}

void ensureInitialized() {
  _checkApiVersion();
  _checkApiChecksums();
}

@Deprecated("Use ensureInitialized instead")
void initialize() {
  ensureInitialized();
}
