import 'dart:io';

import 'package:oicana/oicana.dart';

/// Renders a certificate for the given name to a PDF.
///
/// Uses the certificate template from
/// https://github.com/oicana/oicana-example-templates. Download the packed
/// `certificate-0.1.0.zip` from the releases of that repository, or pack it
/// yourself with `oicana pack templates/certificate`, then run:
///
/// `dart run example/oicana_example.dart certificate-0.1.0.zip "Ada Lovelace" certificate.pdf`
///
/// The getting started guide shows how to write your own Oicana template and
/// use it in a Flutter app: https://oicana.com/docs/getting-started/4-10-dart/
Future<void> main(List<String> args) async {
  if (args.length != 3) {
    stderr.writeln(
      'usage: oicana_example.dart <certificate-0.1.0.zip> <name> <output.pdf>',
    );
    exitCode = 64;
    return;
  }
  final [templatePath, name, outputPath] = args;

  // Load a template once and reuse it for every document.
  final template = await Template.load(File(templatePath).readAsBytesSync());
  try {
    final pdf = await template.exportPdf(
      jsonInputs: {
        'certificate': {'name': name},
      },
    );
    File(outputPath).writeAsBytesSync(pdf);
  } on OicanaException catch (error) {
    stderr.writeln(error.message);
    exitCode = 1;
  } finally {
    template.dispose();
  }
}
