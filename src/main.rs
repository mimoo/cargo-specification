use clap::{AppSettings, ArgEnum, Parser, Subcommand};
use serde::Serialize;
use std::{
    collections::HashMap,
    fmt::Write as FmtWrite,
    fs::{self},
    path::PathBuf,
};
use tinytemplate::TinyTemplate;

mod comment_parser;
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

/// The different options that can be passed to this CLI
#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
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

fn main() {
    //~ 1. parse command-line arguments
    let args = Cli::parse();

    let toml_spec = args
        .specification_path
        .to_owned()
        .unwrap_or(PathBuf::from("Specification.toml"));
    let output_format = args.output_format.unwrap_or(OutputFormat::Markdown);
    let output_file = args.output_file.to_owned();
    let mode = args.mode.unwrap_or(Mode::Build);

    println!("args: {:?}", args);

    use Mode::*;
    match mode {
        Build => build(toml_spec, output_file, output_format),
        Watch => todo!(),
        CI => todo!(),
    };
}

fn build(toml_spec: PathBuf, output_file: Option<PathBuf>, output_format: OutputFormat) {
    //~ 2. parse the Specification.toml file
    let mut specification = toml_parser::parse_toml_spec(toml_spec.as_path());
    println!("specification: {:#?}", specification);

    let spec_dir = PathBuf::from(toml_spec);
    let mut spec_dir = fs::canonicalize(&spec_dir).unwrap();
    spec_dir.pop();

    //~ 3. retrieve the template
    let mut path = spec_dir.clone();
    path.push(&specification.config.template);
    let template = fs::read_to_string(&path).expect("could not read template file");

    //~ 4. retrieve the content from all the files listed in the .toml
    for (_, filename) in &mut specification.sections {
        let mut path = spec_dir.clone();
        path.push(&filename);
        *filename =
            comment_parser::parse_file(path.to_str().expect("couldn't convert path to string"));
    }

    //~ 5. render the template
    let mut tt = TinyTemplate::new();
    tt.add_template("specification", &template)
        .unwrap_or_else(|e| panic!("template file can't be parsed: {}", e));

    let rendered = tt
        .render("specification", &specification)
        .unwrap_or_else(|e| panic!("template file can't be rendered: {}", e));

    //~ 6. build the spec
    use OutputFormat::*;
    match output_format {
        Markdown => formats::markdown::build(&specification, &rendered, output_file),
        Respec => {
            formats::respec::build(&specification, &rendered, output_file);
        }
    }
}
