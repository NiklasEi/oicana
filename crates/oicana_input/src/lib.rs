//! Definitions for Oicana inputs.

use log::warn;
use serde::{Deserialize, Serialize};
use typst::foundations::{Dict, Str, Value};

/// Input values.
pub mod input;
/// Definitions of inputs for Oicana templates.
pub mod input_definition;

/// An input value.
pub trait Input {
    /// The key of the input.
    ///
    /// This is the identifier of the input definition this input value belongs to.
    fn key(&self) -> Str;

    /// Create a Typst value to be passed into the template.
    fn to_value(self) -> Value;
}

/// Combine template inputs.
#[derive(Debug)]
pub struct TemplateInputs {
    inputs: Dict,
    config: CompilationConfig,
}

impl Default for TemplateInputs {
    fn default() -> Self {
        TemplateInputs::new()
    }
}

impl TemplateInputs {
    /// Create a new and empty inputs collection.
    pub fn new() -> Self {
        TemplateInputs {
            inputs: Dict::new(),
            config: CompilationConfig::development(),
        }
    }

    /// Add a compilation configuration to the template inputs.
    pub fn with_config(&mut self, config: CompilationConfig) -> &mut Self {
        self.config = config;
        self
    }

    /// Add an input to the collection.
    pub fn with_input<I: Input>(&mut self, input: I) -> &mut Self {
        if self.inputs.contains(&input.key()) {
            warn!("An input is overwriting a previous input value!");
        }
        self.inputs.insert(input.key(), input.to_value());
        self
    }

    /// Build the Typst [`Dict`] that contains all previously added inputs and configuration.
    pub fn to_dict(self) -> Dict {
        let mut combined_inputs = Dict::new();
        combined_inputs.insert("oicana-inputs".into(), Value::Dict(self.inputs));
        combined_inputs.insert("oicana-config".into(), Value::Dict(self.config.into()));

        combined_inputs
    }
}

/// Configuration for template compilation
///
/// These values are passed into the template
#[derive(Debug)]
pub struct CompilationConfig {
    mode: CompilationMode,
}

/// Modes of compilation
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CompilationMode {
    /// Compile the template in production mode, ignoring development values for inputs.
    #[serde(alias = "Production", alias = "PRODUCTION", alias = "prod")]
    Production,
    /// Compile the template in development mode using development values of inputs if configured.
    #[serde(alias = "Development", alias = "DEVELOPMENT", alias = "dev")]
    Development,
}

impl CompilationMode {
    fn is_production(&self) -> bool {
        matches!(self, CompilationMode::Production)
    }
}

impl CompilationConfig {
    /// Create a new configuration
    pub fn new(mode: CompilationMode) -> Self {
        CompilationConfig { mode }
    }

    /// Configuration for a production template compilation
    ///
    /// This will prevent the template from using fallback input values
    pub fn production() -> Self {
        CompilationConfig {
            mode: CompilationMode::Production,
        }
    }

    /// Configuration for a development template compilation
    ///
    /// This will allow the template to use fallback input values
    pub fn development() -> Self {
        CompilationConfig {
            mode: CompilationMode::Development,
        }
    }
}

impl From<CompilationConfig> for Dict {
    fn from(value: CompilationConfig) -> Self {
        let mut dict = Dict::new();
        dict.insert("production".into(), Value::Bool(value.mode.is_production()));

        dict
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::blob::BlobInput;
    use crate::input::json::JsonInput;
    use typst::foundations::Bytes;

    #[test]
    fn combines_blob_and_string_inputs() {
        let mut inputs = TemplateInputs::new();
        let json_input = "{\"foo\": \"bar\"}".to_string();
        inputs
            .with_input(JsonInput::new("data", json_input))
            .with_input(BlobInput::new("blob1", Bytes::new([1u8, 2, 3].as_slice())))
            .with_input(BlobInput::new("blob2", Bytes::new([4u8].as_slice())));

        let Value::Dict(inputs) = inputs
            .to_dict()
            .at("oicana-inputs".into(), None)
            .expect("No inputs built!")
        else {
            panic!("Inputs should be dictionary!")
        };

        assert_eq!(inputs.len(), 3);
        assert!(inputs.contains("data"));
        assert!(inputs.contains("blob1"));
        assert!(inputs.contains("blob2"));
    }

    #[test]
    fn inputs_with_same_key_overwrite_old_value() {
        let mut inputs = TemplateInputs::new();
        let json_input = "{\"foo\": \"bar\"}".to_string();
        inputs
            .with_input(JsonInput::new("data", json_input))
            .with_input(BlobInput::new("blob1", Bytes::new([1u8, 2, 3].as_slice())))
            .with_input(BlobInput::new("data", Bytes::new([4u8].as_slice())));

        let Value::Dict(inputs) = inputs
            .to_dict()
            .at("oicana-inputs".into(), None)
            .expect("No inputs built!")
        else {
            panic!("Inputs should be dictionary!")
        };

        assert_eq!(inputs.len(), 2);
        assert!(inputs.contains("data"));
        assert!(inputs.contains("blob1"));
        assert!(!inputs.contains("blob2"));
    }

    #[test]
    fn sets_dev_compilation_mode() {
        let mut inputs = TemplateInputs::new();
        inputs.with_config(CompilationConfig::development());

        let Value::Dict(config) = inputs
            .to_dict()
            .at("oicana-config".into(), None)
            .expect("No config built!")
        else {
            panic!("Config should be dictionary!")
        };

        assert_eq!(
            config
                .at("production".into(), None)
                .expect("Mode should be in compilation config"),
            Value::Bool(false)
        );
    }

    #[test]
    fn sets_prod_compilation_mode() {
        let mut inputs = TemplateInputs::new();
        inputs.with_config(CompilationConfig::production());

        let Value::Dict(config) = inputs
            .to_dict()
            .at("oicana-config".into(), None)
            .expect("No config built!")
        else {
            panic!("Config should be dictionary!")
        };

        assert_eq!(
            config
                .at("production".into(), None)
                .expect("Mode should be in compilation config"),
            Value::Bool(true)
        );
    }

    #[test]
    fn sets_default_compilation_mode() {
        let inputs = TemplateInputs::new();

        let Value::Dict(config) = inputs
            .to_dict()
            .at("oicana-config".into(), None)
            .expect("No config built!")
        else {
            panic!("Config should be dictionary!")
        };

        assert_eq!(
            config
                .at("production".into(), None)
                .expect("Mode should be in compilation config"),
            Value::Bool(false)
        );
    }
}
