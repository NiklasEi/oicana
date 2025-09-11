= Guides

A collection of concrete problems and solutions for using Oicana.

== Structured Inputs

Document templates in applications often need to be customizable. For example, a user might want to customize the footer of a given document. A common requirement is that these customizations need to be styled. In the footer, the users might want multiple blocks of text in a grid and some bold or underlined sections. We can support this by passing Typst code into the template instead of plain text and using #link("https://typst.app/docs/reference/foundations/eval/")[`#eval`] in the template to render the given Typst code as the footer.

If our users know Typst, we are done at this point. But users often don't know Typst and might not be technical at all. In these cases, the input used to customize the template should likely be a WYSIWYG editor.

Currently, there is no production-ready WYSIWYG editor for Typst (though keep an eye on https://github.com/tyx-editor/TyX). A well supported format for WYSIWYG editors is HTML. You can use a tool like pandoc to convert from the format your editor exports to Typst and then pass the generated Typst code into the template.

== Plots / Data visualizing

Sometimes, a plot or diagram conveys information more effectively than a table. There are a couple of community developed Typst packages that can help with that:

- #link("https://typst.app/universe/package/cetz")[cetz] - Drawing with Typst made easy, providing an API inspired by TikZ and Processing. Includes modules for plotting, charts and tree layout.
- #link("https://typst.app/universe/package/fletcher")[fletcher] - Draw diagrams with nodes and arrows.
- #link("https://typst.app/universe/package/lilaq")[lilaq] - Scientific data visualization.

The #link("https://github.com/oicana/oicana-example-templates/tree/main/templates/dependency")["dependency"] example template uses cetz to create a piechart.
