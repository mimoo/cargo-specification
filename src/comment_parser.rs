use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

const SPECIFICATION_COMMENT: &str = "//~ ";
const SPECIFICATION_INSTRUCTION: &str = "spec:";

/// Parse a file and return the file specification
pub(crate) fn parse_file(file_name: &str) -> String {
    // state
    let mut print_line = false; // indicates if we're between `//~ spec:startcode` and `//~spec:endcode` statements
    let mut result = String::new();

    // go over file line by line
    let file = File::open(file_name).unwrap_or_else(|e| panic!("{}", e));
    let lines = BufReader::new(file).lines();
    for line in lines {
        let line = line.unwrap();

        if !line.trim().starts_with(SPECIFICATION_COMMENT) {
            // only print a normal line if it is between `//~ spec:startcode` and `//~spec:endcode` statements
            // TODO: reset indentation
            if print_line {
                writeln!(&mut result, "{}", line);
            }
            continue;
        }

        // if the line starts with //~ parse it
        let comment = line.split_once(SPECIFICATION_COMMENT).unwrap().1;
        if comment.starts_with(SPECIFICATION_INSTRUCTION) {
            // match on the instruction given in `//~ spec:instruction`
            match comment.split_once(SPECIFICATION_INSTRUCTION).unwrap().1 {
                // spec:startcode will print every line afterwards, up until a spec:endcode statement
                "startcode" if !print_line => print_line = true,
                "startcode" if print_line => panic!("cannot startcode when already started"),
                // spec:endcode ends spec:startcode
                "endcode" if print_line => print_line = false,
                "endcode" if !print_line => {
                    panic!("cannot endcode if haven't startcode before")
                }
                //
                _ => unimplemented!(),
            };
        } else {
            // if the specification comment is not an instruction, save it
            writeln!(&mut result, "{}", comment);
        }
    }

    //
    result
}
