/// Generate typeset PDFs from Typst templates, in process. Templates are packed
/// with the Oicana CLI.
///
/// ```dart
/// final template = await Template.load(File('invoice-0.1.0.zip').readAsBytesSync());
/// final pdf = await template.exportPdf(jsonInputs: {'invoice': {'number': '2026-001'}});
/// ```
library;

export 'src/exceptions.dart';
export 'src/functions.dart';
export 'src/manifest.dart';
export 'src/template.dart';
export 'src/types.dart';
