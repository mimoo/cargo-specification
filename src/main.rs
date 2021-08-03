use askama::Template;
use clap::{App, Arg};
use comrak::{
    markdown_to_html, ComrakExtensionOptions, ComrakOptions, ComrakParseOptions,
    ComrakRenderOptions,
};
use std::{
    fmt::Write as FmtWrite,
    fs::{self, File},
    io::Write as IOWrite,
    path::PathBuf,
};

//~ ## Cargo-specification

mod comment_parser;
mod git;
mod toml_parser;

#[derive(Template)]
#[template(path = "specification.html", escape = "none")]
struct HtmlSpecification {
    name: String,
    editors: Vec<(String, String)>,
    github: String,
    short_name: String,
    description: String,
    content: String,
}

//~ The main algorithm:
fn main() {
    //~ * parse arguments
    let matches = App::new("cargo-specification")
        .version("1.0")
        .author("David W. <davidwg@fb.com>")
        .about("The code is the spec")
        .arg(
            Arg::with_name("specification-path")
                .help("Sets the path to the required Specification.toml")
                .short("s")
                .long("specification-path")
                .default_value("./Specification.toml")
                .takes_value(true)
                .value_name("PATH"),
        )
        .arg(
            Arg::with_name("delimiter")
                .help("Sets the marker that Cargo-specification will recognize, default is //~")
                .short("d")
                .long("delimiter")
                .default_value("//~")
                .takes_value(true)
                .value_name("PATH"),
        )
        .arg(
            Arg::with_name("html-output")
                .help("prints the output as HTML")
                .short("o")
                .long("html-output")
                .default_value("./specification.html")
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
    let delimiter = matches
        .value_of("delimiter")
        .expect("must use --delimiter option");

    //~ * parse the Specification.toml file
    let specification = toml_parser::parse_toml_spec(toml_spec);
    println!("{:?}", specification);

    //~ * get dir of specification file
    let spec_dir = PathBuf::from(toml_spec);
    let mut spec_dir = fs::canonicalize(&spec_dir).unwrap();
    spec_dir.pop();
    println!("{:?}", spec_dir);

    //~ * flatten the sections
    let files: Vec<&String> = specification.sections.values().flatten().collect();

    //~ * retrieve the content from all the files
    let mut content = String::new();
    for file in files {
        let mut path = spec_dir.clone();
        path.push(file);
        let res = comment_parser::parse_file(delimiter, path.to_str().unwrap());
        writeln!(&mut content, "{}", res).unwrap();
    }

    //~ * markdown -> HTML
    let content = markdown_to_html(
        &content,
        &ComrakOptions {
            extension: ComrakExtensionOptions {
                strikethrough: true,
                tagfilter: true,
                table: true,
                autolink: true,
                tasklist: true,
                superscript: true,
                header_ids: None,
                footnotes: true,
                description_lists: true,
                front_matter_delimiter: None,
            },
            parse: ComrakParseOptions {
                smart: true,
                default_info_string: None,
            },
            render: ComrakRenderOptions {
                hardbreaks: false,
                github_pre_lang: true,
                width: 0,
                unsafe_: true, // it's our spec afterall
                escape: false,
            },
        },
    );

    //~ * html output
    let html_page = HtmlSpecification {
        name: specification.metadata.name,
        editors: specification
            .metadata
            .authors
            .into_iter()
            .map(|author| (author, "".to_string()))
            .collect(),
        github: "".to_string(),
        short_name: "".to_string(),
        description: specification.metadata.description,
        content: content,
    };

    let mut file = File::create(html_output).unwrap_or_else(|e| panic!("{}", e));
    let _ = write!(&mut file, "{}", html_page.render().unwrap()).unwrap();
    println!("\n=> html output saved at {}", html_output);
}
