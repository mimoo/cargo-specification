use clap::ArgEnum;
use miette::{IntoDiagnostic, Result, WrapErr};
use std::{
    collections::HashSet,
    fs::{self},
    path::PathBuf,
};
use tinytemplate::TinyTemplate;

use crate::{comment_parser, errors::SpecError, formats, git::get_local_repo_path, toml_parser};

/// The different specification format that cargo-spec can output
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum OutputFormat {
    /// Markdown (the default)
    Markdown,

    /// Respec
    Respec,
}

/// Builds the specification and returns a number of files to watch
pub fn build(
    toml_spec: PathBuf,
    output_file: Option<PathBuf>,
    output_format: OutputFormat,
) -> Result<HashSet<PathBuf>> {
    let mut files_to_watch = HashSet::new();

    //~ 1. parse the specification file with the [toml_parser](#toml-parser)
    let mut specification = toml_parser::parse_toml_spec(toml_spec.as_path())?;

    let mut spec_dir =
        fs::canonicalize(&toml_spec).expect("couldn't canonicalize the specification path");
    spec_dir.pop();

    //~ 2. retrieve the template file
    let mut template_path = spec_dir.clone();
    template_path.push(&specification.config.template);
    files_to_watch.insert(template_path.clone());

    let template = fs::read_to_string(&template_path)
        .into_diagnostic()
        .wrap_err_with(|| format!("could not read template {}", template_path.display(),))?;

    //~ 3. extract the spec comments from all the files listed using [comment_parser](#comment-parser)
    let base = get_local_repo_path();
    for filename in specification.sections.values_mut() {
        let path = if matches!(filename.chars().next(), Some('@')) {
            let base = base
                .as_ref()
                .ok_or(SpecError::NotGitRepo(filename.clone()))
                .into_diagnostic()?;
            let base = base.trim();
            // TODO: this will panic if we just wrote @ and not @/
            let filename = filename.split_at(2).1.to_string();
            PathBuf::from(base).join(filename)
        } else {
            let mut path = spec_dir.clone();
            path.push(&filename);
            path
        };
        files_to_watch.insert(path.clone());

        *filename = comment_parser::parse_file(&path)?;
    }

    //~ 4. render the template
    let mut tt = TinyTemplate::new();
    tt.set_default_formatter(&tinytemplate::format_unescaped);
    tt.add_template("specification", &template)
        .into_diagnostic()
        .wrap_err_with(|| format!("can't parse template {}", template_path.display(),))?;

    let rendered = tt
        .render("specification", &specification)
        .into_diagnostic()
        .wrap_err_with(|| {
            format!(
                "template file can't be rendered: {}",
                template_path.display()
            )
        })?;

    //~ 5. build the spec. We currently support two different formats:
    use OutputFormat::*;
    match output_format {
        //~     - [markdown](https://daringfireball.net/projects/markdown/)
        Markdown => formats::markdown::build(&rendered, output_file),
        //~     - [respec](https://github.com/w3c/respec/)
        Respec => {
            formats::respec::build(&specification, &rendered, output_file);
        }
    };

    // return a number of files to watch (useful for the [watch] function)
    Ok(files_to_watch)
}

pub fn watch(toml_spec: PathBuf, output_format: OutputFormat, output_file: Option<PathBuf>) {
    use notify::{watcher, RecursiveMode, Watcher};
    use std::sync::mpsc::channel;
    use std::time::Duration;

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // Create a watcher object, delivering debounced events.
    // The notification back-end is selected based on the platform.
    let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();
    watcher
        .watch(toml_spec.clone(), RecursiveMode::NonRecursive)
        .unwrap_or_else(|_e| {
            panic!(
                "could not watch specification file: {}",
                toml_spec.display()
            )
        });

    let mut files_to_watch = HashSet::new();

    loop {
        // build and get files to watch
        match build(toml_spec.clone(), output_file.clone(), output_format) {
            Err(e) => println!("error: {}", e),
            Ok(new_files_to_watch) => {
                // watch any new files contained in the specification
                for file in new_files_to_watch.difference(&files_to_watch) {
                    watcher
                        .watch(&file, RecursiveMode::NonRecursive)
                        .unwrap_or_else(|_e| {
                            panic!("could not find file to watch {}", file.display())
                        });
                }

                // unwatch files that are not in the specification
                for file in files_to_watch.difference(&new_files_to_watch) {
                    watcher.unwatch(&file).unwrap_or_else(|_e| {
                        panic!("could not find file to watch {}", file.display())
                    });
                }

                files_to_watch = new_files_to_watch;
            }
        };

        match rx.recv() {
            Ok(event) => println!("{:?}", event),
            Err(e) => panic!("watch error: {:?}", e),
        }
    }
}
