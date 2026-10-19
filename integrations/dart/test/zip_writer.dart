import 'dart:convert';
import 'dart:typed_data';

/// Writes an uncompressed zip archive for test templates.
Uint8List storedZip(Map<String, String> files) {
  final archive = BytesBuilder();
  final central = BytesBuilder();
  for (final MapEntry(:key, :value) in files.entries) {
    final name = utf8.encode(key);
    final content = utf8.encode(value);
    final crc = _crc32(content);
    final offset = archive.length;
    archive
      ..add(_le([(0x04034b50, 4), (10, 2), (0, 2), (0, 2), (0, 2), (0, 2)]))
      ..add(_le([(crc, 4), (content.length, 4), (content.length, 4)]))
      ..add(_le([(name.length, 2), (0, 2)]))
      ..add(name)
      ..add(content);
    central
      ..add(_le([(0x02014b50, 4), (20, 2), (10, 2), (0, 2), (0, 2), (0, 2)]))
      ..add(_le([(0, 2), (crc, 4), (content.length, 4), (content.length, 4)]))
      ..add(_le([(name.length, 2), (0, 2), (0, 2), (0, 2), (0, 2)]))
      ..add(_le([(0, 4), (offset, 4)]))
      ..add(name);
  }
  final centralBytes = central.takeBytes();
  final centralOffset = archive.length;
  archive
    ..add(centralBytes)
    ..add(_le([(0x06054b50, 4), (0, 2), (0, 2), (files.length, 2)]))
    ..add(_le([(files.length, 2), (centralBytes.length, 4)]))
    ..add(_le([(centralOffset, 4), (0, 2)]));
  return archive.takeBytes();
}

/// Little-endian fields of the given byte widths.
Uint8List _le(List<(int, int)> fields) {
  final bytes = BytesBuilder();
  for (final (value, width) in fields) {
    for (var i = 0; i < width; i++) {
      bytes.addByte((value >> (8 * i)) & 0xff);
    }
  }
  return bytes.takeBytes();
}

int _crc32(List<int> data) {
  var crc = 0xffffffff;
  for (final byte in data) {
    crc ^= byte;
    for (var i = 0; i < 8; i++) {
      crc = (crc & 1) != 0 ? (crc >> 1) ^ 0xedb88320 : crc >> 1;
    }
  }
  return crc ^ 0xffffffff;
}
