# Oicana PHP Installer

Composer plugin that automatically downloads and installs the Oicana native extension for PHP.

## What it does

This plugin automatically:

1. Detects your platform (OS, architecture, PHP version, thread-safety)
2. Downloads the appropriate native extension binary from GitHub Releases
3. Installs it to the correct location
4. Provides instructions for enabling the extension

## Requirements

- PHP 8.3, 8.4, 8.5
- Composer 2.0+
- One of: Linux (x64/ARM64), macOS (x64/ARM64), or Windows (x64)

## Installation

This package is automatically installed as a dependency of `oicana/oicana`:

```bash
composer require oicana/oicana
```

## Manual Installation

If automatic installation fails, you can:

### Option 1: Download Manually

1. Go to https://github.com/oicana/oicana/releases
2. Download the binary for your platform
3. Place it in your PHP extensions directory
4. Add `extension=oicana_native` to your php.ini

### Option 2: Build from Source

See the [oicana-php-native](../oicana-php-native) package for build instructions.

## Supported Platforms

| OS | Architecture | PHP 8.3 | PHP 8.4 |
|----|-------------|---------|---------|
| Linux | x64 | ✓ | ✓ |
| Linux | ARM64 | ✓ | ✓ |
| macOS | x64 | ✓ | ✓ |
| macOS | ARM64 | ✓ | ✓ |
| Windows | x64 | ✓ | ✓ |

Both NTS (non-thread-safe) and ZTS (thread-safe) builds are provided.

## License

This project is licensed under the PolyForm Noncommercial License 1.0.0. See [LICENSE.md](LICENSE.md) for details.

For commercial use, please contact hello@oicana.com.
