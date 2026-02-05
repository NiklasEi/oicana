# Oicana PHP Native Extension

Native PHP extension for Oicana.

## Overview

This is the low-level native binding layer for the Oicana PHP integration. **Most users should use the [`oicana/oicana`](../oicana-php) package instead**, which provides a convenient PHP wrapper around this extension.

## Building

### Prerequisites

- Rust 1.88+
- PHP 8.3, 8.4, or 8.5 (with development headers)

### Build Instructions

```bash
cargo build --release
```

The extension will be compiled to `target/release/liboicana_native.so` (Linux), `liboicana_native.dylib` (macOS), or `oicana_native.dll` (Windows).

### Installing Locally

After building, copy the extension to your PHP extensions directory:

```bash
# Find your extension directory
php -i | grep extension_dir

# Copy the extension
cp target/release/liboicana_native.so /path/to/extensions/

# Or use cargo-php if available
cargo install cargo-php
cargo php install --release
```

Then add to your `php.ini`:

```ini
extension=oicana_native.so
```
