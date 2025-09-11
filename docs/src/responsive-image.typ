#import "@preview/based:0.1.0": base64

#let responsive-image(image-path) = {
  context {
    if target() == "html" {
      let data = base64.encode(read(image-path, encoding: none))
      html.elem("img", attrs: (
        class: "full-width",
        src: "data:image/png;base64," + data,
      ))
    } else {
      image(image-path)
    }
  }
}
