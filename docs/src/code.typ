#import "@preview/zebraw:0.5.5": *

#let data-url(mime, src) = {
  import "@preview/based:0.2.0": base64
  "data:" + mime + ";base64," + base64.encode(src)
}

#let code-counter = counter("code-on-page")

#let copy-function(content, id) = (
  "function copyToClipboard"
    + id
    + "() {
  navigator.clipboard.writeText(`"
    + content.replace("`", "\`")
    + "`)
    .catch(err => console.error('Error copying:', err));
}"
)

#let code(header, body) = {
  context {
    let id = code-counter.display()
    code-counter.step()
    if target() == "html" {
      html.elem("div", attrs: (
        style: "overflow: auto; width: 100%; margin-bottom: .5em; position: relative;",
      ))[
        #body
        #html.elem("script", attrs: (
          type: "text/javascript",
          src: data-url("application/javascript", copy-function(body.text, id)),
        ))
        #html.elem(
          "style",
          [.zebraw-code-line{ padding-top: 0 !important; padding-bottom: 0 !important; }],
        )
        #html.elem("button", attrs: (
          onclick: "copyToClipboard" + id + "()",
          style: "position: absolute; top: 10px; right: 10px; padding: 3px;",
        ))[Copy]
      ]
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
