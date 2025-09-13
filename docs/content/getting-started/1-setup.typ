#import "/src/boxes.typ": *
#import "/src/constants.typ": *
#import "/src/docs-link.typ": *


To get started, you need the Oicana CLI and an editor for Typst. Both is only necessary on machines used for Oicana template development. An end-user machine, running software that uses an Oicana integration, does not require any additional installations.

== Oicana CLI

Install a #link(latest-cli)[prebuild binary from GitHub].

\
The CLI can test and package Oicana templates. See the #docs-link(<cli>, "../cli.html")[CLI section for more information].

== Editor for Typst

You can edit Typst files in any text editor, but syntax highlighting and live previews make development significantly easier. Here are some suggestions:


- Several IDEs have Typst plugins with live previews and syntax highlighting.
  - Visual Studio Code: #link("https://marketplace.visualstudio.com/items?itemName=myriad-dreamin.tinymist")[Tinymist] offers a complete experience with life preview, syntax/error highlighting and more.
  - JetBrains IDEs: #link("https://plugins.jetbrains.com/plugin/25061-kvasir")[Kvasir] is in Beta and lags behind Tinymist feature-wise.
- #link("https://typst.app/")[Official Typst editor in the browser].
  - This editor has a #link("https://typst.app/pricing/")[significant free tier and a pro tier for 8#sym.dollar per month]. It is maintained by the Typst company.
  - Web based; No installation required.
  - You can work on the same document with multiple people and sync your templates to GitHub or GitLab.

\
\
== For this Guide
In the following sections, you will create a basic Oicana template and set up an ASP.NET service. The service will use the Oicana C#sym.hash integration to compile the template to PDF when requested by a user.

\
Using the C#sym.hash integration requires a working .NET setup (preferably .NET 8). If you want to follow along, but don't have that installed, please refer to #link("https://learn.microsoft.com/en-us/dotnet/core/install/")[the official Microsoft guide for installation instructions].
