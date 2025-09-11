#import "../../../src/lib.typ": setup

#let read-project-file(path) = return read(path, encoding: none);
#let (input, _, _) = setup(read-project-file);

Meta should contain an entry for "foo" with the value "bar":\
#input.file.meta
