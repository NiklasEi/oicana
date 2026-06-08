<?php

declare(strict_types=1);

use Oicana\CompilationMode;
use Oicana\ExportFormat;
use Oicana\Inputs\BlobInput;
use Oicana\Template;

beforeEach(function () {
    if (!extension_loaded('oicana')) {
        test()->markTestSkipped('oicana extension not loaded');
    }

    if (!file_exists(e2e_template_path())) {
        test()->markTestSkipped('E2E template not found. Run `oicana pack` in e2e-tests/template first.');
    }
});

test('template can be instantiated', function () {
    $templateBytes = file_get_contents(e2e_template_path());
    $template = new Template($templateBytes);

    expect($template)->toBeInstanceOf(Template::class);

    $template->cleanup();
});

test('template compiles to PDF in development mode', function () {
    $templateBytes = file_get_contents(e2e_template_path());
    $template = new Template($templateBytes, mode: CompilationMode::Development);

    try {
        $pdf = $template->export(mode: CompilationMode::Development);

        expect($pdf)
            ->not->toBeEmpty()
            ->and($pdf)->toStartWith('%PDF');
    } finally {
        $template->cleanup();
    }
});

test('template compiles with JSON inputs', function () {
    $templateBytes = file_get_contents(e2e_template_path());
    $template = new Template($templateBytes);

    $jsonContent = file_get_contents(assets_path('inputs/input.json'));
    assert(is_string($jsonContent));

    try {
        $pdf = $template->export(
            jsonInputs: ['development-json' => $jsonContent],
            mode: CompilationMode::Development
        );

        expect($pdf)->not->toBeEmpty();
    } finally {
        $template->cleanup();
    }
});

test('template compiles with blob inputs', function () {
    $templateBytes = file_get_contents(e2e_template_path());
    $template = new Template($templateBytes);

    $blobData = file_get_contents(assets_path('inputs/input.txt'));
    $blobInput = new BlobInput($blobData, [
        'image_format' => 'jpeg',
        'foo' => 43,
        'bar' => ['input', 'two']
    ]);

    try {
        $pdf = $template->export(
            blobInputs: ['development-blob' => $blobInput],
            mode: CompilationMode::Development
        );

        expect($pdf)->not->toBeEmpty();
    } finally {
        $template->cleanup();
    }
});

test('template exports to SVG', function () {
    $templateBytes = file_get_contents(e2e_template_path());
    $template = new Template($templateBytes, mode: CompilationMode::Development);

    try {
        $svg = $template->export(
            exportFormat: ExportFormat::svg(),
            mode: CompilationMode::Development
        );

        expect($svg)
            ->not->toBeEmpty()
            ->and($svg)->toContain('<svg');
    } finally {
        $template->cleanup();
    }
});

test('template exports to PNG', function () {
    $templateBytes = file_get_contents(e2e_template_path());
    $template = new Template($templateBytes, mode: CompilationMode::Development);

    try {
        $png = $template->export(
            exportFormat: ExportFormat::png(pixelsPerPt: 2.0),
            mode: CompilationMode::Development
        );

        expect($png)
            ->not->toBeEmpty()
            ->and($png)->toStartWith("\x89PNG");
    } finally {
        $template->cleanup();
    }
});

test('template provides input definitions', function () {
    $templateBytes = file_get_contents(e2e_template_path());
    $template = new Template($templateBytes);

    try {
        $inputs = $template->inputs();

        expect($inputs)->toBeArray();
        expect($inputs)->toHaveKey('inputs');
    } finally {
        $template->cleanup();
    }
});

test('compilation modes work correctly', function () {
    $templateBytes = file_get_contents(e2e_template_path());

    // Development mode - should work without explicit inputs
    $devTemplate = new Template($templateBytes, mode: CompilationMode::Development);
    try {
        $devPdf = $devTemplate->export(mode: CompilationMode::Development);
        expect($devPdf)->not->toBeEmpty();
    } finally {
        $devTemplate->cleanup();
    }
});

test('production mode requires all inputs', function () {
    $templateBytes = file_get_contents(e2e_template_path());
    $template = new Template($templateBytes, mode: CompilationMode::Development);

    try {
        // Production mode without required inputs should fail
        expect(fn() => $template->export(mode: CompilationMode::Production))
            ->toThrow(Exception::class);
    } finally {
        $template->cleanup();
    }
});

test('compile defaults to production mode', function () {
    $templateBytes = file_get_contents(e2e_template_path());
    $template = new Template($templateBytes, mode: CompilationMode::Development);

    try {
        // Default export() should use production mode and fail without inputs
        expect(fn() => $template->export())
            ->toThrow(Exception::class);
    } finally {
        $template->cleanup();
    }
});
