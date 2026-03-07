# Oicana PHP Integration

PDF templating with Typst for PHP.

## Installation

First, add the Oicana Composer repository to your `composer.json`:

```json
{
    "repositories": [
        {
            "type": "composer",
            "url": "https://composer.oicana.com"
        }
    ]
}
```

Then install via Composer:

```bash
composer require oicana/oicana
```

The installer will automatically download the appropriate native extension for your platform. To enable the extension, run:

```bash
vendor/bin/oicana-env
```

This outputs the `PHP_INI_SCAN_DIR` export command for your platform. Add it to your shell profile to make it permanent.

## Requirements

- PHP 8.3, 8.4, or 8.5
- One of: Linux (x64/ARM64), macOS (x64/ARM64), or Windows (x64)

## Quick Start

```php
<?php

require 'vendor/autoload.php';

use Oicana\Template;

// One-off compilation
$pdf = Template::compileOnce(
    file_get_contents('path/to/template.zip'),
    jsonInputs: [
        'data' => ['foo' => 'bar'],
    ]
);
file_put_contents('output.pdf', $pdf);
```

For multiple compilations with the same template, reuse a `Template` instance:

```php
use Oicana\ExportFormat;
use Oicana\Template;

$template = new Template(file_get_contents('path/to/template.zip'));

try {
    $pdf = $template->compile(
        jsonInputs: [
            'title' => ['value' => 'My Document'],
            'date' => ['value' => '2025-01-01'],
        ],
        exportFormat: ExportFormat::pdf()
    );
    file_put_contents('output.pdf', $pdf);
} finally {
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

### One-off Compilation

For one-off compilations where you don't need to reuse the template, use `compileOnce()`. It handles template registration and cleanup automatically:

```php
$pdf = Template::compileOnce(
    file_get_contents('template.zip'),
    jsonInputs: ['inputkey' => ['name' => 'Alice']],
    exportFormat: ExportFormat::pdf()
);
```

### Compilation Modes

The mode can be set separately for template creation (`new Template`) and compilation (`compile`).

**Development Mode** — uses default values from template when inputs are missing. Good for registration.

**Production Mode** — requires all inputs to be explicitly provided. Ensures no missing data in final output.

By default, `new Template()` uses Development mode (so the template can be registered without providing all inputs), while `compile()` uses Production mode by default (so you catch missing inputs at render time). This is the recommended pattern for most use cases:

```php
use Oicana\CompilationMode;

// Development for creation, Production for compilation (defaults)
$template = new Template($bytes);
$pdf = $template->compile(jsonInputs: ['name' => ['value' => 'Alice']]);

// Override if needed
$template = new Template($bytes, mode: CompilationMode::Production);
$pdf = $template->compile(jsonInputs: $data, mode: CompilationMode::Development);
```

### Working with Inputs

**JSON Inputs:**

You can pass inputs as arrays (recommended) or as pre-encoded JSON strings:

```php
// Arrays are automatically JSON-encoded
$pdf = $template->compile(
    jsonInputs: [
        'user' => ['name' => 'Alice', 'email' => 'alice@example.com'],
        'items' => [['id' => 1, 'name' => 'Item 1']],
    ]
);

// Pre-encoded JSON strings also work
$pdf = $template->compile(
    jsonInputs: [
        'user' => '{"name": "Alice", "email": "alice@example.com"}',
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
