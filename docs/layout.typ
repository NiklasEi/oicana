#let oicana-docs(
  version: none,
  abstract: none,
  body,
) = {
  show link: text.with(blue)

  let title = "Oicana"
  set document(title: title)

  set page(
    numbering: "1",
    number-align: center,
    header: context [
      #if here().page() == 1 {
        return []
      }
      #box(
        stroke: (bottom: 0.7pt),
        inset: 0.2em,
      )[#text(font: "New Computer Modern")[#title]]
    ],
  )

  set heading(numbering: "1.")
  show heading: it => {
    set text(font: "New Computer Modern")
    set par(first-line-indent: 0em)
    block[
      #if it.numbering != none {
        text(rgb("#2196F3"))[#counter(heading).display() ]
      }
      #it.body
      #v(0.6em)
    ]
  }

  set text(font: "New Computer Modern", lang: "en")

  align(center)[
    #set text(font: "New Computer Modern")
    #block(text(weight: 700, 25pt, title))
    #v(0.4em, weak: true)
    #if version != none [#text(18pt, weight: 500)[#version]]
  ]

  if abstract != none [#align(center)[#abstract]]


  // Main body.
  set par(justify: true)

  body
}
