use std::{
    collections::HashMap,
    io,
    path::{Component, Path, PathBuf},
    str,
};

use log::trace;
use thiserror::Error;
use walkdir::WalkDir;

use crate::{PrepareTestError, TemplateTestCollection, Test, TestCollectionError};

/// Collect all tests from collection in a given directory
///
/// This method will search recursively from the given path
/// and collect all tests from test collections. Any file ending with
/// `tests.toml` will be interpreted as a test collection.
pub fn collect_tests(test_dir: &Path) -> Result<TemplateTests, CollectTestsError> {
    let walk_dir = WalkDir::new(test_dir);
    let entries = walk_dir
        .into_iter()
        .filter_entry(|entry| {
            let name = entry.file_name().to_string_lossy();
            if entry.file_type().is_file() && !name.ends_with("tests.toml") {
                trace!("File '{name}' does not end with 'tests.toml', skipping...");
                return false;
            }

            true
        })
        .filter_map(|e| e.ok());

    let mut tests = vec![];
    for entry in entries {
        if !entry.file_type().is_file() {
            continue;
        }
        let collection_path = entry.path();
        trace!("Reading test collection at {collection_path:?}");
        let root = collection_path.parent().ok_or(CollectTestsError::Io {
            file_path: collection_path.into(),
            source: io::Error::new(
                io::ErrorKind::InvalidInput,
                "A test collection file needs to have a parent as it is inside the test directory.",
            ),
        })?;
        let test_path_components: Vec<String> = root
            .strip_prefix(test_dir)
            .ok()
            .map(|path| {
                path.components()
                    .filter_map(|comp| match comp {
                        Component::Normal(s) => s.to_str().map(|s| s.to_string()),
                        _ => None,
                    })
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        let tests_collection = TemplateTestCollection::read_from(collection_path)?;

        for test_case in tests_collection.tests {
            tests.push(Test::new(
                test_case,
                tests_collection.name.clone(),
                &test_path_components,
                collection_path,
                root,
            )?);
        }
    }

    tests.sort_by(|a, b| a.descriptor.cmp(&b.descriptor));
    let warnings = duplication_warnings(&tests);

    Ok(TemplateTests { tests, warnings })
}

fn duplication_warnings(tests: &Vec<Test>) -> Vec<String> {
    let mut groups: HashMap<&str, Vec<&Test>> = HashMap::new();
    for test in tests {
        groups.entry(&test.descriptor).or_default().push(test);
    }
    groups
        .iter()
        .filter_map(|(description, tests)| {
            let count = tests.len();
            if count < 2 {
                None
            } else {
                Some(format!(
                    "There is {count} tests with the full name '{description}'."
                ))
            }
        })
        .collect()
}

/// Tests collected from a single template
#[derive(Default, Debug)]
pub struct TemplateTests {
    /// Tests included in the template
    pub tests: Vec<Test>,
    /// Warnings from the collected tests
    pub warnings: Vec<String>,
}

/// Errors that can be produced when collection test cases
#[derive(Debug, Error)]
pub enum CollectTestsError {
    /// Error opening or reading file
    #[error("Failed to read file '{file_path}': {source}")]
    Io {
        /// The path of the file causing the error
        file_path: PathBuf,
        /// The io error
        #[source]
        source: io::Error,
    },
    /// Error opening or reading file
    #[error("Issue with a test collection")]
    TestCollection(#[from] TestCollectionError),
    /// Error while preparing a test case
    #[error("Issue with a test collection")]
    PrepareTestError(#[from] PrepareTestError),
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use std::path::Path;
    use tempfile::tempdir;

    use crate::collect::TemplateTests;

    use super::collect_tests;

    #[test]
    fn collect_all_tests() {
        let tests_dir = tempdir().unwrap();
        let base_collection = tests_dir.path().join("tests.toml");
        let first_collection = tests_dir.path().join("bar.tests.toml");
        let second_collection = tests_dir.path().join("dir").join("ignored.tests.toml");
        write_collection(&base_collection, None);
        write_collection(&first_collection, None);
        write_collection(&second_collection, Some("name".to_owned()));

        let TemplateTests { tests, warnings } =
            collect_tests(tests_dir.path()).expect("Failed to collect tests");
        assert!(warnings.is_empty());
        assert_eq!(tests.len(), 6);

        assert!(tests
            .iter()
            .any(|test| test.name == "foo" && test.descriptor == "bar > foo"));
        assert!(tests
            .iter()
            .any(|test| test.name == "bar" && test.descriptor == "bar > bar"));

        assert!(tests
            .iter()
            .any(|test| test.name == "foo" && test.descriptor == "foo"));
        assert!(tests
            .iter()
            .any(|test| test.name == "bar" && test.descriptor == "bar"));

        assert!(tests
            .iter()
            .any(|test| test.name == "foo" && test.descriptor == "dir > name > foo"));
        assert!(tests
            .iter()
            .any(|test| test.name == "bar" && test.descriptor == "dir > name > bar"));
    }

    #[test]
    fn warn_for_duplicate_tests() {
        let tests_dir = tempdir().unwrap();
        let base_collection = tests_dir.path().join("bar.tests.toml");
        let second_collection = tests_dir.path().join("bar").join("tests.toml");
        write_collection(&base_collection, None);
        write_collection(&second_collection, None);

        let TemplateTests { tests, warnings } =
            collect_tests(tests_dir.path()).expect("Failed to collect tests");
        assert_eq!(warnings.len(), 2);
        assert_eq!(tests.len(), 4);

        assert!(warnings.contains(&"There is 2 tests with the full name 'bar > bar'.".to_owned()));
        assert!(warnings.contains(&"There is 2 tests with the full name 'bar > foo'.".to_owned()));
    }

    fn write_collection(path: &Path, name: Option<String>) {
        _ = create_dir_all(path.parent().unwrap());
        let mut file = File::create(path).unwrap();
        write!(
            &mut file,
            r#"
                {}
                tests_version = 1
    
                [[test]]
                name = "foo"
    
                [[test]]
                name = "bar"
                snapshot = "foo.png" # use same snapshot as the test above
                "#,
            name.map(|name| format!("name = \"{name}\""))
                .unwrap_or_default()
        )
        .unwrap();

        let _ = File::create(path.parent().unwrap().join("foo.png")).unwrap();
    }
}
