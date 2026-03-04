#import "@preview/shiroa:0.3.1": *

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
      "getting-started/3-defining-inputs.typ",
      section: "2.3",
    )[Defining Template Inputs]
  - #chapter(
      "getting-started/4-0-integrations.typ",
      section: "2.4",
    )[Choose Your Integration]
    - #chapter(
        "getting-started/4-1-csharp-integration.typ",
        section: "2.4.1",
      )[C#sym.hash using ASP.NET]
    - #chapter(
        "getting-started/4-2-rust-integration.typ",
        section: "2.4.2",
      )[Rust using Axum]
    - #chapter(
        "getting-started/4-3-nodejs-integration.typ",
        section: "2.4.3",
      )[Node.js using NestJS]
    - #chapter(
        "getting-started/4-4-python-integration.typ",
        section: "2.4.4",
      )[Python using FastAPI]
    - #chapter(
        "getting-started/4-5-php-integration.typ",
        section: "2.4.5",
      )[PHP using Slim]

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
    - #chapter("templates/export.typ", section: "3.4")[Export]
    - #chapter("templates/tests.typ", section: "3.5")[Testing]

  #v(2cm)

  - #chapter("cli.typ", section: "4")[CLI]
  - #chapter("integrations.typ", section: "5")[Integrations]
  - #chapter("guides.typ", section: "6")[Guides]
    - #chapter("guides/cache-management.typ", section: "6.1")[Cache Management]
  - #suffix-chapter("credits.typ")[Credits]
]
