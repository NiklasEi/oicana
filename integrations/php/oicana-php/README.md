# Oicana PHP Integration

PDF templating with Typst for PHP.

## Installation

Install via Composer:

```bash
composer require oicana/oicana
```

The installer will automatically download the appropriate native extension for your platform. Follow installer output instructions to enable the native extension.

## Requirements

- PHP 8.3, 8.4, or 8.5
- One of: Linux (x64/ARM64), macOS (x64/ARM64), or Windows (x64)

## Quick Start

```php
<?php

require 'vendor/autoload.php';

use Oicana\CompilationMode;
use Oicana\ExportFormat;
use Oicana\Template;

// Load your template
$templateBytes = file_get_contents('path/to/template.zip');
$template = new Template($templateBytes);

try {
    // Compile to PDF
    $pdf = $template->compile(
        jsonInputs: [
            'title' => '{"value": "My Document"}',
            'date' => '{"value": "2025-01-01"}'
        ],
        exportFormat: ExportFormat::pdf(),
        mode: CompilationMode::Production
    );

    // Save the PDF
    file_put_contents('output.pdf', $pdf);
} finally {
    // Always cleanup
    $template->cleanup();
}
```

## Usage

### Creating Templates

Templates are created using [Typst](https://typst.app/) and packaged with the `oicana` CLI tool. See the [Oicana documentation][oicana-docs] for more info.

### Export Formats

Oicana supports three export formats via the `ExportFormat` class:

```php
use Oicana\ExportFormat;

$pdf = $template->compile(exportFormat: ExportFormat::pdf());
$png = $template->compile(exportFormat: ExportFormat::png(pixelsPerPt: 3.0));
$svg = $template->compile(exportFormat: ExportFormat::svg());
```

### Compilation Modes

**Development Mode:**
- Uses default values from template when inputs are missing
- Good for testing and preview

```php
$template = new Template($bytes, mode: CompilationMode::Development);
```

**Production Mode (recommended):**
- Requires all inputs to be explicitly provided
- Ensures no missing data in final output

```php
$template = new Template($bytes, mode: CompilationMode::Production);
```

### Working with Inputs

**JSON Inputs:**
```php
$pdf = $template->compile(
    jsonInputs: [
        'user' => '{"name": "Alice", "email": "alice@example.com"}',
        'items' => '[{"id": 1, "name": "Item 1"}]'
    ]
);
```

**Blob Inputs (images, fonts, etc.):**
```php
use Oicana\Inputs\BlobInput;

$logoData = file_get_contents('logo.png');
$logo = new BlobInput($logoData, ['type' => 'image/png']);

$pdf = $template->compile(
    blobInputs: ['logo' => $logo]
);
```

### Template Introspection

**Get input definitions:**
```php
$inputs = $template->inputs();
// Returns array of input definitions from template manifest
```

**Read template files:**
```php
$source = $template->source('main.typ');  // Get Typst source code
$file = $template->file('assets/logo.png');  // Get binary file
```


[oicana-docs]: https://docs.oicana.com/
