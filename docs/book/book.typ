#import "@preview/shiroa:0.2.3": *

#show: book


// re-export page template
#import "page.typ": heading-reference, project as book-page
#import "summary.typ": summary

#book-meta(
  title: "Oicana",
  description: "Oicana Documentation",
  repository: "https://github.com/oicana/oicana",
  language: "en",
  summary: summary,
)
