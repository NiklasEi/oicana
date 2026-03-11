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
        "getting-started/4-1-browser-integration.typ",
        section: "2.4.1",
      )[Browser using React]
    - #chapter(
        "getting-started/4-2-csharp-integration.typ",
        section: "2.4.2",
      )[C#sym.hash using ASP.NET]
    - #chapter(
        "getting-started/4-3-java-integration.typ",
        section: "2.4.3",
      )[Java using Spring Boot]
    - #chapter(
        "getting-started/4-4-nodejs-integration.typ",
        section: "2.4.4",
      )[Node.js using NestJS]
    - #chapter(
        "getting-started/4-5-rust-integration.typ",
        section: "2.4.5",
      )[Rust using Axum]
    - #chapter(
        "getting-started/4-6-python-integration.typ",
        section: "2.4.6",
      )[Python using FastAPI]
    - #chapter(
        "getting-started/4-7-php-integration.typ",
        section: "2.4.7",
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

  == Comparisons
  - #chapter(
      "comparisons/vs-html-to-pdf.typ",
      section: "4.1",
    )[Oicana vs HTML-to-PDF]
  - #chapter(
      "comparisons/vs-pdf-libraries.typ",
      section: "4.2",
    )[Oicana vs PDF Libraries]
  - #chapter(
      "comparisons/vs-latex.typ",
      section: "4.3",
    )[Oicana vs LaTeX]
  - #chapter(
      "comparisons/vs-commercial-services.typ",
      section: "4.4",
    )[Oicana vs Commercial Services]

  == Reference
  - #chapter("cli.typ", section: "5")[CLI]
  - #chapter("integrations.typ", section: "6")[Integrations]
  - #chapter("guides.typ", section: "7")[Guides]
    - #chapter("guides/cache-management.typ", section: "7.1")[Cache Management]
  - #suffix-chapter("credits.typ")[Credits]
]
