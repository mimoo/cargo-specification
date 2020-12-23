#![feature(str_split_once)]

mod comment_parser;

fn main() {
    let files = vec!["../src/data_structures.rs"];

    for file in files {
        let res = comment_parser::parse_file(file);
        println!("{}", res);
    }
}
