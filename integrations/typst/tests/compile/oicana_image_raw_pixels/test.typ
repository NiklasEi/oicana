#import "../../../src/lib.typ": setup

#let read-project-file(path) = return read(path, encoding: none);
#let (input, oicana-image, _) = setup(read-project-file);

A 2x2 raw rgb8 image rendered through `oicana-image`, where `image_format`
is a dictionary describing the pixel encoding rather than a string format:

#oicana-image("logo", width: 4cm)
