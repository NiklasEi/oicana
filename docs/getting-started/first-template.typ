#import "../src/code.typ": *
#import "../src/boxes.typ": *

== Create an Oicana template

Create a new directory and open it in your IDE with Typst support, or start a new empty document in the online Typst editor. The simplest Oicana template consists of a `typst.toml` file and a Typst source file with some static content.

Create a `main.typ` file with the following content (the online editor should already have created an empty `main.typ` for you):

#code("main.typ", "getting-started-first-template-static-typst-file")[
  ```typst
  = Hello from Typst

  Compiling this template will always result in the same document.
  We will add inputs later on in the #text(fill: blue, weight: "bold")[Getting Started Guide].
  ```]

#note[If you are new to Typst, this might look a bit confusing to you. The "=" above is a heading and the normal text is just that, normal text.

  A "\#" denotes a function call. Here, we use the built-in `text` function to write "Getting Started Guide" in blue and bold. Please refer to #link("https://typst.app/docs/")[Typst's online documentation] for more information.]

The preview of `main.typ` should show the title and text.

#note[The online Typst editor automatically shows a preview. In an IDE, you might need to open the preview. For example, if you use VS code with `Tinymist`, press `Ctrl` + `K` followed by `V` while the `main.typ` file is open.]

To make an Oicana template out of this Typst document, we need to put a `typst.toml` file next to `main.typ`.

#code("typst.toml", "getting-started-first-template-typst-toml")[
  ```toml
  [package]
  name = "example"
  version = "0.1.0"
  entrypoint = "main.typ"

  [tool.oicana]
  manifest_version = 1
  ```]

This manifest file gives our template the name `example`, the semantic version `0.1.0`, and defines the entrypoint to the Typst project as `main.typ`. The `tool.oicana` section only configures the Oicana manifest version for now.

To prepare the template for compilation out of Oicana integrations, like the C#sym.hash one we are about to use, it needs to be packaged. Navigate into the template directory in a terminal and execute `oicana pack`. This will create a file called `example-0.1.0.zip`.
