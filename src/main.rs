use clap::{ArgEnum, Args, Parser, Subcommand};
use miette::{IntoDiagnostic, Result, WrapErr};
use std::{
    collections::HashSet,
    fs::{self},
    path::PathBuf,
};
use tinytemplate::TinyTemplate;

mod comment_parser;
mod errors;
mod formats;
mod git;
mod toml_parser;

/// The different specification format that cargo-spec can output
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]

enum OutputFormat {
    /// Markdown (the default)
    Markdown,

    /// Respec
    Respec,
}

/// To make cargo subcommands work, you need to use `bin_name`,
/// as well as a subcommand.
#[derive(Debug, Parser)]
#[clap(bin_name = "cargo")]
enum Cli {
    #[clap(author, version, about)]
    Spec(Opt),
}

/// The different options that can be passed to this CLI
#[derive(Args, Debug)]
#[clap(author, version, about, bin_name = "cargo")]
struct Opt {
    /// The path to the specification toml file (defaults to Specification.toml).
    #[clap(short, long, parse(from_os_str), value_name = "SPEC_PATH")]
    specification_path: Option<PathBuf>,

    /// The path to the specification file to write
    /// (defaults to specification.md or specification.html)
    #[clap(short, long, parse(from_os_str), value_name = "OUTPUT_FILE")]
    output_file: Option<PathBuf>,

    /// The output format (defaults to markdown)
    #[clap(short = 'f', long, value_name = "OUTPUT_FORMAT")]
    #[clap(arg_enum)]
    output_format: Option<OutputFormat>,

    #[clap(subcommand)]
    mode: Option<Mode>,
}

/// There are several subcommands:return an error if it doesn't
#[derive(Debug, Clone, Copy, Subcommand)]
enum Mode {
    /// Create the specification file at the given path.
    Build,

    /// Watches any listed files in the specification toml file and
    /// re-create the specification on any changes.
    Watch,

    /// Useful for CI: makes sure that the generated specification
    /// matches the given path, otherwise returns an error.
    CI,
}

fn main() -> Result<()> {
    //~ 1. parse command-line arguments
    let Cli::Spec(args) = Cli::parse();

    let toml_spec = args
        .specification_path
        .to_owned()
        .unwrap_or(PathBuf::from("Specification.toml"));
    let output_format = args.output_format.unwrap_or(OutputFormat::Markdown);
    let output_file = args.output_file.to_owned();
    let mode = args.mode.unwrap_or(Mode::Build);

    //    println!("args: {:?}", args);

    //~ 2. depending on the mode:
    use Mode::*;
    match mode {
        //~   a. the `Build` mode builds the specification
        Build => {
            let _ = build(toml_spec.clone(), output_file.clone(), output_format)?;
        }
        //~   b. the `Watch` mode builds the specification on every change
        Watch => {
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
        //~   c. the CI mode builds the specification and errors out if it doesn't match the given output path
        //~      this is useful in CI to make sure that the latest specification
        //~      has been pushed to the repository
        CI => {
            todo!();

            // create tmp dir

            // build in tmp dir

            // check if what is built is the same as the result file, if not return an error (how to do exit(1) ?)
        }
    };

    Ok(())
}

/// Builds the specification and returns a number of files to watch
fn build(
    toml_spec: PathBuf,
    output_file: Option<PathBuf>,
    output_format: OutputFormat,
) -> Result<HashSet<PathBuf>> {
    let mut files_to_watch = HashSet::new();

    //~ 3. parse the Specification.toml file
    let mut specification = toml_parser::parse_toml_spec(toml_spec.as_path())?;
    //    println!("specification: {:#?}", specification);

    let mut spec_dir =
        fs::canonicalize(&toml_spec).expect("couldn't canonicalize the specification path");
    spec_dir.pop();

    //~ 4. retrieve the template
    let mut template_path = spec_dir.clone();
    template_path.push(&specification.config.template);
    files_to_watch.insert(template_path.clone());

    let template = fs::read_to_string(&template_path)
        .into_diagnostic()
        .wrap_err_with(|| format!("could not read template {}", template_path.display(),))?;

    //~ 5. retrieve the content from all the files listed in the .toml
    for (_, filename) in &mut specification.sections {
        let mut path = spec_dir.clone();
        path.push(&filename);
        files_to_watch.insert(path.clone());

        *filename = comment_parser::parse_file(&path)?;
    }

    //~ 6. render the template
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

    //~ 7. build the spec
    use OutputFormat::*;
    match output_format {
        Markdown => formats::markdown::build(&specification, &rendered, output_file),
        Respec => {
            formats::respec::build(&specification, &rendered, output_file);
        }
    };

    // return a number of files to watch
    return Ok(files_to_watch);
}
