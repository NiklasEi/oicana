
#import "@preview/shiroa:0.2.0": *

#show: book

#book-meta(
  title: "Oicana",
  description: "Oicana Documentation",
  repository: "https://github.com/oicana/oicana",
  language: "en",
  summary: [
    - #chapter("intro.typ")[Oicana]
    - #chapter("getting-started.typ", section: "2")[Getting Started]
      - #chapter(
          "getting-started/setup.typ",
          section: "2.1",
        )[Set up Typst and Oicana]
      - #chapter(
          "getting-started/first-template.typ",
          section: "2.2",
        )[Create an Oicana template]
      - #chapter(
          "getting-started/c-sharp-integration.typ",
          section: "2.3",
        )[#text("C# Integration")]
      - #chapter("getting-started/inputs.typ", section: "2.4")[Template inputs]
    - #chapter("templates.typ", section: "3")[Oicana templates]
      - #chapter("oicana-templates/inputs.typ", section: "3.1")[Inputs]
      - #chapter(
          "oicana-templates/dependencies.typ",
          section: "3.2",
        )[Dependencies]
        - #chapter(
            "oicana-templates/helpful-packages.typ",
            section: "3.2.1",
          )[Helpful Packages]
      - #chapter("oicana-templates/fonts.typ", section: "3.3")[Fonts]
    - #chapter("integrations.typ", section: "4")[Integrations]
    - #chapter("guides.typ", section: "5")[Guides]
    - #chapter("credits.typ", section: "6")[Credits]
  ],
)


// re-export page template
#import "templates/page.typ": project
#let book-page = project
