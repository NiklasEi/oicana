<?php

declare(strict_types=1);

use Oicana\CompilationMode;
use Oicana\Configuration;
use Oicana\ExportFormat;
use Oicana\RegisteredFont;
use Oicana\Template;

beforeEach(function () {
    if (!extension_loaded('oicana')) {
        test()->markTestSkipped('oicana extension not loaded');
    }

    // The font registry is process-global, so isolate every test.
    Configuration::clearFonts();
});

afterEach(function () {
    Configuration::clearFonts();
});

/**
 * Family the test font provides. No system or Typst-embedded font has it, so a
 * template requiring it can only be registered once the host registers the font.
 */
const TEST_FAMILY = 'Oicana Test';

/** The test font shipped with the repository. */
function a_font_file(): string
{
    return dirname(__DIR__, 4) . '/assets/fonts/oicana-test-font.ttf';
}

/** Pack a template whose manifest requires the given font family. */
function pack_template_requiring(string $family): string
{
    $manifest = <<<TOML
        [package]
        name = "font-test"
        version = "0.1.0"
        entrypoint = "main.typ"

        [tool.oicana]
        manifest_version = 1

        [tool.oicana.fonts]
        require = ["{$family}"]
        TOML;

    $path = tempnam(sys_get_temp_dir(), 'oicana-font-test-') . '.zip';
    $zip = new ZipArchive();
    $zip->open($path, ZipArchive::CREATE | ZipArchive::OVERWRITE);
    $zip->addFromString('typst.toml', $manifest);
    $zip->addFromString('main.typ', 'Content');
    $zip->setCompressionName('typst.toml', ZipArchive::CM_STORE);
    $zip->setCompressionName('main.typ', ZipArchive::CM_STORE);
    $zip->close();

    $bytes = file_get_contents($path);
    assert(is_string($bytes));
    unlink($path);

    return $bytes;
}

test('registry starts empty', function () {
    expect(Configuration::registeredFonts())->toBe([]);
});

test('fonts can be registered from bytes without a path', function () {
    $data = file_get_contents(a_font_file());
    assert(is_string($data));

    expect(Configuration::registerFonts([$data]))->toBe(1);

    $fonts = Configuration::registeredFonts();
    expect($fonts)->toHaveCount(1);
    expect($fonts[0])->toBeInstanceOf(RegisteredFont::class);
    expect($fonts[0]->family)->toBe(TEST_FAMILY);
    // Registered from memory, so no path is reported.
    expect($fonts[0]->path)->toBeNull();
});

test('data without a font is ignored', function () {
    expect(Configuration::registerFonts(['not a font']))->toBe(0);
    expect(Configuration::registeredFonts())->toBe([]);
});

test('fonts registered by path report the path', function () {
    $path = a_font_file();

    expect(Configuration::registerFontPaths([$path]))->toBe(1);

    $fonts = Configuration::registeredFonts();
    expect($fonts)->toHaveCount(1);
    expect($fonts[0]->family)->toBe(TEST_FAMILY);
    expect($fonts[0]->path)->toBe($path);
});

test('unreadable paths are skipped', function () {
    expect(Configuration::registerFontPaths(['/nonexistent/font.ttf']))->toBe(0);
    expect(Configuration::registeredFonts())->toBe([]);
});

test('clearFonts empties the registry', function () {
    Configuration::registerFontPaths([a_font_file()]);
    expect(Configuration::registeredFonts())->not->toBe([]);

    Configuration::clearFonts();

    expect(Configuration::registeredFonts())->toBe([]);
});

test('a template requiring an unavailable family is rejected', function () {
    $template = pack_template_requiring('Nonexistent Host Family');

    expect(fn () => new Template($template))
        ->toThrow(Exception::class, 'Nonexistent Host Family');
});

test('the test template is rejected until the font is registered', function () {
    // Proves the family really is unavailable without the host font.
    expect(fn () => new Template(pack_template_requiring(TEST_FAMILY)))
        ->toThrow(Exception::class, TEST_FAMILY);
});

test('a template requiring a registered family compiles', function () {
    Configuration::registerFontPaths([a_font_file()]);

    $template = new Template(pack_template_requiring(TEST_FAMILY), mode: CompilationMode::Development);

    try {
        $svg = $template->export(exportFormat: ExportFormat::svg(), mode: CompilationMode::Development);
        expect($svg)->toContain('<svg');
    } finally {
        $template->cleanup();
    }
});
