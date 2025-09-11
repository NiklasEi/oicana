#import "../src/boxes.typ": *

== Custom Fonts <fonts>

To use any font in an Oicana template, add a `.ttf`, `.ttc`, `.otf`, or `.otc` file to the project. The location of the file in the template is not relevant, it can even be part of an imported package. Some Typst editors, like the official web app, also support font files as part of a Typst project and will use them in their preview. If you use an IDE plugin for Typst development, the settings of said plugin might support loading additional fonts for the preview.

The fonts "Libertinus Serif", "New Computer Modern", "DejaVu Sans Mono", and "New Computer Modern Math" are included in Typst by default and always available in Oicana templates.

#example[Take a look at the #link("https://github.com/oicana/oicana-example-templates/tree/main/templates/fonts")[example Oicana template `fonts`]. It demonstrates how to include and use custom fonts.]
