#import "/src/boxes.typ": *

Templates can be exported to PDF, SVG, and PNG.

== PDF export

The PDF export supports #link("https://typst.app/docs/reference/pdf/#pdf-standards")[all standards that Typst has to offer]. The default in Oicana is PDF/A-3b (PDF 1.7).\
\
You can configure the default PDF standards to use when exporting a given template in the template's manifest file.\
\
```toml
[tool.oicana.export.pdf]
standards = ["ua-1"]
```

The manifest above would configure this template to #link("https://typst.app/docs/reference/pdf/#pdf-ua")[produce PDF files for Universal Access].


== PNG export

In many scenarios, PNG export is an easy option for previews.

Oicana integrations allow configuring the pixels per point in a PNG export. A smaller ratio leads to faster file generation and smaller files, but lower resolution. The default is $1"px"/"pt"$.

#note[Please note that the PNG export is optimized for speed, not file size. Before you save the images or send them over the network, consider optimizing them.]
