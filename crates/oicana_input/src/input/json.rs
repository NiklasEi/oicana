use crate::Input;
use typst::foundations::{IntoValue, Str, Value};

/// A JSON input.
pub struct JsonInput {
    /// The key of the input.
    ///
    /// This corresponds to the identifier of an input definition in the manifest.
    pub key: Str,
    /// Stringified JSON as the input.
    pub value: String,
}

impl JsonInput {
    /// Create a new JSON input with given key and value.
    pub fn new(key: impl Into<Str>, value: impl Into<String>) -> Self {
        JsonInput {
            key: key.into(),
            value: value.into(),
        }
    }
}

impl Input for JsonInput {
    fn key(&self) -> Str {
        self.key.clone()
    }

    fn to_value(self) -> Value {
        self.value.into_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_json_input() {
        let content = r#"{
            "key": "value",
            "object": {
                "foo": "bar"
            }
            }"#;
        let json_input = JsonInput::new("json", content);

        let json = json_input.to_value();

        let Value::Str(json_str) = json else {
            panic!("JSON input is not converted to Value::Str");
        };
        assert_eq!(json_str.as_str(), content);
    }
}
