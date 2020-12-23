#![feature(str_split_once)]

use askama::Template;
use clap::{App, Arg};
use comrak::{markdown_to_html, ComrakOptions};
use std::{
    fmt::Write as FmtWrite,
    fs::{self, File},
    io::Write as IOWrite,
    path::PathBuf,
};

mod comment_parser;
mod toml_parser;

#[derive(Template)]
#[template(path = "specification.html", escape = "none")]
struct HtmlSpecification {
    name: String,
    editors: Vec<(String, String)>,
    github: String,
    shortName: String,
    description: String,
    content: String,
}

fn main() {
    // parse arguments
    let matches = App::new("cargo-specification")
        .version("1.0")
        .author("David W. <davidwg@fb.com>")
        .about("The code is the spec")
        .arg(
            Arg::with_name("specification-path")
                .help("Sets the path to the required Specification.toml")
                .short("s")
                .long("specification-path")
                .takes_value(true)
                .value_name("PATH"),
        )
        .arg(
            Arg::with_name("html-output")
                .help("prints the output as HTML")
                .short("o")
                .long("html-output")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .help("suppress any output to stdout"),
        )
        // cargo install cargo-dephell won't work without this
        .arg(Arg::with_name("catch-cargo-cli-bug"))
        .get_matches();

    let html_output = matches
        .value_of("html-output")
        .expect("must use --html-output option");
    let toml_spec = matches
        .value_of("specification-path")
        .expect("must use --specification-path option");

    // parse the Specification.toml file
    let specification = toml_parser::parse_toml_spec(toml_spec);
    println!("{:?}", specification);

    // get dir of specification file
    let spec_dir = PathBuf::from(toml_spec);
    let mut spec_dir = fs::canonicalize(&spec_dir).unwrap();
    spec_dir.pop();
    println!("{:?}", spec_dir);

    // flatten the sections
    let files: Vec<&String> = specification.sections.values().flatten().collect();

    // retrieve the content from all the files
    let mut content = String::new();
    for file in files {
        let mut path = spec_dir.clone();
        path.push(file);
        let res = comment_parser::parse_file(path.to_str().unwrap());
        writeln!(&mut content, "{}", res).unwrap();
    }

    // markdown -> HTML
    let content = markdown_to_html(&content, &ComrakOptions::default());

    // html output
    let html_page = HtmlSpecification {
        name: specification.specification.name,
        editors: specification
            .specification
            .authors
            .into_iter()
            .map(|author| (author, "".to_string()))
            .collect(),
        github: "".to_string(),
        shortName: "".to_string(),
        description: specification.specification.description,
        content: content,
    };

    let mut file = File::create(html_output).unwrap_or_else(|e| panic!("{}", e));
    let _ = write!(&mut file, "{}", html_page.render().unwrap()).unwrap();
    println!("\n=> html output saved at {}", html_output);
}
