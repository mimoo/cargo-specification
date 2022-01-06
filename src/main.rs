use clap::{App, Arg};
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

fn main() {
    //~ 1. parse command-line arguments
    let matches = App::new("cargo-specification")
        .version("1.0")
        .author("David W. <davidwg@fb.com>")
        .about("The code is the spec")
        .arg(
            Arg::new("specification-path")
                .help("Sets the path to the required Specification.toml")
                .short('s')
                .long("specification-path")
                .default_value("./Specification.toml")
                .takes_value(true)
                .value_name("PATH"),
        )
        // TODO: move this in the config
        .arg(
            Arg::new("delimiter")
                .help("Sets the marker that Cargo-specification will recognize, default is //~")
                .short('d')
                .long("delimiter")
                .default_value("//~")
                .takes_value(true)
                .value_name("PATH"),
        )
        .arg(
            Arg::new("output-file")
                .help("destination file for the generated specification")
                .short('o')
                .long("output-file")
                .takes_value(true),
        )
        .arg(
            Arg::new("output-format")
                .help("the format of the specification (respec, markdown, rfc, mdbook, zkdocs, gitbook, etc.)")
                .short('f')
                .long("output-format")
                .default_value("markdown")
                .takes_value(true),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .help("suppress any output to stdout"),
        )
        // `cargo install cargo-specification` won't work without this
        .arg(Arg::new("catch-cargo-cli-bug"))
        .get_matches();

    let toml_spec = matches
        .value_of("specification-path")
        .expect("must use --specification-path option");
    let delimiter = matches
        .value_of("delimiter")
        .expect("must use --delimiter option");
    let output_file = matches.value_of("output-file");
    let spec_format = matches
        .value_of("output-format")
        .expect("must use --output-format option");

    //~ 2. parse the Specification.toml file
    let mut specification = toml_parser::parse_toml_spec(toml_spec);
    println!("{:#?}", specification);

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
        *filename = comment_parser::parse_file(
            delimiter,
            path.to_str().expect("couldn't convert path to string"),
        );
    }

    //~ 5. render the template
    let mut tt = TinyTemplate::new();
    tt.add_template("specification", &template)
        .unwrap_or_else(|e| panic!("template file can't be parsed: {}", e));

    let rendered = tt
        .render("specification", &specification)
        .unwrap_or_else(|e| panic!("template file can't be rendered: {}", e));

    //~ 6. build the spec
    match spec_format {
        "respec" => {
            formats::respec::build(&specification, &rendered, output_file);
        }
        "markdown" => formats::markdown::build(&specification, &rendered, output_file),
        x => {
            panic!("spec format {} not supported", x);
        }
    }
}
