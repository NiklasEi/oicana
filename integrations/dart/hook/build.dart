import 'dart:io';
import 'dart:typed_data';

import 'package:code_assets/code_assets.dart';
import 'package:crypto/crypto.dart';
import 'package:hooks/hooks.dart';
import 'package:oicana/src/version.dart';

void main(List<String> args) async {
  await build(args, (input, output) async {
    // Web builds use the WebAssembly assets declared in pubspec.yaml.
    if (!input.config.buildCodeAssets) return;

    final code = input.config.code;
    final target = _rustTarget(code);
    final fileName = code.targetOS.dylibFileName('oicana_uniffi');
    final File library;
    final localDirectory = input.userDefines.path('native_library_dir');
    if (localDirectory != null) {
      final isHost =
          code.targetOS == OS.current &&
          code.targetArchitecture == Architecture.current;
      library = _localLibrary(localDirectory, target, fileName, isHost);
      output.dependencies.add(library.uri);
    } else {
      library = File.fromUri(
        input.outputDirectoryShared.resolve(
          '$packageVersion/$target/$fileName',
        ),
      );
      if (!library.existsSync()) {
        final checksums = await _checksums(input.packageRoot, packageVersion);
        await _download(packageVersion, target, fileName, checksums, library);
      }
    }

    output.assets.code.add(
      CodeAsset(
        package: input.packageName,
        name: 'uniffi:oicana_uniffi',
        linkMode: DynamicLoadingBundled(),
        file: library.uri,
      ),
    );
  });
}

File _localLibrary(Uri path, String target, String fileName, bool isHost) {
  final directory = path.path.endsWith('/')
      ? path
      : path.replace(path: '${path.path}/');
  final candidates = [
    directory.resolve('$target/$fileName'),
    if (isHost) directory.resolve(fileName),
  ];
  for (final candidate in candidates) {
    final file = File.fromUri(candidate);
    if (file.existsSync()) return file;
  }
  throw StateError(
    'native_library_dir is set, but there is no library at '
    '${candidates.map((candidate) => candidate.toFilePath()).join(' or ')}',
  );
}

Future<String> _checksums(Uri packageRoot, String version) async {
  final bundled = File.fromUri(packageRoot.resolve('hook/SHA256SUMS'));
  if (bundled.existsSync()) return bundled.readAsString();
  return String.fromCharCodes(await _get(_releaseUrl(version, 'SHA256SUMS')));
}

Future<void> _download(
  String version,
  String target,
  String fileName,
  String checksums,
  File library,
) async {
  final asset = 'oicana_uniffi-$version-$target.${fileName.split('.').last}.gz';
  final expected = RegExp(
    '^([0-9a-f]{64}) +\\*?${RegExp.escape(asset)}\$',
    multiLine: true,
  ).firstMatch(checksums)?.group(1);
  if (expected == null) {
    throw UnsupportedError('Oicana $version has no native library for $target');
  }
  final compressed = await _get(_releaseUrl(version, asset));
  final actual = sha256.convert(compressed).toString();
  if (actual != expected) {
    throw StateError(
      'Checksum mismatch for $asset: expected $expected, got $actual',
    );
  }
  await library.parent.create(recursive: true);
  final partial = File('${library.path}.partial');
  await partial.writeAsBytes(gzip.decode(compressed));
  await partial.rename(library.path);
}

Uri _releaseUrl(String version, String asset) => Uri.parse(
  'https://github.com/oicana/oicana/releases/download/oicana_uniffi-v$version/$asset',
);

Future<List<int>> _get(Uri url) async {
  final client = HttpClient()
    ..connectionTimeout = const Duration(seconds: 30)
    ..findProxy = HttpClient.findProxyFromEnvironment;
  try {
    final response = await (await client.getUrl(url)).close();
    if (response.statusCode != HttpStatus.ok) {
      throw HttpException(
        'Downloading failed with status ${response.statusCode}',
        uri: url,
      );
    }
    final builder = BytesBuilder(copy: false);
    await response.forEach(builder.add);
    return builder.takeBytes();
  } finally {
    client.close();
  }
}

String _rustTarget(CodeConfig code) {
  final architecture = code.targetArchitecture;
  final target = switch (code.targetOS) {
    OS.android => switch (architecture) {
      Architecture.arm64 => 'aarch64-linux-android',
      Architecture.arm => 'armv7-linux-androideabi',
      Architecture.x64 => 'x86_64-linux-android',
      _ => null,
    },
    OS.iOS => switch (architecture) {
      Architecture.arm64 when code.iOS.targetSdk == IOSSdk.iPhoneSimulator =>
        'aarch64-apple-ios-sim',
      Architecture.arm64 => 'aarch64-apple-ios',
      Architecture.x64 => 'x86_64-apple-ios',
      _ => null,
    },
    OS.macOS => switch (architecture) {
      Architecture.arm64 => 'aarch64-apple-darwin',
      Architecture.x64 => 'x86_64-apple-darwin',
      _ => null,
    },
    OS.linux => switch (architecture) {
      Architecture.arm64 => 'aarch64-unknown-linux-gnu',
      Architecture.x64 => 'x86_64-unknown-linux-gnu',
      _ => null,
    },
    OS.windows => switch (architecture) {
      Architecture.arm64 => 'aarch64-pc-windows-msvc',
      Architecture.x64 => 'x86_64-pc-windows-msvc',
      _ => null,
    },
    _ => null,
  };
  return target ??
      (throw UnsupportedError(
        'Oicana does not support ${code.targetOS} on $architecture',
      ));
}
