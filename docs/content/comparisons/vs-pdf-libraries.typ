PDF libraries let you construct PDF documents programmatically by calling API methods to draw text, shapes, and images on a page. Popular libraries include iText (Java/.NET), PDFKit (Node.js), FPDF/TCPDF (PHP), ReportLab (Python), and Apache PDFBox (Java).

This approach gives you full control over every pixel, but comes at a cost.

== How PDF libraries work

With a PDF library, you typically write code like this:

+ Create a document object
+ Add pages and set dimensions
+ Position text, images, tables, and shapes using coordinates or a layout API
+ Write the resulting bytes to a file or response stream

The template _is_ the code. There is no separate template file.

== Challenges with PDF libraries

=== Templates live in code

Since the layout is defined in application code, changing a template means changing, testing, and redeploying your application. Designers and non-developers cannot edit templates without developer involvement.

With Oicana, templates are standalone Typst files. They can be edited, previewed, and tested independently of the application.

=== Language lock-in

An iText template written in Java cannot be reused in a Node.js service. If your organization uses multiple languages, you end up maintaining separate PDF generation code for each stack.

Oicana templates are language-agnostic. The same template works with the Java, C\#, Node.js, Rust, Python, PHP, and browser integrations.

=== Tedious layout work

Positioning elements with coordinates or building table layouts through API calls is time-consuming and error-prone. Simple changes like adjusting spacing often require recompiling and inspecting the output.

Typst provides a high-level markup language with good layout control, reusable components, and a package ecosystem. You can build and reuse different layouts or rely on existing ones.

=== Limited previewing

Most PDF libraries require you to run your code to see the output. There is no live preview during development, which slows down the design iteration cycle.

Oicana templates can be previewed in any Typst editor, givign you instant feedback.

== When PDF libraries might still be the right choice

If you need to manipulate existing PDFs (merge, split, annotate, fill forms).

== Comparison at a glance

#table(
  columns: (auto, 1fr, 1fr),
  align: (left, left, left),
  table.header([], [*PDF Libraries*], [*Oicana*]),
  [Template format], [Application code], [Typst markup files],
  [Editable by designers], [No], [Yes],
  [Cross-language reuse], [No], [Yes],
  [Live preview], [No], [Yes],
  [Layout approach], [Coordinates / API calls], [Declarative markup],
  [Learning curve], [Library API + PDF spec], [Typst markup],
  [PDF manipulation], [Yes], [Generation only],
)
