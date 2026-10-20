import 'dart:typed_data';

import 'backend.dart';
import 'platform.dart';
import 'types.dart';

/// Loads Oicana.
///
/// Required on the web before calling any synchronous function, and a no-op on
/// native targets. Flutter web apps load the WebAssembly build from the assets
/// of this package. Dart web apps without Flutter serve
/// `oicana_browser_wasm.js` and `oicana_browser_wasm_bg.wasm` from the `web`
/// directory of this package themselves and have to pass the URL of the
/// JavaScript file as [wasmModule].
Future<void> initOicana({Uri? wasmModule}) => backend.initialize(wasmModule);

/// Compiles a packed template and exports it, without caching anything.
///
/// Skips the warm-up compilation of [Template.load], so it is the fastest way
/// to render a template exactly once. A [String] value in [jsonInputs] is
/// taken as a JSON document; other values are encoded as JSON.
Future<ExportOnceResult> exportOnce(
  Uint8List template, {
  Map<String, Object?> jsonInputs = const {},
  Map<String, BlobInput> blobInputs = const {},
  CompilationMode mode = CompilationMode.production,
  ExportFormat format = ExportFormat.pdf,
  PageRange? pages,
  ZipLimits? limits,
}) {
  final inputs = EncodedInputs(jsonInputs, blobInputs);
  return backend.exportOnceAsync(
    template,
    inputs,
    mode,
    format,
    checkPages(pages),
    limits,
  );
}

/// Like [exportOnce], but blocks until the document is exported.
ExportOnceResult exportOnceSync(
  Uint8List template, {
  Map<String, Object?> jsonInputs = const {},
  Map<String, BlobInput> blobInputs = const {},
  CompilationMode mode = CompilationMode.production,
  ExportFormat format = ExportFormat.pdf,
  PageRange? pages,
  ZipLimits? limits,
}) {
  final inputs = EncodedInputs(jsonInputs, blobInputs);
  return backend.exportOnce(
    template,
    inputs,
    mode,
    format,
    checkPages(pages),
    limits,
  );
}

/// Makes fonts available to every template loaded afterwards. Returns the
/// number of font faces added. Data that holds no font is ignored.
int registerFonts(List<Uint8List> fonts) => backend.registerFonts(fonts);

/// Makes font files, or every font file in the given directories, available to
/// every template loaded afterwards. Returns the number of font faces added.
///
/// Not available on the web, use [registerFonts] there.
int registerFontPaths(List<String> paths) => backend.registerFontPaths(paths);

/// The font faces registered with [registerFonts] or [registerFontPaths].
List<RegisteredFont> registeredFonts() => backend.registeredFonts();

/// Drops all registered fonts. Existing templates keep the fonts they were
/// loaded with.
void clearFonts() => backend.clearFonts();

/// Sets the coloring of compilation diagnostics for all templates.
void configureDiagnosticColor(DiagnosticColor color) =>
    backend.configureDiagnosticColor(color);

/// Evicts cache entries after each compilation that were not used within the
/// last [maxAge] evictions. 0 clears the whole cache every time, `null` stops
/// evicting automatically. The default is 10.
void configureAutomaticCacheEviction(int? maxAge) =>
    backend.configureAutomaticCacheEviction(maxAge);

/// Evicts cache entries that were not used within the last [maxAge] evictions.
void evictCache(int maxAge) => backend.evictCache(maxAge);

/// The version of Oicana.
String oicanaVersion() => backend.version();
