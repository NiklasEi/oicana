# Oicana for Ruby

*Generate PDFs in Ruby without a headless browser.*

Ruby apps generating PDFs usually wrap the abandoned wkhtmltopdf through wicked_pdf or PDFKit, drive headless Chrome through Grover, or write every layout as code with Prawn.

Oicana compiles PDFs in process through a native library instead. You design documents as [Typst](https://typst.app/) templates, load them once at startup, and render them from JSON in single-digit milliseconds. No browser process, no per-document fees, no document data leaving your infrastructure.

> **Free for noncommercial use.** Commercial use is free for 30 days, then needs a [per-application subscription](https://oicana.com/#pricing) with unlimited seats.

## Installation

```bash
gem install oicana     # or: bundle add oicana
```

Requires Ruby 3.2 or newer. Platform gems bring the native library for Linux (x64 and arm64, glibc 2.28 or newer, or musl), macOS (x64 and arm64), and Windows (x64 and arm64).

## Quick start

```ruby
require "oicana"

template = Oicana::Template.new(File.binread("invoice-0.1.0.zip"))

pdf = template.export_pdf(
  json_inputs: { "invoice" => { number: "2026-001", customer: "Acme GmbH", total: "€1,190.00" } }
)
File.binwrite("invoice.pdf", pdf)
```

JSON inputs take Strings holding JSON, or any value `JSON.generate` accepts. Blob inputs take binary Strings or `Oicana::BlobInput` values with metadata. `export_png` and `export_svg` produce the other formats, `pages:` takes a page index or a Range, `compile` keeps a document to export its pages separately, and `Template.export_once` renders a template once without caching it. Errors are subclasses of `Oicana::Error`.

## Running in a web service

Creating a `Template` compiles it once to warm up the Typst cache, so do it at startup, not per request. A template is safe to use from multiple threads, and native calls release the GVL. Templates and compiled documents are freed by the garbage collector, or right away with `close`.

## Licensing

Oicana is source-available under the [PolyForm Noncommercial License 1.0.0](https://github.com/oicana/oicana/blob/main/LICENSE.md) and free for noncommercial use. Commercial use is free for 30 days; see [pricing](https://oicana.com/#pricing) for subscriptions, or write to `hello@oicana.com`.
