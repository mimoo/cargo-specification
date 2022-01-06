use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

const SPECIFICATION_INSTRUCTION: &str = "spec:";

/// Parse a file and return the specification-related content
pub fn parse_file(file_name: &str) -> String {
    //~ parsing is based on the extension of the file:
    match Path::new(file_name)
        .extension()
        .expect("cargo-specification can only parse files that have an extension")
        .to_str()
        .expect("couldn't convert the extension to a string")
    {
        //~ - for markdown files, we retrieve the entire content
        "md" => {
            std::fs::read_to_string(file_name).unwrap_or_else(|e| panic!("{}: {}", e, file_name))
        }

        //~ - for python files we look for comments starting with `#~`
        "py" => parse_code("#~", file_name),

        //~ - for other files we look for comments starting with `//~`
        _ => parse_code("//~", file_name),
    }
}

/// Parse code to return the specification-related content
/// (comments that start with a special delimiter, by default `~`)
pub fn parse_code(delimiter: &str, file_name: &str) -> String {
    // state
    let mut extract_code = false; // indicates if we're between `//~ spec:startcode` and `//~spec:endcode` statements
    let mut result = String::new();

    // go over file line by line
    let file = File::open(file_name).unwrap_or_else(|e| panic!("{}: {}", e, file_name));
    let lines = BufReader::new(file).lines();
    for (line_number, line) in lines.enumerate() {
        let line = line.unwrap();

        // if this is a normal line...
        if !line.trim_start().starts_with(delimiter) {
            // only print a normal line if it is between `//~ spec:startcode` and `//~spec:endcode` statements
            if extract_code {
                // TODO: reset indentation
                writeln!(&mut result, "{}", line).unwrap();
            }

            continue;
        }

        // if the line starts with //~ parse it
        let comment = line.split_once(delimiter).unwrap().1;

        //~ lines starting with `//~ spec:instruction` are specific instructions
        if comment.trim().starts_with(SPECIFICATION_INSTRUCTION) {
            match comment.split_once(SPECIFICATION_INSTRUCTION).unwrap().1 {
                //~ a comment starting with `//~ spec:startcode` will print
                //~ every line afterwards, up until a `//~ spec:endcode` statement
                "startcode" if !extract_code => {
                    writeln!(&mut result, "```rust").unwrap();
                    extract_code = true;
                }
                "startcode" if extract_code => panic!("cannot startcode when already started"),
                // spec:endcode ends spec:startcode
                "endcode" if extract_code => {
                    writeln!(&mut result, "```").unwrap();
                    extract_code = false;
                }
                "endcode" if !extract_code => {
                    panic!("cannot endcode if haven't startcode before")
                }
                //
                _ => panic!(
                    "instruction not recognized in {}:{}\n instruction: {}",
                    file_name, line_number, line
                ),
            };
        } else {
            // extract the specification text
            let comment = comment.strip_prefix(' ').unwrap_or(comment);
            writeln!(&mut result, "{}", comment).unwrap();
        }
    }

    // check state is consistent
    if extract_code {
        panic!("a //~ spec:startcode was left open ended");
    }

    // return the result
    result
}
