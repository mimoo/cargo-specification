use crate::{
    errors::SpecError,
    toml_parser::{Config, Metadata, Specification},
};
use miette::{IntoDiagnostic, Result, WrapErr};
use std::{
    collections::HashMap,
    env,
    fs::{create_dir, File},
    io::Write,
    path::PathBuf,
};

pub const DEFAULT_MANIFEST: &str = "Specification.toml";
pub const DEFAULT_TEMPLATE: &str = "specification_template.md";

pub fn new(name: String) -> Result<()> {
    let path = env::current_dir().into_diagnostic()?;
    init(Some(name), path)
}

pub fn init(name: Option<String>, path: PathBuf) -> Result<()> {
    // we extrapolate the name of the spec from the directory name
    let mut name = if let Some(name) = name {
        name
    } else {
        match path.file_name() {
            Some(dir_name) => dir_name.to_string_lossy().to_string(),
            None => {
                return Err(SpecError::BadPath(path)).into_diagnostic();
            }
        }
    };

    // if the directory doesn't exist, create it
    if !path.is_dir() {
        create_dir(&path).into_diagnostic().wrap_err_with(|| {
            format!(
                "cannot create the specification directory {}",
                path.display()
            )
        })?;
    } else {
        // otherwise make sure there isn't already a spec in there
        let read_dir = path.read_dir().into_diagnostic()?;

        for dir_entry in read_dir.flatten() {
            let spec_file_detected = dir_entry.file_name().to_string_lossy() == DEFAULT_MANIFEST;
            let template_file_detected =
                dir_entry.file_name().to_string_lossy() == DEFAULT_TEMPLATE;
            if spec_file_detected || template_file_detected {
                return Err(SpecError::SpecAlreadyExists(path)).into_diagnostic();
            }
        }
    }

    // create the files
    let manifest_path = path.join(DEFAULT_MANIFEST);
    let mut manifest_file = File::create(&manifest_path).into_diagnostic().wrap_err_with(|| format!("cannot create the specification file {}, make sure you pass a specification toml file via --specification-path", manifest_path.display()))?;

    let template_path = path.join(DEFAULT_TEMPLATE);
    let mut template_file = File::create(&template_path).into_diagnostic().wrap_err_with(|| format!("cannot create the specification template file {}, make sure you pass a specification toml file via --specification-path", template_path.display()))?;

    // fill the specification manifest
    let metadata = Metadata {
        name: name.clone(),
        description: Some("some description".to_string()),
        version: None,
        authors: vec!["your name".to_string()],
    };
    let config = Config {
        template: DEFAULT_TEMPLATE.to_string(),
    };
    let specification = Specification {
        metadata,
        config,
        sections: HashMap::new(),
    };

    let manifest_content =
        toml::to_vec(&specification).expect("couldn't serialize the default manifest");

    manifest_file
        .write_all(&manifest_content)
        .into_diagnostic()
        .wrap_err_with(|| {
            format!(
                "couldn't write to the specification file {}",
                manifest_path.display()
            )
        })?;

    // create the template file
    let title = {
        let first_char = name.remove(0);
        let first_char = first_char.to_uppercase();
        format!("{first_char}{name}")
    };
    let template_content = format!("# {title}\n\n My specification\n");

    template_file
        .write_all(template_content.as_bytes())
        .into_diagnostic()
        .wrap_err_with(|| {
            format!(
                "couldn't write to the specification template file {}",
                template_path.display()
            )
        })?;

    Ok(())
}
