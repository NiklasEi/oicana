#import "../../../src/lib.typ": setup

#let read-project-file(path) = return read(path, encoding: none);
#let (_, oicana-image, _) = setup(read-project-file);

#assert.eq(oicana-image("file"), none)
