#import "../../../src/lib.typ": setup

#let read-project-file(path) = return read(path, encoding: none);
#let (input, oicana-image, _) = setup(read-project-file);

A 2x2 PNG image rendered through `oicana-image`, where `image_format`
is the string `"png"`:

#oicana-image("logo", width: 4cm)
