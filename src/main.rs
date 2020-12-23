#![feature(str_split_once)]

mod comment_parser;
mod toml_parser;

fn main() {
    let specification = toml_parser::parse_toml_spec("../Specification.toml");
    println!("{:?}", specification);

    let files = vec!["../src/data_structures.rs"];

    for file in files {
        let res = comment_parser::parse_file(file);
        println!("{}", res);
    }
}
