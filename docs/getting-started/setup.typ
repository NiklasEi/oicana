#import "../src/boxes.typ": *
#import "../src/constants.typ": *
#import "../src/docs-link.typ": *

= Setup<getting-started-setup>

To get started with Oicana, you need the CLI and an editor for Typst. Both is only necessary on machines used for Oicana template development. An end-user machine running your software which uses an Oicana integration, does not require any additional installation.

== Oicana CLI

#link(latest-cli)[Builds are published on GitHub]. With the CLI, you can test and package Oicana templates.

See the #docs-link(<cli>, "../cli")[CLI section for more information].

== Typst Editor

You can edit Typst files in any text editor, but syntax highlighting and live previews make development significantly easier. Here are some suggestions:

- #link("https://typst.app/")[Official Typst editor in the browser].
  - This editor has a significant free tier and a pro tier for 8#sym.dollar per month. It is maintained by the Typst company.
  - No installation required.
  - You can work on the same document with multiple people and sync your templates to GitHub or GitLab.
- Several IDEs have Typst plugins with live previews and syntax highlighting.
  - Visual Studio Code #sym.arrow #link("https://marketplace.visualstudio.com/items?itemName=myriad-dreamin.tinymist")[Tinymist] offers a complete experience with life preview, syntax/error highlighting and more.
  - JetBrains IDEs #sym.arrow #link("https://plugins.jetbrains.com/plugin/25061-kvasir")[Kvasir] is in Beta and lags behind Tinymist feature-wise.
