#import "../../../src/lib.typ": setup


#let read-project-file(path) = return read(path, encoding: none);
#let (_, oicana-image, _) = setup(read-project-file);

/// Complains when a blob's `image_format` meta is not a valid image format
#let error = catch(() => oicana-image("file"));
#assert.eq(
  error,
  "expected \"png\", \"jpg\", \"gif\", \"webp\", dictionary, \"svg\", \"pdf\", or auto",
)
