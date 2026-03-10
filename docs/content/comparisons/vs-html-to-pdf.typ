HTML-to-PDF conversion is one of the most popular approaches for generating PDFs in web applications. Tools like wkhtmltopdf, Puppeteer, WeasyPrint, and Gotenberg render HTML/CSS in a browser engine and can produce a PDF from the result.

While this approach leverages existing web development skills, it comes with significant trade-offs.

== How it works

The typical HTML-to-PDF workflow looks like this:

+ Construct an HTML document (often using a templating engine like Handlebars, Jinja2, or Razor)
+ Render it in a headless browser (Chromium, WebKit) or a dedicated rendering engine
+ Export the rendered page as a PDF

== Challenges with HTML-to-PDF

=== Heavy runtime dependency

Most HTML-to-PDF tools require a full browser engine. Chromium alone adds hundreds of megabytes to your deployment. This impacts container sizes, cold start times, and resource consumption.

Oicana's Typst-based compiler is lightweight — the native libraries are a few megabytes, and the WASM build can run directly in the browser without any server-side dependency.

=== Inconsistent layout

CSS was designed for screens, not for paged documents. Getting reliable page breaks, headers, footers, and precise positioning requires fighting against the CSS box model. Different browser versions can produce different results.

Typst was designed for document layout from the ground up. Page breaks, headers, footers, margins, and multi-column layouts work predictably and consistently.

=== Performance

Spinning up a headless browser, loading HTML, and rendering to PDF typically takes hundreds of milliseconds to seconds per document. Under load, browser instances compete for memory and CPU.

Oicana compiles templates to PDF in single-digit milliseconds. No browser process, no DOM rendering, no waiting for fonts to load.

=== Security concerns

Running a full browser engine to process potentially untrusted content introduces a large attack surface. Headless Chrome has had numerous security vulnerabilities, and sandboxing it properly in a server environment is non-trivial.

Oicana templates are compiled by Typst, which has a minimal attack surface and does not execute arbitrary code.

== When HTML-to-PDF might still be the right choice

- You already have complex HTML templates and the migration cost is too high
- You need to render content that is inherently web-based (e.g. screenshots of dashboards)
- Your team has deep CSS/HTML expertise but no capacity to learn a new tool

== Comparison at a glance

#table(
  columns: (auto, 1fr, 1fr),
  align: (left, left, left),
  table.header([], [*HTML-to-PDF*], [*Oicana*]),
  [Runtime size], [100+ MB (browser engine)], [#sym.tilde 15 MB (native library)],
  [Compilation speed], [100ms – seconds], [1 – 10ms],
  [Layout model], [CSS (screen-first)], [Typst (document-first)],
  [Page breaks], [Fragile], [Built-in, predictable],
  [Headers/Footers], [Limited, browser-dependent], [First-class support],
  [Template language], [HTML + templating engine], [Typst markup],
  [Server dependency], [Headless browser required], [Native library or WASM],
)
