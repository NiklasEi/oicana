use anyhow::bail;
use clap::Args;
use log::{debug, trace};
use oicana_template::manifest::TemplateManifest;
use oicana_testing::collect::{collect_tests, TemplateTests};
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Debug, Args)]
pub struct TargetArgs {
    #[arg(help = "Target the template at the given path")]
    template: Option<String>,
    #[clap(
        long,
        short,
        help = "Target all templates in the directory and any subdirectories"
    )]
    all: bool,
}

impl TargetArgs {
    pub fn get_targets(&self) -> anyhow::Result<Vec<TemplateDir>> {
        let mut templates = vec![];
        let path = match self.template {
            None => Path::new("."),
            Some(ref template) => Path::new(template),
        };

        if self.all {
            for maybe_template in WalkDir::new(path) {
                let Ok(dir_entry) = maybe_template else {
                    continue;
                };
                if !dir_entry.path().is_dir() {
                    continue;
                }
                trace!("checking {:?}", dir_entry.path());
                if let Some(manifest) = is_path_oicana_template(dir_entry.path())? {
                    templates.push(TemplateDir {
                        path: dir_entry.into_path(),
                        manifest,
                    });
                }
            }
        } else if let Some(manifest) = is_path_oicana_template(path)? {
            templates.push(TemplateDir {
                path: path.to_path_buf(),
                manifest,
            });
        }
        if templates.is_empty() {
            bail!(
                "No valid Oicana template found {} {:?}.",
                if self.all { "in" } else { "at" },
                path.canonicalize()
                    .expect("Failed to canonicalize path for error message")
            );
        }
        debug!("Targeting templates: {templates:?}");

        Ok(templates)
    }
}

pub fn is_path_oicana_template(path: &Path) -> anyhow::Result<Option<TemplateManifest>> {
    if !path.exists() {
        bail!("No such file or directory.")
    }
    if !path.is_dir() {
        bail!("Given path is not a directory.")
    }
    let possible_manifest_path = path.join("typst.toml");
    match possible_manifest_path.try_exists() {
        Err(error) => {
            bail!("Error while checking for existence of {possible_manifest_path:?}: {error:?}");
        }
        Ok(false) => {
            debug!("Skipping {path:?} since it doesn't contain a 'typst.toml' file.");
            return Ok(None);
        }
        _ => (),
    }
    let manifest = match read_to_string(&possible_manifest_path) {
        Err(error) => {
            bail!("Failed to read manifest {possible_manifest_path:?}: {error:?}");
        }
        Ok(manifest) => manifest,
    };
    let manifest = match TemplateManifest::from_toml(&manifest) {
        Err(error) => {
            debug!("{possible_manifest_path:?} is not a valid Oicana template manifest: {error:?}");
            return Ok(None);
        }
        Ok(manifest) => manifest,
    };

    Ok(Some(manifest))
}

#[derive(Debug)]
pub struct TemplateDir {
    pub path: PathBuf,
    pub manifest: TemplateManifest,
}

impl TemplateDir {
    /// Collect all tests found in this template directory
    pub fn gather_tests(&self) -> Result<TemplateTests, anyhow::Error> {
        let test_dir = self.path.join(&self.manifest.tool.oicana.tests);
        if !test_dir.exists() || !test_dir.is_dir() {
            debug!(
                "Template {} does not have a test directory at {:?}.",
                self.manifest.package.name, test_dir
            );
            return Ok(TemplateTests::default());
        }

        let tests = collect_tests(&test_dir)?;

        Ok(tests)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs::File, path::PathBuf};

    use oicana_template::{manifest::TemplateManifest, OicanaConfig};
    use tempfile::tempdir;
    use typst::syntax::package::PackageInfo;

    use super::TemplateDir;

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
    fn no_tests_without_test_dir() {
        let tempdir = tempdir().unwrap();

        let template_dir = TemplateDir {
            path: tempdir.path().into(),
            manifest: TemplateManifest::new(
                default_package_info(),
                OicanaConfig {
                    manifest_version: 1,
                    inputs: vec![],
                    tests: PathBuf::from("tests"),
                },
            ),
        };

        let mut result = template_dir.gather_tests();
        assert!(result.is_ok());
        assert!(result.unwrap().tests.is_empty());

        let temp_file = tempdir.path().join("tests");
        File::create(temp_file).unwrap();

        result = template_dir.gather_tests();
        assert!(result.is_ok());
        assert!(result.unwrap().tests.is_empty());
    }
}
