use crate::Input;
use typst::foundations::{Bytes, Dict, IntoValue, Str, Value};

/// A blob input with its key and value.
#[derive(Clone, Debug)]
pub struct BlobInput {
    /// The key of the input.
    ///
    /// This corresponds to the identifier of an input definition in the manifest.
    pub key: Str,
    /// The blob value.
    pub value: Blob,
}

impl BlobInput {
    /// Create a new blob input.
    pub fn new(key: impl Into<Str>, value: impl Into<Blob>) -> Self {
        BlobInput {
            key: key.into(),
            value: value.into(),
        }
    }

    /// Create a blob input for an image, setting the `image_format` metadata
    /// entry that the `oicana-image` helper uses to pick a decoder.
    pub fn image<B>(key: impl Into<Str>, bytes: B, format: impl Into<Str>) -> Self
    where
        B: AsRef<[u8]> + Send + Sync + 'static,
    {
        BlobInput::new(
            key,
            Blob::with_metadata(bytes, [("image_format", format.into())]),
        )
    }
}

/// A blob with metadata.
#[derive(Clone, Debug)]
pub struct Blob {
    /// The bytes of the Blob.
    pub bytes: Bytes,
    /// Metadata containing mostly optional info like an image format.
    pub metadata: Dict,
}

impl Blob {
    /// Create a blob with the given bytes and metadata entries.
    pub fn with_metadata<B, K, V, I>(bytes: B, metadata: I) -> Self
    where
        B: AsRef<[u8]> + Send + Sync + 'static,
        K: Into<Str>,
        V: IntoValue,
        I: IntoIterator<Item = (K, V)>,
    {
        let mut dict = Dict::new();
        for (key, value) in metadata {
            dict.insert(key.into(), value.into_value());
        }
        Blob {
            bytes: Bytes::new(bytes),
            metadata: dict,
        }
    }
}

impl From<Bytes> for Blob {
    fn from(bytes: Bytes) -> Self {
        Blob {
            bytes,
            metadata: Dict::new(),
        }
    }
}

impl From<Vec<u8>> for Blob {
    fn from(bytes: Vec<u8>) -> Self {
        Blob {
            bytes: Bytes::new(bytes),
            metadata: Dict::new(),
        }
    }
}

impl From<Blob> for Dict {
    fn from(value: Blob) -> Self {
        let mut dict = Dict::new();
        dict.insert("bytes".into(), Value::Bytes(value.bytes));
        dict.insert("meta".into(), Value::Dict(value.metadata));

        dict
    }
}

impl Input for BlobInput {
    fn key(&self) -> Str {
        self.key.clone()
    }

    fn to_value(self) -> Value {
        Value::Dict(self.value.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use typst::foundations::{Array, IndexMap};

    #[test]
    fn build_blob_input() {
        let blob_input = BlobInput::new("blob", Bytes::new([4u8].as_slice()));

        let blob = blob_input.to_value();
        let Value::Dict(mut blob) = blob else {
            panic!("blob is not a dict");
        };

        assert_eq!(blob.len(), 2);
        assert_eq!(
            blob.remove("bytes".into(), None).unwrap(),
            Value::Bytes(Bytes::new([4u8].as_slice()))
        );
        assert_eq!(
            blob.remove("meta".into(), None).unwrap(),
            Value::Dict(Dict::new())
        );
    }

    #[test]
    fn blob_with_metadata_collects_string_entries() {
        let blob = Blob::with_metadata(
            vec![1u8, 2, 3],
            [
                ("image_format", Str::from("png")),
                ("variant", Str::from("dark")),
            ],
        );

        assert_eq!(blob.bytes, Bytes::new([1u8, 2, 3].as_slice()));
        assert_eq!(blob.metadata.len(), 2);
        assert_eq!(
            blob.metadata
                .at("image_format".into(), None)
                .expect("image_format missing"),
            Value::Str("png".into())
        );
        assert_eq!(
            blob.metadata
                .at("variant".into(), None)
                .expect("variant missing"),
            Value::Str("dark".into())
        );
    }

    #[test]
    fn blob_with_metadata_accepts_bool() {
        let blob = Blob::with_metadata(vec![1u8], [("flag", true)]);

        assert_eq!(
            blob.metadata.at("flag".into(), None).expect("flag missing"),
            Value::Bool(true)
        );
    }

    #[test]
    fn blob_with_metadata_mixed_value_types() {
        let blob = Blob::with_metadata(
            vec![1u8, 2, 3],
            [
                ("image_format", Value::Str("png".into())),
                ("flag", Value::Bool(true)),
            ],
        );

        assert_eq!(blob.metadata.len(), 2);
        assert_eq!(
            blob.metadata
                .at("image_format".into(), None)
                .expect("image_format missing"),
            Value::Str("png".into())
        );
        assert_eq!(
            blob.metadata.at("flag".into(), None).expect("flag missing"),
            Value::Bool(true)
        );
    }

    #[test]
    fn blob_input_image_sets_format_metadata() {
        let blob_input = BlobInput::image("logo", vec![9u8, 8, 7], "png");

        assert_eq!(blob_input.key, Str::from("logo"));
        let blob = blob_input.to_value();
        let Value::Dict(mut blob) = blob else {
            panic!("blob is not a dict");
        };
        let Value::Dict(meta) = blob.remove("meta".into(), None).unwrap() else {
            panic!("meta is not a dict");
        };
        assert_eq!(meta.len(), 1);
        assert_eq!(
            meta.at("image_format".into(), None)
                .expect("image_format missing"),
            Value::Str("png".into())
        );
    }

    #[test]
    fn build_blob_input_with_meta() {
        let blob_input = BlobInput::new(
            "blob",
            Blob {
                bytes: Bytes::new([1u8, 2, 3].as_slice()),
                metadata: {
                    let mut meta = Dict::new();
                    meta.insert("format".into(), Value::Str("png".into()));
                    meta.insert(
                        "custom".into(),
                        Value::Array(Array::from_iter(vec![
                            Value::Str("value1".into()),
                            Value::Str("value2".into()),
                        ])),
                    );

                    meta
                },
            },
        );

        let blob = blob_input.to_value();
        let Value::Dict(mut blob) = blob else {
            panic!("blob is not a dict");
        };

        assert_eq!(blob.len(), 2);
        assert_eq!(
            blob.remove("bytes".into(), None).unwrap(),
            Value::Bytes(Bytes::new([1u8, 2, 3].as_slice()))
        );
        assert_eq!(
            blob.remove("meta".into(), None).unwrap(),
            Value::Dict(Dict::from(IndexMap::from_iter(vec![
                ("format".into(), Value::Str("png".into())),
                (
                    "custom".into(),
                    Value::Array(Array::from_iter(vec![
                        Value::Str("value1".into()),
                        Value::Str("value2".into())
                    ]))
                )
            ])))
        );
    }
}
