#import "../../../src/lib.typ": setup

#let read-project-file(path) = return read(path, encoding: none);
#let (input, _, _) = setup(read-project-file);

Should contain 6 inputs: #input.len()\

`default-blob` should have default value: #str(input.default-blob.bytes)\
`development-blob` should have development value: #str(input.development-blob.bytes)\
`both-blob` should have development value: #str(input.both-blob.bytes)\


`default-json` should have default value: #input.default-json.name\
`development-json` should have development value: #input.development-json.name\
`both-json` should have development value: #input.both-json.name\
