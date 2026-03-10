LaTeX is a long-established typesetting system widely used in academia and publishing. Some applications use LaTeX (or derived engines like XeTeX, LuaLaTeX) to generate PDFs by compiling `.tex` files with injected data.

Oicana uses #link("https://typst.app")[Typst], a modern typesetting system that shares many of LaTeX's goals but takes a fundamentally different approach to syntax, compilation, and developer experience.

== How LaTeX-based PDF generation works

+ Create a `.tex` template with placeholders (often using a text templating engine)
+ Inject data into the template by replacing placeholders
+ Run the LaTeX compiler (`pdflatex`, `xelatex`, or `lualatex`) to produce a PDF

== Challenges with LaTeX for application PDF generation

=== Installation size

A typical LaTeX distribution (TeX Live) is 4–7 GB. Even minimal installations are hundreds of megabytes. This makes LaTeX impractical for containerized deployments, serverless functions, or client-side generation.

The Typst compiler (wrapped by Oicana) is a few megabytes. The WASM build runs directly in the browser.

=== Compilation speed

LaTeX compilation is slow — often several seconds per document. This can be acceptable for academic papers but problematic for on-demand PDF generation in applications.

Typst, and thus Oicana, can compile templates in single-digit milliseconds.

=== Steep learning curve

LaTeX's syntax is notoriously difficult to learn. Error messages are often cryptic, and debugging layout issues requires deep knowledge of the TeX engine.

Typst has a modern, readable syntax and produces clear error messages. Developers who have never used a typesetting system can be productive in very short time frames.

=== Escaping and injection

Injecting dynamic data into LaTeX templates is risky. LaTeX has many special characters (`\`, `{`, `}`, `%`, `$`, `&`, `#`, `_`, `^`, `~`) that need careful escaping. Improper escaping can break compilation or lead to unexpected output.

Oicana templates define explicit typed inputs. Data is passed through a structured API, not through string interpolation, eliminating injection issues entirely.

== When LaTeX might still be the right choice

- Your team already has deep LaTeX expertise and a large library of existing templates
- You need very specific typographic features that LaTeX's ecosystem uniquely provides

== Comparison at a glance

#table(
  columns: (auto, 1fr, 1fr),
  align: (left, left, left),
  table.header([], [*LaTeX*], [*Oicana*]),
  [Install size], [4–7 GB (TeX Live)], [#sym.tilde 15 MB (native library)],
  [Compilation speed], [Seconds], [Milliseconds],
  [Syntax], [Backslash commands, complex], [Modern, readable markup],
  [Error messages], [Cryptic], [Clear and actionable],
  [Data injection],
  [String replacement (escaping required)],
  [Typed inputs],

  [Browser support], [No], [Yes],
  [Package ecosystem], [Vast (CTAN)], [Growing (Typst Universe)],
)
