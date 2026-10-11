# Oicana for Go

*Generate PDFs in Go without a headless browser.*

In Go, PDF generation usually means headless Chrome, driven through chromedp or running in a Gotenberg service, or drawing every table and page break yourself with a low-level library. The first adds a browser to your deployment, the second buries the document design in code.

Oicana compiles PDFs in process through a statically linked native library instead. You design documents as [Typst](https://typst.app/) templates, load them once at startup, and render them from JSON in single-digit milliseconds. No browser process, no per-document fees, no document data leaving your infrastructure.

> **Free for noncommercial use.** Commercial use is free for 30 days, then needs a [per-application subscription](https://oicana.com/#pricing) with unlimited seats.

## Installation

```bash
go get go.oicana.com/oicana
```

The package links the native Oicana library through cgo, so it needs a C toolchain (MinGW-w64 on Windows). The installer downloads the library for your platform, verifies its checksum, caches it, and prints the `CGO_LDFLAGS` that link it:

```bash
export CGO_LDFLAGS="$(go run go.oicana.com/oicana/cmd/oicana-install)"
go build ./...
```

In PowerShell, use `$env:CGO_LDFLAGS = go run go.oicana.com/oicana/cmd/oicana-install`. Run it inside your module, so that it installs the library matching the version in your `go.mod`.

Supported platforms are Linux (x64 and arm64, glibc 2.28 or newer, or musl), macOS (x64 and arm64), and Windows x64. On musl, the built program needs `libgcc_s` at runtime (`apk add libgcc` on Alpine).

## Quick start

```go
package main

import (
	"log"
	"os"

	oicana "go.oicana.com/oicana"
)

func main() {
	files, err := os.ReadFile("invoice-0.1.0.zip")
	if err != nil {
		log.Fatal(err)
	}
	template, err := oicana.NewTemplate(files)
	if err != nil {
		log.Fatal(err)
	}
	defer template.Close()

	pdf, err := template.ExportPdf(oicana.Inputs{
		JSON: map[string]any{
			"invoice": map[string]any{"number": "2026-001", "customer": "Acme GmbH", "total": "€1,190.00"},
		},
	})
	if err != nil {
		log.Fatal(err)
	}
	if err := os.WriteFile("invoice.pdf", pdf, 0o644); err != nil {
		log.Fatal(err)
	}
}
```

Create templates at startup, they are safe for concurrent use. `ExportPng` and `ExportSvg` produce the other formats, `Compile` keeps a document to export its pages separately, and `ExportOnce` renders a template once without caching it. Errors match the `Err*` values of the package with `errors.Is`.

## Licensing

Oicana is source-available under the [PolyForm Noncommercial License 1.0.0](https://github.com/oicana/oicana/blob/main/LICENSE.md) and free for noncommercial use. Commercial use is free for 30 days; see [pricing](https://oicana.com/#pricing) for subscriptions, or write to `hello@oicana.com`.
