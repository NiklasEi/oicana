#import "@preview/zebraw:0.5.5": *

#let data-url(mime, src) = {
  import "@preview/based:0.2.0": base64
  "data:" + mime + ";base64," + base64.encode(src)
}

#let copy-function = "function copyToClipboard(id) {
var copyText = document.getElementById(id);
const pre = copyText.querySelector('pre');
if (pre) {
  let html = pre.innerHTML;
  html = html.replace(/<br\s*\/?>/gi, '\\n');
  const plainText = html.replace(/<[^>]*>/g, '');
  console.log(plainText);
  navigator.clipboard.writeText(plainText)
    .catch(err => console.error('Error copying:', err));
} else {
  navigator.clipboard.writeText(copyText.textContent.trim())
    .catch(err => console.error('Error copying:', err));
}
}"

#let local-code(header, id, body) = {
  context {
    if target() == "html" {
      html.elem("div", attrs: (
        style: "overflow: auto; background-color: var(--searchbar-bg); color: var(--searchbar-fg); padding: .1em 1em; width: 100%; box-sizing: border-box; margin-bottom: .5em; position: relative;",
      ))[#html.elem("div", attrs: (id: id))[#body] #html.elem("script", attrs: (
          type: "text/javascript",
          src: data-url("application/javascript", copy-function),
        ))
        #html.elem("button", attrs: (
          onclick: "copyToClipboard(\"" + id + "\")",
          style: "position: absolute; top: 5px; right: 5px; padding: 3px;",
        ))[Copy]]
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
