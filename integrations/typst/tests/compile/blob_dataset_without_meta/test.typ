#import "../../../src/lib.typ": setup

#let read-project-file(path) = return read(path, encoding: none);
#let (input, _, _) = setup(read-project-file);

Meta should be empty since the manifest does not define any default meta\
#input.file.meta
