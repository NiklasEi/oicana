use crate::OicanaConfig;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use thiserror::Error;
use typst::diag::EcoString;
use typst::syntax::package::{PackageInfo, TemplateInfo, UnknownFields};
use unicode_ident::{is_xid_continue, is_xid_start};

/// An Oicana template's relevant information.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemplateManifest {
    /// Package information in the manifest.
    pub package: PackageInfo,
    /// Details about the template, if the package is one.
    #[serde(default)]
    pub template: Option<TemplateInfo>,
    /// Tool section of the manifest.
    pub tool: ToolSection,
    /// All parsed but unknown fields, this can be used for validation.
    #[serde(flatten, skip_serializing)]
    pub unknown_fields: UnknownFields,
}

impl TemplateManifest {
    /// Create a new template manifest from `PackageInfo` and a
    pub fn new(package: PackageInfo, templating_config: OicanaConfig) -> Self {
        TemplateManifest {
            package,
            template: None,
            unknown_fields: BTreeMap::new(),
            tool: ToolSection::new(templating_config),
        }
    }

    /// validate the Typst package part of the manifest.
    ///
    /// This follows Typst's own package validation and checks Oicana specific rules on top.
    pub fn validate(&self) -> Result<(), ManifestValidationError> {
        let mut unknown_keys: Vec<_> = self.unknown_fields.keys().map(String::from).collect();
        unknown_keys.extend(
            self.package
                .unknown_fields
                .keys()
                .map(String::from)
                .map(|key| format!("package.{key}"))
                .collect::<Vec<_>>(),
        );
        if !unknown_keys.is_empty() {
            return Err(ManifestValidationError::UnknownManifestKeys(unknown_keys));
        }

        if !is_ident(&self.package.name) {
            return Err(ManifestValidationError::InvalidTemplateName);
        }

        let oicana_config = &self.tool.oicana;

        if !oicana_config.tests.is_relative()
            || (oicana_config.tests.exists() && !oicana_config.tests.is_dir())
        {
            return Err(ManifestValidationError::InvalidTestsPath);
        }

        Ok(())
    }

    /// Checks if the given path in a template should be packed
    pub fn should_path_be_packed(&self, path: &Path) -> bool {
        // we need to ensure that both start with './' if relative
        let relative_test = format_relative_path(&self.tool.oicana.tests);
        let relative_path = format_relative_path(path);

        !relative_path.starts_with(&relative_test)
    }

    /// Parse toml to a manifest
    pub fn from_toml(toml_content: &str) -> Result<Self, toml::de::Error> {
        toml::de::from_str::<TemplateManifest>(toml_content)
    }
}

fn format_relative_path(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('\\', "/");

    if !path.is_relative() {
        return normalized;
    }

    match normalized.as_str() {
        "." => "./".to_string(),
        ".." => "..".to_string(),
        "" => "./".to_string(),
        _ if normalized.starts_with("./") || normalized.starts_with("../") => normalized,
        _ => format!("./{normalized}"),
    }
}

/// Tool section of a Typst package manifest.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolSection {
    /// The Oicana part in the tool section.
    pub oicana: OicanaConfig,
    /// Any other fields parsed in the tool section.
    #[serde(flatten)]
    _sections: BTreeMap<EcoString, toml::Table>,
}

impl ToolSection {
    /// Create a new tool section with the given Oicana config
    pub fn new(config: OicanaConfig) -> Self {
        Self {
            oicana: config,
            _sections: BTreeMap::new(),
        }
    }
}

/// Error from the manifest file.
#[derive(Debug, Error)]
pub enum ManifestValidationError {
    /// The manifest contains unknown keys.
    #[error("Unknown keys found in the manifest.")]
    UnknownManifestKeys(Vec<String>),
    /// The template name must be a valid Typst identifier.
    #[error("The template name is not a valid identifier.")]
    InvalidTemplateName,
    /// Value of 'tests' needs to be a relative path from the template root to a directory.
    #[error("Value of 'tests' needs to be a relative path from the template root to a directory.")]
    InvalidTestsPath,
}

/// Whether a string is a valid Typst identifier.
fn is_ident(string: &str) -> bool {
    let mut chars = string.chars();
    chars
        .next()
        .is_some_and(|c| is_id_start(c) && chars.all(is_id_continue))
}

/// Whether a character can start an identifier.
fn is_id_start(c: char) -> bool {
    is_xid_start(c) || c == '_'
}

/// Whether a character can continue an identifier.
fn is_id_continue(c: char) -> bool {
    is_xid_continue(c) || c == '_' || c == '-'
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        path::{Path, PathBuf},
    };

    use typst::syntax::package::PackageInfo;

    use crate::{
        manifest::{format_relative_path, ManifestValidationError, TemplateManifest},
        OicanaConfig,
    };

    // ToDo: use new methods from https://github.com/typst/typst/pull/6625 when released
    fn default_package_info() -> PackageInfo {
        PackageInfo {
            name: "test-package".into(),
            version: "0.1.0".parse().unwrap(),
            entrypoint: "main.typ".into(),
            authors: vec![],
            categories: vec![],
            compiler: None,
            description: None,
            disciplines: vec![],
            exclude: vec![],
            homepage: None,
            keywords: vec![],
            license: None,
            repository: None,
            unknown_fields: BTreeMap::new(),
        }
    }

    #[test]
    fn ignores_files_in_tests_dir() {
        let manifest = TemplateManifest::new(
            default_package_info(),
            OicanaConfig {
                manifest_version: 1,
                inputs: vec![],
                tests: PathBuf::from("tests"),
            },
        );

        assert!(!manifest.should_path_be_packed(Path::new("./tests")));
        assert!(!manifest.should_path_be_packed(Path::new("./tests/file.txt")));
        assert!(!manifest.should_path_be_packed(Path::new("./tests/sub_dir")));
        assert!(!manifest.should_path_be_packed(Path::new("./tests/sub_dir/file.txt")));

        assert!(manifest.should_path_be_packed(Path::new("./test")));
        assert!(manifest.should_path_be_packed(Path::new("./sub_dir/tests")));
        assert!(manifest.should_path_be_packed(Path::new("./sub_dir/tests/file.txt")));
        assert!(manifest.should_path_be_packed(Path::new("./sub_dir/tests/sub_dir")));
        assert!(manifest.should_path_be_packed(Path::new("./sub_dir/tests/sub_dir/file.txt")));
    }

    #[test]
    fn validates_that_tests_dir_is_relative() {
        let manifest = TemplateManifest::new(
            default_package_info(),
            OicanaConfig {
                manifest_version: 1,
                inputs: vec![],
                tests: PathBuf::from("/absolute"),
            },
        );

        assert!(matches!(
            manifest.validate(),
            Err(ManifestValidationError::InvalidTestsPath)
        ));
    }

    #[test]
    fn should_format_relative_path() {
        assert_eq!(format_relative_path(Path::new("foo")), "./foo");
        assert_eq!(format_relative_path(Path::new("foo/bar")), "./foo/bar");
        assert_eq!(format_relative_path(Path::new("./foo")), "./foo");
        assert_eq!(format_relative_path(Path::new("../foo")), "../foo");
        assert_eq!(format_relative_path(Path::new(".")), "./");
        assert_eq!(format_relative_path(Path::new("")), "./");
    }
}
