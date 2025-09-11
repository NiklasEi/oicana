#let docs-link(tag, url, content) = {
  context {
    if target() == "html" {
      link(url, content)
    } else {
      link(tag, content)
    }
  }
}
