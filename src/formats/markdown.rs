use askama::Template;
use comrak::{
    markdown_to_html, ComrakExtensionOptions, ComrakOptions, ComrakParseOptions,
    ComrakRenderOptions,
};
use std::{
    fs::File,
    io::Write as IOWrite,
    path::{Path, PathBuf},
};

use crate::toml_parser::Specification;

pub fn build(specification: &Specification, content: &str, output_file: Option<PathBuf>) {
    let output_file = output_file.unwrap_or(PathBuf::from("specification.md"));
    let mut file = File::create(&output_file).unwrap_or_else(|e| panic!("{}", e));
    let _ = write!(&mut file, "{}", content).unwrap();
    println!("\n=> html output saved at {}", output_file.display());
}
