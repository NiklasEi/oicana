use crate::{OicanaConfig, PdfStandard};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Component, Path};
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
    /// Never touches the filesystem. Use [`Self::validate_at`]
    /// when the template root is available for more checks.
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

        let tests = &self.tool.oicana.tests;
        if tests.is_absolute()
            || tests
                .components()
                .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(ManifestValidationError::InvalidTestsPath);
        }

        Ok(())
    }

    /// Validate the manifest including filesystem checks against the template root.
    ///
    /// On top of [`Self::validate`], this checks that the `tests` path does not point
    /// to an existing non-directory inside the template.
    pub fn validate_at(&self, template_root: &Path) -> Result<(), ManifestValidationError> {
        self.validate()?;

        let tests = template_root.join(&self.tool.oicana.tests);
        if tests.exists() && !tests.is_dir() {
            return Err(ManifestValidationError::InvalidTestsPath);
        }

        Ok(())
    }

    /// Build a gitignore-style matcher from the Typst `package.exclude` patterns.
    ///
    /// The test directory (configured via `tool.oicana.tests`) and the `output/`
    /// directory are always excluded by default. User-supplied patterns extend
    /// these defaults and can re-include the defaults with a leading `!`
    /// (gitignore semantics: later patterns override earlier ones).
    pub fn build_exclude_matcher(&self) -> Gitignore {
        let mut builder = GitignoreBuilder::new("");
        let test_dir = self.tool.oicana.tests.to_string_lossy().replace('\\', "/");
        builder
            .add_line(None, &format!("/{test_dir}/"))
            .expect("test directory exclude pattern should be valid");
        builder
            .add_line(None, "/output/")
            .expect("output directory exclude pattern should be valid");
        for pattern in &self.package.exclude {
            if let Err(error) = builder.add_line(None, pattern.as_str()) {
                log::warn!("Ignoring invalid exclude pattern '{pattern}': {error}");
            }
        }
        builder.build().expect("exclude patterns should be valid")
    }

    /// Parse toml to a manifest
    pub fn from_toml(toml_content: &str) -> Result<Self, toml::de::Error> {
        toml::de::from_str::<TemplateManifest>(toml_content)
    }

    /// The PDF standards configured for this template's export.
    pub fn pdf_standards(&self) -> &[PdfStandard] {
        &self.tool.oicana.export.pdf.standards
    }

    /// Whether this template's PDF export should be tagged.
    pub fn pdf_tagged(&self) -> bool {
        self.tool.oicana.export.pdf.tagged
    }

    /// Font families this template expects its host to provide.
    pub fn required_font_families(&self) -> &[String] {
        &self.tool.oicana.fonts.require
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
#[derive(Debug, Error, PartialEq, Eq)]
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

/// Whether a string is a valid Oicana template name.
///
/// Template names follow Typst identifier rules: must start with a letter or
/// underscore, and may contain letters, digits, `_`, and `-`.
pub fn is_valid_template_name(name: &str) -> bool {
    is_ident(name)
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
    use std::path::{Path, PathBuf};

    use typst::syntax::package::PackageInfo;

    use crate::{
        manifest::{ManifestValidationError, TemplateManifest},
        ExportConfig, FontConfig, OicanaConfig,
    };

    fn default_package_info() -> PackageInfo {
        PackageInfo::new("test-package", "0.1.0".parse().unwrap(), "main.typ")
    }

    /// Helper: returns true if the path should be excluded from packing.
    fn is_excluded(manifest: &TemplateManifest, path: &str, is_dir: bool) -> bool {
        let matcher = manifest.build_exclude_matcher();
        matcher
            .matched_path_or_any_parents(Path::new(path), is_dir)
            .is_ignore()
    }

    #[test]
    fn package_exclude_patterns_extend_defaults() {
        let mut package_info = default_package_info();
        package_info.exclude = vec!["docs/*.pdf".into(), "/assets*/".into()];
        let manifest = TemplateManifest::new(
            package_info,
            OicanaConfig {
                manifest_version: 1,
                inputs: vec![],
                validate_json_inputs_by_default: true,
                tests: PathBuf::from("tests"),
                export: ExportConfig::default(),
                fonts: FontConfig::default(),
            },
        );

        // Default exclusions still apply alongside the user's patterns.
        assert!(is_excluded(&manifest, "tests", true));
        assert!(is_excluded(&manifest, "tests/file.txt", false));
        assert!(is_excluded(&manifest, "tests/sub_dir", true));
        assert!(!is_excluded(&manifest, "test", true));
        assert!(!is_excluded(&manifest, "sub_dir/tests", true));

        assert!(is_excluded(&manifest, "output", true));
        assert!(is_excluded(&manifest, "output/result.pdf", false));
        assert!(!is_excluded(&manifest, "sub_dir/output", true));

        // User patterns also apply.
        assert!(is_excluded(&manifest, "docs/manual.pdf", false));
        assert!(!is_excluded(&manifest, "docs/readme.md", false));
        assert!(is_excluded(&manifest, "assets_old", true));
        assert!(!is_excluded(&manifest, "src/assets_old", true));
    }

    #[test]
    fn negation_re_includes_default_excluded_dirs() {
        let mut package_info = default_package_info();
        // `!` patterns negate earlier matches, so defaults can be opted back in.
        package_info.exclude = vec!["!/tests/".into(), "!/output/".into()];
        let manifest = TemplateManifest::new(
            package_info,
            OicanaConfig {
                manifest_version: 1,
                inputs: vec![],
                validate_json_inputs_by_default: true,
                tests: PathBuf::from("tests"),
                export: ExportConfig::default(),
                fonts: FontConfig::default(),
            },
        );

        assert!(!is_excluded(&manifest, "tests", true));
        assert!(!is_excluded(&manifest, "tests/file.txt", false));
        assert!(!is_excluded(&manifest, "output", true));
        assert!(!is_excluded(&manifest, "output/result.pdf", false));
    }

    #[test]
    fn negation_re_includes_custom_tests_dir() {
        let mut package_info = default_package_info();
        package_info.exclude = vec!["!/custom_tests/".into()];
        let manifest = TemplateManifest::new(
            package_info,
            OicanaConfig {
                manifest_version: 1,
                inputs: vec![],
                validate_json_inputs_by_default: true,
                tests: PathBuf::from("custom_tests"),
                export: ExportConfig::default(),
                fonts: FontConfig::default(),
            },
        );

        assert!(!is_excluded(&manifest, "custom_tests", true));
        assert!(!is_excluded(&manifest, "custom_tests/file.txt", false));
        // The output default is untouched.
        assert!(is_excluded(&manifest, "output", true));
    }

    #[test]
    fn empty_exclude_defaults_to_tests_and_output() {
        let manifest = TemplateManifest::new(
            default_package_info(),
            OicanaConfig {
                manifest_version: 1,
                inputs: vec![],
                validate_json_inputs_by_default: true,
                tests: PathBuf::from("tests"),
                export: ExportConfig::default(),
                fonts: FontConfig::default(),
            },
        );

        assert!(is_excluded(&manifest, "tests", true));
        assert!(is_excluded(&manifest, "tests/file.txt", false));
        assert!(is_excluded(&manifest, "output", true));
        assert!(is_excluded(&manifest, "output/result.pdf", false));
        assert!(!is_excluded(&manifest, "src/main.typ", false));
        assert!(!is_excluded(&manifest, "sub_dir/tests", true));
        assert!(!is_excluded(&manifest, "sub_dir/output", true));
    }

    #[test]
    fn empty_exclude_respects_custom_tests_dir() {
        let manifest = TemplateManifest::new(
            default_package_info(),
            OicanaConfig {
                manifest_version: 1,
                inputs: vec![],
                validate_json_inputs_by_default: true,
                tests: PathBuf::from("custom_tests"),
                export: ExportConfig::default(),
                fonts: FontConfig::default(),
            },
        );

        assert!(is_excluded(&manifest, "custom_tests", true));
        assert!(!is_excluded(&manifest, "tests", true));
        assert!(is_excluded(&manifest, "output", true));
    }

    fn manifest_with_tests_path(tests: PathBuf) -> TemplateManifest {
        TemplateManifest::new(
            default_package_info(),
            OicanaConfig {
                manifest_version: 1,
                inputs: vec![],
                validate_json_inputs_by_default: true,
                tests,
                export: ExportConfig::default(),
                fonts: FontConfig::default(),
            },
        )
    }

    #[test]
    fn validates_that_tests_dir_is_relative() {
        let manifest = manifest_with_tests_path(PathBuf::from(".").canonicalize().unwrap());

        assert_eq!(
            manifest.validate(),
            Err(ManifestValidationError::InvalidTestsPath)
        );
    }

    #[test]
    fn validates_that_tests_dir_does_not_leave_the_template_root() {
        let manifest = manifest_with_tests_path(PathBuf::from("../outside"));

        assert_eq!(
            manifest.validate(),
            Err(ManifestValidationError::InvalidTestsPath)
        );
    }

    #[test]
    fn validate_at_resolves_tests_against_the_given_root() {
        let manifest = manifest_with_tests_path(PathBuf::from("tests"));

        // A file named `tests` in the template root is invalid...
        let root_with_file = tempfile::tempdir().unwrap();
        std::fs::File::create(root_with_file.path().join("tests")).unwrap();
        assert_eq!(
            manifest.validate_at(root_with_file.path()),
            Err(ManifestValidationError::InvalidTestsPath)
        );

        // ...a directory or no entry at all is fine.
        let root_with_dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(root_with_dir.path().join("tests")).unwrap();
        assert_eq!(manifest.validate_at(root_with_dir.path()), Ok(()));

        let empty_root = tempfile::tempdir().unwrap();
        assert_eq!(manifest.validate_at(empty_root.path()), Ok(()));
    }
}
