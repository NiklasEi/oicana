#import "@preview/shiroa:0.2.3": *

#let summary = [
  - #prefix-chapter("intro.typ")[Intro]
  == Getting Started
  - #chapter(
      "getting-started/1-setup.typ",
      section: "2.1",
    )[Installation and Setup]
  - #chapter(
      "getting-started/2-first-template.typ",
      section: "2.2",
    )[Create an Oicana template]
  - #chapter(
      "getting-started/3-c-sharp-integration.typ",
      section: "2.3",
    )[#text("Using the C# Integration")]
  - #chapter("getting-started/4-inputs.typ", section: "2.4")[Template inputs]

  == Templating
  - #chapter("templates.typ", section: "3")[Oicana templates]
    - #chapter("templates/inputs.typ", section: "3.1")[Inputs]
    - #chapter(
        "templates/dependencies.typ",
        section: "3.2",
      )[Dependencies]
      - #chapter(
          "templates/helpful-packages.typ",
          section: "3.2.1",
        )[Helpful Packages]
    - #chapter("templates/fonts.typ", section: "3.3")[Fonts]
    - #chapter("templates/tests.typ", section: "3.4")[Testing]

  #v(2cm)

  - #chapter("cli.typ", section: "4")[CLI]
  - #chapter("integrations.typ", section: "5")[Integrations]
  - #chapter("guides.typ", section: "6")[Guides]
  - #suffix-chapter("credits.typ")[Credits]
]
