@TestOn('vm')
library;

import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:oicana/oicana.dart';
import 'package:test/test.dart';

import 'zip_writer.dart';

final root = Directory.current.uri.resolve('../../');
const testFamily = 'Oicana Test';

Uint8List asset(String path) =>
    File.fromUri(root.resolve('assets/$path')).readAsBytesSync();

final e2eTemplate = File.fromUri(
  root.resolve('e2e-tests/template/oicana-e2e-test-x.y.z.zip'),
).readAsBytesSync();

Uint8List minimalTemplate(
  String main, {
  List<String> requiredFonts = const [],
}) {
  var manifest =
      '[package]\nname = "test"\nversion = "0.1.0"\nentrypoint = "main.typ"\n\n'
      '[tool.oicana]\nmanifest_version = 1\n';
  if (requiredFonts.isNotEmpty) {
    manifest +=
        '\n[tool.oicana.fonts]\nrequire = ${jsonEncode(requiredFonts)}\n';
  }
  return storedZip({'typst.toml': manifest, 'main.typ': main});
}

void writeOutput(String name, Uint8List data) {
  final file = File('test/testOutput/$name');
  file.parent.createSync(recursive: true);
  file.writeAsBytesSync(data);
}

BlobInput blob(int foo) => BlobInput(
  asset('inputs/input.txt'),
  metadata: {
    'image_format': 'jpeg',
    'foo': foo,
    'bar': ['input', 'two'],
  },
);

bool isPdf(Uint8List data) => ascii.decode(data.sublist(0, 4)) == '%PDF';

bool isPng(Uint8List data) =>
    data.length > 4 &&
    data[0] == 0x89 &&
    ascii.decode(data.sublist(1, 4)) == 'PNG';

