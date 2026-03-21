use crate::target::TargetArgs;
use anyhow::Context;
use clap::Args;
use console::{style, Emoji};
use log::info;
use oicana_files::native::{package_data_dir, NativeTemplate};
use oicana_files::TemplateFiles;
use oicana_template::package::package_with_dependencies;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs::{create_dir_all, read_dir, File};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use typst::syntax::ast::ModuleImport;

static PACKAGE: Emoji<'_, '_> = Emoji("📦", "");
use typst::syntax::package::PackageSpec;
use typst::syntax::{ast, FileId, VirtualPath};

#[derive(Debug, Args)]
pub struct PackArgs {
    #[clap(flatten)]
    target: TargetArgs,
    #[clap(short, long, help = "Output directory", default_value = ".")]
    out_dir: String,
    #[clap(
        short,
        long,
        help = "Name template for the artifacts",
        default_value = "{template}-{version}.zip"
    )]
    name: String,
}

#[rustfmt::skip]
pub const PACK_AFTER_HELP: &str = color_print::cstr!("\
<s><u>Examples:</></>
  oicana pack templates/invoice
  oicana pack templates/invoice -o out
  oicana pack -a
  oicana pack templates -a
");

pub fn pack(args: PackArgs) -> anyhow::Result<()> {
    let templates = args.target.get_targets()?;
    let out = Path::new(&args.out_dir);
    let packages = package_data_dir().context("Failed to find data directory for packages")?;

    for template in templates {
        info!("Packing template '{}'.", template.manifest.package.name);
        template.manifest.validate()?;

        let files = NativeTemplate::new(&template.path, packages.clone());

        let dependencies = collect_dependencies(&template.path, &files)?;

        create_dir_all(out)?;
        let out_file_path = out.join(
            args.name
                .replace("{template}", &template.manifest.package.name)
                .replace("{version}", &template.manifest.package.version.to_string()),
        );
        let out_file = File::create(&out_file_path).context("Failed to create the zip file")?;

        // Otherwise the zip file includes a partial version of itself if `pack` is called in the template directory
        let exclude = out_file_path.canonicalize().ok().and_then(|abs_out| {
            template.path.canonicalize().ok().and_then(|abs_template| {
                abs_out
                    .strip_prefix(abs_template)
                    .ok()
                    .map(Path::to_path_buf)
            })
        });

        let exclude_matcher = template.manifest.build_exclude_matcher();
        package_with_dependencies(
            &template.path,
            out_file,
            &exclude_matcher,
            exclude.as_deref(),
            &dependencies,
        )?;

        println!(
            "{PACKAGE}  {} packed to {}",
            style(&template.manifest.package.name).bold(),
            style(out_file_path.display()).cyan(),
        );
    }

    Ok(())
}

/// Collect all package dependencies by scanning imports in template `.typ` files.
///
/// Returns a list of `(source_dir, zip_prefix)` pairs where `source_dir` is the
/// package's location on disk and `zip_prefix` is the path it should have in the zip
/// (e.g. `.dependencies/preview/pkg/0.1.0`).
fn collect_dependencies(
    root: &Path,
    files: &NativeTemplate,
) -> anyhow::Result<Vec<(PathBuf, PathBuf)>> {
    let mut collected = HashSet::new();
    let mut result = Vec::new();

    scan_imports_in_dir(root, root, files, &mut collected, &mut result)?;

    Ok(result)
}

fn scan_imports_in_dir(
    root: &Path,
    dir: &Path,
    files: &NativeTemplate,
    collected: &mut HashSet<PackageSpec>,
    result: &mut Vec<(PathBuf, PathBuf)>,
) -> anyhow::Result<()> {
    if dir.file_name().and_then(OsStr::to_str) == Some(".dependencies") {
        return Ok(());
    }
    for entry in read_dir(dir).context("Failed to read directory")? {
        let entry = entry?;
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let path = dir.join(entry.file_name());
        if meta.is_dir() {
            scan_imports_in_dir(root, &path, files, collected, result)?;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("typ") {
            let fid =
                FileId::new(
                    None,
                    VirtualPath::new(path.strip_prefix(root).context(
                        "Prefix stripping failed even though `path` is built from `root`",
                    )?),
                );
            let source = files.source(fid).context("Can't read source file")?;
            let imports = source
                .root()
                .children()
                .filter_map(|ch| ch.cast::<ModuleImport>());
            for import in imports {
                let ast::Expr::Str(source_str) = import.source() else {
                    continue;
                };

                if let Ok(import_spec) = PackageSpec::from_str(source_str.get().as_str()) {
                    if collected.insert(import_spec.clone()) {
                        let package_dir = files
                            .package_dir(&import_spec)
                            .context(format!("Failed to resolve package {import_spec}"))?;

                        let zip_prefix = PathBuf::from(format!(
                            ".dependencies/{}/{}/{}",
                            import_spec.namespace, import_spec.name, import_spec.version
                        ));
                        result.push((package_dir.clone(), zip_prefix));

                        // Recursively scan the package directory for transitive dependencies
                        scan_imports_in_package(
                            &package_dir,
                            &import_spec,
                            files,
                            collected,
                            result,
                        )?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Scan a package directory for imports to find transitive dependencies.
fn scan_imports_in_package(
    package_dir: &Path,
    package_spec: &PackageSpec,
    files: &NativeTemplate,
    collected: &mut HashSet<PackageSpec>,
    result: &mut Vec<(PathBuf, PathBuf)>,
) -> anyhow::Result<()> {
    for entry in walkdir::WalkDir::new(package_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("typ") {
            continue;
        }

        let relative = path.strip_prefix(package_dir).unwrap();
        let fid = FileId::new(Some(package_spec.clone()), VirtualPath::new(relative));
        let source = files
            .source(fid)
            .context("Can't read source file in package")?;
        let imports = source
            .root()
            .children()
            .filter_map(|ch| ch.cast::<ModuleImport>());

        for import in imports {
            let ast::Expr::Str(source_str) = import.source() else {
                continue;
            };

            if let Ok(import_spec) = PackageSpec::from_str(source_str.get().as_str()) {
                if collected.insert(import_spec.clone()) {
                    let dep_dir = files.package_dir(&import_spec).context(format!(
                        "Failed to resolve transitive package {import_spec}"
                    ))?;

                    let zip_prefix = PathBuf::from(format!(
                        ".dependencies/{}/{}/{}",
                        import_spec.namespace, import_spec.name, import_spec.version
                    ));
                    result.push((dep_dir.clone(), zip_prefix));

                    scan_imports_in_package(&dep_dir, &import_spec, files, collected, result)?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::fs::create_dir_all;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_mock_package(packages_dir: &Path, spec: &PackageSpec, content: &str) {
        let package_dir =
            packages_dir.join(format!("{}/{}/{}", spec.namespace, spec.name, spec.version));
        create_dir_all(&package_dir).unwrap();
        let mut f = File::create(package_dir.join("package.typ")).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        // Also create a typst.toml so the package is valid
        let mut m = File::create(package_dir.join("typst.toml")).unwrap();
        m.write_all(
            format!(
                r#"[package]
name = "{}"
version = "{}"
entrypoint = "package.typ"
"#,
                spec.name, spec.version
            )
            .as_bytes(),
        )
        .unwrap();
    }

    #[test]
    fn no_dependencies_for_template_without_imports() {
        let tempdir = tempdir().unwrap();
        let temp_template = tempdir.path().join("template");
        create_dir_all(&temp_template).unwrap();
        let temp_packages = tempdir.path().join("cache");
        create_dir_all(&temp_packages).unwrap();
        {
            let file_path = temp_template.join("test.typ");
            let mut tmp_file = File::create(file_path).unwrap();
            tmp_file
                .write_all("This Typst file has no imports!".as_bytes())
                .unwrap();
        }
        let files = NativeTemplate::new(&temp_template, temp_packages);

        let deps = collect_dependencies(&temp_template, &files).unwrap();
        assert!(deps.is_empty());
        assert_eq!(
            temp_template.join(".dependencies").try_exists().ok(),
            Some(false)
        );
    }

    #[test]
    fn resolves_dependencies() {
        let tempdir = tempdir().unwrap();
        let temp_template = tempdir.path().join("template");
        create_dir_all(&temp_template).unwrap();
        let temp_packages = tempdir.path().join("cache");
        {
            let file_path = temp_template.join("test.typ");
            let mut tmp_file = File::create(file_path).unwrap();
            tmp_file
                .write_all(
                    "#import \"@local/test:0.1.0\": *\nThis Typst file imports the test package."
                        .as_bytes(),
                )
                .unwrap();
        }
        let spec = PackageSpec::from_str("@local/test:0.1.0").unwrap();
        create_mock_package(&temp_packages, &spec, "Some package content");

        let files = NativeTemplate::new(&temp_template, temp_packages);
        let deps = collect_dependencies(&temp_template, &files).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].1, PathBuf::from(".dependencies/local/test/0.1.0"));
        // No .dependencies created on disk
        assert_eq!(
            temp_template.join(".dependencies").try_exists().ok(),
            Some(false)
        );
    }

    #[test]
    fn resolves_transitive_dependencies() {
        let tempdir = tempdir().unwrap();
        let temp_template = tempdir.path().join("template");
        create_dir_all(&temp_template).unwrap();
        let temp_packages = tempdir.path().join("cache");
        {
            let file_path = temp_template.join("test.typ");
            let mut tmp_file = File::create(file_path).unwrap();
            tmp_file
                .write_all(
                    "#import \"@local/test:0.1.0\": *\nThis Typst file imports the test package."
                        .as_bytes(),
                )
                .unwrap();
        }
        let spec = PackageSpec::from_str("@local/test:0.1.0").unwrap();
        let spec2 = PackageSpec::from_str("@local/test2:0.1.0").unwrap();
        create_mock_package(
            &temp_packages,
            &spec,
            "#import \"@local/test2:0.1.0\": *\nSome package content with import",
        );
        create_mock_package(&temp_packages, &spec2, "Some other package content");

        let files = NativeTemplate::new(&temp_template, temp_packages);
        let deps = collect_dependencies(&temp_template, &files).unwrap();

        assert_eq!(deps.len(), 2);
        let zip_prefixes: HashSet<_> = deps.iter().map(|(_, p)| p.clone()).collect();
        assert!(zip_prefixes.contains(&PathBuf::from(".dependencies/local/test/0.1.0")));
        assert!(zip_prefixes.contains(&PathBuf::from(".dependencies/local/test2/0.1.0")));
        // No .dependencies created on disk
        assert_eq!(
            temp_template.join(".dependencies").try_exists().ok(),
            Some(false)
        );
    }
}
