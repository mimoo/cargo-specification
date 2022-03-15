use askama::Template;
use comrak::{
    markdown_to_html, ComrakExtensionOptions, ComrakOptions, ComrakParseOptions,
    ComrakRenderOptions,
};
use std::{fs::File, io::Write as IOWrite, path::PathBuf};

use crate::toml_parser::Specification;

#[derive(Template)]
#[template(path = "respec.html", escape = "none")]
struct Respec {
    name: String,
    editors: Vec<(String, String)>,
    github: String,
    short_name: String,
    description: String,
    content: String,
}

pub fn build(specification: &Specification, content: &str, output_file: Option<PathBuf>) {
    let output_file = output_file.unwrap_or_else(|| PathBuf::from("specification.html"));

    //~ - converts markdown content to pure HTML
    let content = markdown_to_html(
        content,
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

    //~ - produces the HTML output
    let html_page = Respec {
        name: specification.metadata.name.clone(),
        editors: specification
            .metadata
            .authors
            .iter()
            .map(|author| (author.clone(), "".to_string()))
            .collect(),
        github: "".to_string(),
        short_name: "".to_string(),
        description: specification
            .metadata
            .description
            .as_deref()
            .unwrap_or("")
            .to_string(),
        content,
    };

    let mut file = File::create(&output_file).unwrap_or_else(|e| panic!("{}", e));
    let _ = write!(&mut file, "{}", html_page.render().unwrap()).unwrap();
    println!("\n=> html output saved at {}", output_file.display());
}
