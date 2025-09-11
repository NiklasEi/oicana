use std::path::Path;

use oicana_files::native::{package_data_dir, NativeTemplate};
use oicana_input::TemplateInputs;
use oicana_template::manifest::TemplateManifest;
use oicana_world::{manifest::OicanaWorldFiles, world::OicanaWorld};

use crate::{Template, TemplateInitializationError};

impl Template<NativeTemplate> {
    /// Initialize the given template
    pub fn init(path: &Path) -> Result<Self, TemplateInitializationError> {
        let files = NativeTemplate::new(
            path,
            package_data_dir().ok_or(TemplateInitializationError::PackageDirectoryNotFound)?,
        );
        let manifest = files.manifest()?;

        let world = OicanaWorld::new(files, TemplateInputs::new(), manifest)?;

        Ok(Template { world })
    }

    /// Create a native template from all required parts
    pub fn from(
        template_root: &Path,
        packages: &Path,
        manifest: TemplateManifest,
    ) -> Result<Self, TemplateInitializationError> {
        let files = NativeTemplate::new(template_root, packages.to_path_buf());
        let world = OicanaWorld::new(files, TemplateInputs::new(), manifest)?;

        Ok(Template { world })
    }
}
