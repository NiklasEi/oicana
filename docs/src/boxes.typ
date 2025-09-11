#import "@preview/showybox:2.0.4": showybox
#import "@preview/theorion:0.4.0": *
#import cosmos.rainbow: *
#show: show-theorion


#let title-style = (
  weight: 900,
  color: red.darken(40%),
  sep-thickness: 0pt,
  align: center,
)

#let frame = color => {
  return (
    title-color: color.lighten(80%),
    border-color: color.darken(40%),
    thickness: (left: 1pt),
    radius: 0pt,
  )
}

#let docs-box = (color, title, body) => {
  context {
    if target() == "html" {
      html.elem("div", attrs: (
        style: "border-inline-start: .25em solid "
          + color.to-hex()
          + "; padding: .1em 1em; width: 100%; box-sizing: border-box; margin-bottom: .5em; overflow: auto;",
      ))[
        #html.elem(
          "p",
          attrs: (
            style: "margin-top: .5em; font-weight: bold; color: "
              + color.to-hex()
              + ";",
          ),
          title,
        )
        #body
      ]
    } else {
      return showybox(
        title-style: title-style,
        frame: frame(color),
        title: title,
      )[#body]
    }
  }
}

#let alpha-note(body) = {
  docs-box(red, "Early Alpha", body)
}

#let example(body) = {
  docs-box(blue, "Example", body)
}

#let note(body) = {
  docs-box(blue, "Note", body)
}
