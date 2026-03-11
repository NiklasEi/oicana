Commercial PDF generation services like DocRaptor, PSPDFKit Document Engine, Anvil, and similar platforms offer managed APIs for creating PDFs. You send a template and data to their API, and they return a PDF.

These services can be a quick way to get started, but they come with trade-offs around cost, data privacy, and control.

== How commercial PDF services work

+ Design a template in the service's editor or upload your own (HTML, DOCX, or proprietary format)
+ Call the service's API with your data
+ Receive the generated PDF in the response

== Challenges with commercial PDF services

=== Data leaves your infrastructure

Every PDF generation request sends your data — which may include personal information, financial data, or business-critical content — to a third-party server. This can be a compliance issue for regulations like GDPR, HIPAA, or SOC 2.

Oicana runs entirely in your infrastructure. Data never leaves your servers (or your users' browsers, when using the WASM integration).

=== Ongoing costs

Commercial services typically charge per document or per API call. At scale, these costs add up significantly. A service charging \$0.01 per document costs \$10,000 for a million documents.

Commercial use of Oicana comes with license costs. The costs are independent from the amount of documents you generate.

=== Vendor lock-in

Templates are often stored in a proprietary format or tied to the service's editor. Migrating away means rebuilding all your templates from scratch.

Oicana templates are standard Typst files. They can easily be ported to other Typst-based tools. The Typst compiler is open source!

=== Latency

Every PDF generation requires a network round-trip to the service's API. This adds latency, especially for applications that generate PDFs on user interaction.

Oicana generates PDFs locally in milliseconds. With the WASM integration, PDFs can be generated directly in the user's browser with zero network latency.

=== Availability dependency

Your PDF generation depends on the service's uptime. If they have an outage, your application cannot generate PDFs.

Oicana has no external dependencies at runtime.

== When commercial services might still be the right choice

- Your volume is low enough that per-document pricing is cheaper than developer time
- You need features beyond PDF generation (e.g. e-signatures, form filling, document workflows)

== Comparison at a glance

#table(
  columns: (auto, 1fr, 1fr),
  align: (left, left, left),
  table.header([], [*Commercial Services*], [*Oicana*]),
  [Data privacy], [Data sent to third party], [Stays in your infrastructure],
  [Cost model], [Per document / per API call], [Fixed cost],

  [Latency], [Network round-trip], [Local],
  [Availability], [Depends on service uptime], [No external dependency],
  [Template format], [Often proprietary], [Typst files],
  [Vendor lock-in], [High], [None — Typst is open source],
  [Browser generation], [No (API only)], [Yes (WASM)],
)
