import { readFile } from 'node:fs/promises';
import * as zlib from 'node:zlib';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { CompilationMode } from './CompilationMode';
import { Svg } from './ExportFormat';
import {
  clearFonts,
  registeredFonts,
  registerFontPaths,
  registerFonts,
} from './index';
import { Template } from './Template';

const manifest = (requiredFamily?: string) => {
  const lines = [
    '[package]',
    'name = "font-test"',
    'version = "0.1.0"',
    'entrypoint = "main.typ"',
    '',
    '[tool.oicana]',
    'manifest_version = 1',
    '',
  ];
  if (requiredFamily) {
    lines.push('[tool.oicana.fonts]', `require = ["${requiredFamily}"]`, '');
  }
  return lines.join('\n');
};

/** Build a stored (uncompressed) zip with the given entries. */
function packTemplate(entries: Record<string, string>): Uint8Array {
  const localParts: Buffer[] = [];
  const centralParts: Buffer[] = [];
  let offset = 0;

  for (const [name, content] of Object.entries(entries)) {
    const nameBuffer = Buffer.from(name, 'utf-8');
    const data = Buffer.from(content, 'utf-8');
    const crc = zlib.crc32(data);

    const localHeader = Buffer.alloc(30);
    localHeader.writeUInt32LE(0x04034b50, 0);
    localHeader.writeUInt16LE(20, 4);
    localHeader.writeUInt32LE(crc, 14);
    localHeader.writeUInt32LE(data.length, 18);
    localHeader.writeUInt32LE(data.length, 22);
    localHeader.writeUInt16LE(nameBuffer.length, 26);
    localParts.push(localHeader, nameBuffer, data);

    const centralHeader = Buffer.alloc(46);
    centralHeader.writeUInt32LE(0x02014b50, 0);
    centralHeader.writeUInt16LE(20, 4);
    centralHeader.writeUInt16LE(20, 6);
    centralHeader.writeUInt32LE(crc, 16);
    centralHeader.writeUInt32LE(data.length, 20);
    centralHeader.writeUInt32LE(data.length, 24);
    centralHeader.writeUInt16LE(nameBuffer.length, 28);
    centralHeader.writeUInt32LE(offset, 42);
    centralParts.push(centralHeader, nameBuffer);

    offset += 30 + nameBuffer.length + data.length;
  }

  const centralSize = centralParts.reduce((sum, part) => sum + part.length, 0);
  const end = Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50, 0);
  end.writeUInt16LE(Object.keys(entries).length, 8);
  end.writeUInt16LE(Object.keys(entries).length, 10);
  end.writeUInt32LE(centralSize, 12);
  end.writeUInt32LE(offset, 16);

  return Buffer.concat([...localParts, ...centralParts, end]);
}

/** The test font shipped with the repository. */
const fontFile = '../../../assets/fonts/oicana-test-font.ttf';

/**
 * Family the test font provides. No system or Typst-embedded font has it, so a
 * template requiring it can only be registered once the host registers the font.
 */
const TEST_FAMILY = 'Oicana Test';

describe('host fonts', () => {
  // The font registry is process-global, so isolate every test.
  beforeEach(() => clearFonts());
  afterEach(() => clearFonts());

  it('starts empty', () => {
    expect(registeredFonts()).toEqual([]);
  });

  it('registers fonts from bytes without a path', async () => {
    const data = await readFile(fontFile);

    expect(registerFonts([data])).toBe(1);

    // Registered from memory, so no path is reported.
    expect(registeredFonts()).toEqual([{ family: TEST_FAMILY }]);
  });

  it('ignores data that holds no font', () => {
    expect(registerFonts([new Uint8Array([1, 2, 3, 4])])).toBe(0);
    expect(registeredFonts()).toEqual([]);
  });

  it('registers fonts by path and reports the path', () => {
    expect(registerFontPaths([fontFile])).toBe(1);

    expect(registeredFonts()).toEqual([
      { family: TEST_FAMILY, path: fontFile },
    ]);
  });

  it('skips paths that cannot be read', () => {
    expect(registerFontPaths(['/nonexistent/font.ttf'])).toBe(0);
    expect(registeredFonts()).toEqual([]);
  });

  it('clears the registry', () => {
    registerFontPaths([fontFile]);
    expect(registeredFonts().length).toBeGreaterThan(0);

    clearFonts();

    expect(registeredFonts()).toEqual([]);
  });

  it('rejects a template requiring a family no host font provides', () => {
    const template = packTemplate({
      'typst.toml': manifest('Nonexistent Host Family'),
      'main.typ': 'Content',
    });

    expect(() => new Template(template)).toThrow('Nonexistent Host Family');
  });

  it('rejects the test template until the font is registered', () => {
    const template = packTemplate({
      'typst.toml': manifest(TEST_FAMILY),
      'main.typ': 'Content',
    });

    // Proves the family really is unavailable without the host font.
    expect(() => new Template(template)).toThrow(TEST_FAMILY);
  });

  it('accepts a template whose required family was registered', () => {
    registerFontPaths([fontFile]);

    const template = new Template(
      packTemplate({
        'typst.toml': manifest(TEST_FAMILY),
        'main.typ': 'Content',
      }),
    );

    const svg = template.export(
      new Map(),
      new Map(),
      Svg,
      CompilationMode.Development,
    );
    expect(Buffer.from(svg).toString('utf-8')).toContain('<svg');
  });
});
