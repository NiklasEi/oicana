#import "@preview/zebraw:0.5.5": *

#let code(header, body) = {
  context {
    if target() == "html" {
      body
    } else {
      return zebraw(
        inset: (top: 4pt, bottom: 4pt),
        numbering: false,
        header: [*#header*],
        lang: true,
        lang-color: eastern,
        lang-font-args: (
          font: "libertinus serif",
          fill: white,
          weight: "bold",
        ),
        body,
      )
    }
  }
}