void main() {
  late Template template;

  setUp(() async => template = await Template.load(e2eTemplate));
  tearDown(() => template.dispose());

  test('development', () async {
    writeOutput(
      'development.png',
      await template.exportPng(mode: CompilationMode.development),
    );
  });

  test('production', () async {
    final image = await template.exportPng(
      jsonInputs: {'development-json': utf8.decode(asset('inputs/input.json'))},
      blobInputs: {'development-blob': blob(43)},
    );
    writeOutput('production.png', image);
  });

  test('all inputs', () async {
    final json = jsonDecode(utf8.decode(asset('inputs/input.json')));
    final image = await template.exportPng(
      jsonInputs: {
        'default-json': json,
        'development-json': json,
        'both-json': json,
      },
      blobInputs: {
        'default-blob': blob(42),
        'development-blob': blob(43),
        'both-blob': blob(44),
      },
    );
    writeOutput('all-inputs.png', image);
  });

  test('export formats', () async {
    const development = CompilationMode.development;
    expect(isPdf(await template.exportPdf(mode: development)), isTrue);
    expect(
      isPng(await template.exportPng(mode: development, pixelsPerPt: 2)),
      isTrue,
    );
    expect(
      utf8.decode(await template.exportSvg(mode: development)),
      contains('<svg'),
    );
    expect(
      isPng(template.exportSync(ExportFormat.png(), mode: development)),
      isTrue,
    );
    expect(isPdf(template.exportPdfSync(mode: development)), isTrue);
  });

  test('exports default to production mode', () async {
    await expectLater(
      template.exportPdf(),
      throwsA(
        isA<CompilationException>().having(
          (e) => e.message,
          'message',
          contains('No value for the required input'),
        ),
      ),
    );
  });

  test('production warm-up needs inputs', () async {
    await expectLater(
      Template.load(e2eTemplate, mode: CompilationMode.production),
      throwsA(isA<CompilationException>()),
    );
  });

  test('manifest', () {
    final manifest = template.manifest();
    expect(manifest.package.name, 'oicana-e2e-test');
    expect(manifest.package.version, '0.1.0');
    expect(manifest.oicana.manifestVersion, 1);
    expect(manifest.oicana.validateJsonInputsByDefault, isTrue);
    expect(manifest.oicana.export.pdf.standards, isEmpty);
    expect(manifest.oicana.fonts.require, isEmpty);

    final inputs = {
      for (final input in manifest.oicana.inputs) input.key: input,
    };
    final jsonInput = inputs['development-json'] as JsonInputDefinition;
    expect(jsonInput.schema, 'input.schema.json');
    expect(jsonInput.development, 'development.json');
    expect(jsonInput.defaultValue, isNull);
    expect(jsonInput.validate, isTrue);

    final blobInput = inputs['default-blob'] as BlobInputDefinition;
    expect(blobInput.defaultValue!.file, 'default.txt');
    expect(blobInput.defaultValue!.meta!['image_format'], 'png');
    expect(blobInput.development, isNull);
  });

  test('template files', () {
    expect(template.source('main.typ'), isNotEmpty);
    expect(template.file('default.txt'), isNotEmpty);
    expect(
      () => template.file('missing.txt'),
      throwsA(isA<FileNotFoundException>()),
    );
  });

  test('validate inputs', () async {
    final inputs = {
      'development-json': {'unexpected': true},
    };
    await expectLater(
      template.exportPdf(jsonInputs: inputs, mode: CompilationMode.development),
      throwsA(isA<InvalidInputException>()),
    );

    template.setValidateInputs(false);
    try {
      await template.exportPdf(
        jsonInputs: inputs,
        mode: CompilationMode.development,
      );
    } on CompilationException {
      // The template may still fail on the unexpected value, but not on its schema.
    }
  });

  test('values that cannot be encoded as JSON', () {
    expect(
      () => template.exportPdfSync(jsonInputs: {'development-json': Object()}),
      throwsA(isA<InvalidInputException>()),
    );
  });

  test('compiled document outlives its template', () async {
    final document = await template.compile(mode: CompilationMode.development);
    template.dispose();

    expect(document.pages, isNotEmpty);
    expect(document.pageCount, document.pages.length);
    expect(document.pages.first.width, greaterThan(0));
    expect(
      isPdf(await document.exportPdf(pages: const PageRange.single(0))),
      isTrue,
    );
    expect(
      isPng(document.exportPngSync(pages: const PageRange(end: 0))),
      isTrue,
    );
    expect(
      utf8.decode(
        document.exportSvgSync(pages: const PageRange(start: 0, end: 0)),
      ),
      contains('<svg'),
    );
    expect(
      () => document.exportSvgSync(pages: PageRange(start: document.pageCount)),
      throwsA(isA<ExportException>()),
    );
    expect(
      () => document.exportPdfSync(pages: const PageRange.single(-1)),
      throwsA(isA<InvalidInputException>()),
    );
    document.dispose();
  });

  test('disposed', () async {
    final document = template.compileSync(mode: CompilationMode.development);
    template.dispose();
    template.dispose();
    document.dispose();

    expect(
      () => template.compileSync(mode: CompilationMode.development),
      throwsStateError,
    );
    expect(() => document.exportPdfSync(), throwsStateError);
  });

  test('invalid template', () {
    expect(
      () => Template.loadSync(Uint8List.fromList(utf8.encode('not a zip'))),
      throwsA(isA<InvalidTemplateException>()),
    );
  });

  test('zip limits', () {
    expect(
      () => Template.loadSync(
        e2eTemplate,
        limits: const ZipLimits(maxEntries: 1),
      ),
      throwsA(isA<InvalidTemplateException>()),
    );
  });

  test('export once', () async {
    final result = await exportOnce(
      e2eTemplate,
      mode: CompilationMode.development,
    );
    expect(isPdf(result.document), isTrue);

    final svg = exportOnceSync(
      minimalTemplate('#set text(font: "No Such Font")\nHello'),
      format: ExportFormat.svg,
      pages: const PageRange.single(0),
    );
    expect(svg.warnings, hasLength(1));
    final warning = svg.warnings.single;
    expect(warning.severity, DiagnosticSeverity.warning);
    expect(warning.message, 'unknown font family: no such font');
    expect(warning.location!.file, '/main.typ');
    expect(
      [
        warning.location!.line,
        warning.location!.column,
        warning.location!.endLine,
        warning.location!.endColumn,
      ],
      [1, 17, 1, 31],
    );
    expect(warning.hints, isEmpty);
    expect(warning.trace, isEmpty);
    expect(warning.toString(), contains('No Such Font'));

    await expectLater(
      exportOnce(minimalTemplate('#panic("broken")')),
      throwsA(isA<CompilationException>()),
    );
  });

  test('template warnings', () async {
    final warned = await Template.load(
      minimalTemplate('#set text(font: "No Such Font")\nHello'),
    );
    expect(warned.warnings, hasLength(1));
    final document = await warned.compile();
    expect(document.warnings.single.message, warned.warnings.single.message);
    document.dispose();
    warned.dispose();

    final clean = Template.loadSync(minimalTemplate('Hello'));
    expect(clean.warnings, isEmpty);
    clean.dispose();
  });

  test('fonts', () {
    addTearDown(clearFonts);
    clearFonts();
    final fontTemplate = minimalTemplate(
      '#set text(font: "$testFamily")\nHello',
      requiredFonts: [testFamily],
    );
    expect(
      () => Template.loadSync(fontTemplate),
      throwsA(isA<InvalidTemplateException>()),
    );

    expect(
      registerFonts([
        asset('fonts/oicana-test-font.ttf'),
        Uint8List.fromList(utf8.encode('no font')),
      ]),
      1,
    );
    expect(registeredFonts().single.family, testFamily);
    expect(registeredFonts().single.path, isNull);
    Template.loadSync(fontTemplate).dispose();

    clearFonts();
    expect(registerFontPaths([root.resolve('assets/fonts').toFilePath()]), 1);
    expect(registeredFonts().single.path, endsWith('oicana-test-font.ttf'));
  });

  test('diagnostic color', () {
    addTearDown(() => configureDiagnosticColor(DiagnosticColor.none));
    final broken = minimalTemplate('#panic("broken")');

    configureDiagnosticColor(DiagnosticColor.ansi);
    expect(
      () => exportOnceSync(broken),
      throwsA(
        isA<CompilationException>().having(
          (e) => e.message,
          'message',
          contains('\x1b['),
        ),
      ),
    );

    configureDiagnosticColor(DiagnosticColor.none);
    expect(
      () => exportOnceSync(broken),
      throwsA(
        isA<CompilationException>().having(
          (e) => e.message,
          'message',
          isNot(contains('\x1b[')),
        ),
      ),
    );
  });

  test('cache eviction', () {
    addTearDown(() => configureAutomaticCacheEviction(10));
    configureAutomaticCacheEviction(null);
    evictCache(0);
    configureAutomaticCacheEviction(0);
    expect(
      isPdf(template.exportPdfSync(mode: CompilationMode.development)),
      isTrue,
    );
  });

  test('version', () {
    final pubspec = File('pubspec.yaml').readAsStringSync();
    final version = RegExp(r'^version: (\S+)', multiLine: true);
    expect(version.firstMatch(pubspec)!.group(1), oicanaVersion());
  });

  test('concurrent exports', () async {
    final exports = await Future.wait([
      for (var i = 0; i < 8; i++)
        template.exportPdf(mode: CompilationMode.development),
    ]);
    expect(exports.every(isPdf), isTrue);
  });

  test('async calls do not block the calling isolate', () async {
    final document = template.compileSync(mode: CompilationMode.development);
    addTearDown(document.dispose);
    var ticks = 0;
    var exporting = true;
    // A single native call, made slow through a high resolution.
    final export = document.exportPng(pixelsPerPt: 8).whenComplete(() {
      exporting = false;
    });
    while (exporting) {
      await Future<void>.delayed(const Duration(milliseconds: 1));
      ticks++;
    }
    await export;
    expect(ticks, greaterThan(1));
  });
}
