use build::{build, watch, OutputFormat};
use clap::{Args, Parser, Subcommand};
use init::{init, new};
use miette::Result;
use std::path::PathBuf;

mod build;
mod comment_parser;
mod errors;
mod formats;
mod git;
mod init;
mod toml_parser;

/// To make cargo subcommands work, you need to use `bin_name`,
/// as well as a subcommand.
#[derive(Debug, Parser)]
#[clap(bin_name = "cargo")]
#[clap(author, version, about)]
enum Cli {
    #[clap(subcommand)]
    Spec(Spec),
}

/// The struct that represents the default command of `cargo spec`
#[derive(Debug, Subcommand)]
enum Spec {
    /// Creates the necessary files to start a specification in a folder
    New { name: String },

    /// Creates the necessary files to start a specification in an existing directory
    Init {
        #[clap(parse(from_os_str), value_name = "SPEC_DIR")]
        path: PathBuf,
    },

    /// Create the specification file at the given path.
    Build(Opt),

    /// Watches any listed files in the specification toml file and
    /// re-create the specification on any changes.
    Watch(Opt),
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
}

fn main() -> Result<()> {
    //~ 1. parse command-line arguments
    let Cli::Spec(args) = Cli::parse();

    //~ 2. depending on the mode:
    match args {
        Spec::New { name } => new(name)?,
        Spec::Init { path } => init(None, path)?,
        //~   a. the `Build` mode builds the specification
        Spec::Build(Opt {
            specification_path,
            output_file,
            output_format,
        }) => {
            let toml_spec =
                specification_path.unwrap_or_else(|| PathBuf::from("Specification.toml"));
            let output_format = output_format.unwrap_or(OutputFormat::Markdown);

            let _ = build(toml_spec, output_file, output_format)?;
        }
        //~   b. the `Watch` mode builds the specification on every change
        Spec::Watch(Opt {
            specification_path,
            output_file,
            output_format,
        }) => {
            let toml_spec =
                specification_path.unwrap_or_else(|| PathBuf::from("Specification.toml"));
            let output_format = output_format.unwrap_or(OutputFormat::Markdown);

            watch(toml_spec, output_format, output_file);
        }
    };

    Ok(())
}
