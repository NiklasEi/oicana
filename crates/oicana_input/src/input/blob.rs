use crate::Input;
use typst::foundations::{Bytes, Dict, Str, Value};

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
}

/// A blob with metadata.
#[derive(Clone, Debug)]
pub struct Blob {
    /// The bytes of the Blob.
    pub bytes: Bytes,
    /// Metadata containing mostly optional info like an image format.
    pub metadata: Dict,
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
