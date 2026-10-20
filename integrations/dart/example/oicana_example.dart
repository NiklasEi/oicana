import 'dart:io';

import 'package:oicana/oicana.dart';

/// Renders a packed template to a PDF:
/// `dart run example/oicana_example.dart invoice-0.1.0.zip invoice.pdf`
Future<void> main(List<String> args) async {
  if (args.length != 2) {
    stderr.writeln('usage: oicana_example.dart <template.zip> <output.pdf>');
    exitCode = 64;
    return;
  }

  final template = await Template.load(File(args[0]).readAsBytesSync());
  try {
    for (final warning in template.warnings) {
      stderr.writeln(warning);
    }
    final pdf = await template.exportPdf(mode: CompilationMode.development);
    File(args[1]).writeAsBytesSync(pdf);
  } on OicanaException catch (error) {
    stderr.writeln(error.message);
    exitCode = 1;
  } finally {
    template.dispose();
  }
}
