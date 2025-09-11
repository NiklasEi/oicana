#import "layout.typ": oicana-docs

#show: oicana-docs.with(
  version: "0.1.0",
  abstract: [
    A set of libraries and tools to use the open-source typesetter #link("https://typst.app/")[Typst] for PDF templating.

    Oicana simplifies writing Typst documents that can take programmatic input from different tech stacks. For example, you can preview an invoice template in your React frontend and compile it through your C#sym.hash backend. Before compiling, prepare inputs in any way you see fit.
  ],
)

#include "intro.typ"
#include "getting-started.typ"
#include "getting-started/setup.typ"
#include "getting-started/first-template.typ"
#include "getting-started/c-sharp-integration.typ"
#include "getting-started/inputs.typ"
#include "templates.typ"
#include "oicana-templates/development.typ"
#include "oicana-templates/inputs.typ"
#include "oicana-templates/dependencies.typ"
#include "oicana-templates/helpful-packages.typ"
#include "oicana-templates/fonts.typ"
#include "guides.typ"
