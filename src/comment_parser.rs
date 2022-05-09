use std::fmt::Write as FmtWrite;
use std::path::Path;

use miette::{IntoDiagnostic, NamedSource, Result, WrapErr};

use crate::errors::SpecError;

/// The prefix to any spec instructions
const SPECIFICATION_INSTRUCTION: &str = "spec:";

/// Parse a file and return the specification-related content
pub fn parse_file(file_name: &Path) -> Result<String> {
    //~ parsing is based on the extension of the file:
    match Path::new(file_name)
        .extension()
        .ok_or_else(|| SpecError::CantParseFile(file_name.to_path_buf()))?
        .to_str()
        .expect("couldn't convert the extension to a string")
    {
        //~ - for markdown files, we retrieve the entire content
        "md" => std::fs::read_to_string(file_name)
            .into_diagnostic()
            .wrap_err_with(|| format!("could not read file {}", file_name.display())),

        //~ - for python files we look for comments starting with `#~`
        "py" => parse_code("python", "#~", None, file_name),

        //~ - for ML files we look for comments starting with `#~`
        "ml" | "mli" => parse_code("ocaml", "(*~", Some("*)"), file_name),

        //~ - for other files we look for comments starting with `//~`
        ext => parse_code(ext, "//~", None, file_name),
    }
}

/// detects if a comment ends on this same line
fn has_end(end: &str, comment: &str) -> bool {
    comment.trim().ends_with(end)
}

//~
//~ for each file listed by the specification manifest, we follow these steps:
//~

/// Parse code to return the specification-related content
/// (comments that start with a special delimiter, by default `~`)
pub fn parse_code(
    lang: &str,
    start_comment: &str,
    end_comment: Option<&str>,
    file_name: &Path,
) -> Result<String> {
    // set to the offset of the startcode if we're waiting for an endcode instruction
    let mut extract_code = None;

    // set to the indentation of the 1st line if we're within a multi-line in a comment
    let mut in_spec_comment = None;

    // to store the result of extracting doc comments
    let mut result = String::new();

    // read file
    let source = std::fs::read_to_string(file_name)
        .into_diagnostic()
        .wrap_err_with(|| format!("could not read file {}", file_name.display()))?;

    // go over the file line by line
    let mut byte_offset_for_errors = 0;
    for line in source.lines() {
        //~ 1. only print a normal line if it is between `//~ spec:startcode` and `//~spec:endcode` statements
        if !line.trim_start().starts_with(start_comment) && in_spec_comment.is_none() {
            if extract_code.is_some() {
                // TODO: reset indentation
                writeln!(&mut result, "{}", line).unwrap();
            }

            byte_offset_for_errors += line.len() + 1; // +1 for the newline character
            continue;
        }

        //~ 2. if we are within a multi-line comment, we remove the indentation
        //~   based on the indentation of the first line of the comment
        let comment = if let Some(indentation) = in_spec_comment {
            let left_trimmed = line.trim_start();
            let whitespaces_len = line.len() - left_trimmed.len();
            if indentation > whitespaces_len {
                left_trimmed
            } else {
                &line[indentation..]
            }
        } else {
            //~ 3. otherwise, we extract what comes after the comment delimiter
            //~   (note that the result might still have a starting space)
            line.split_once(start_comment).unwrap().1
        };

        //~ 4. lines starting with `//~ spec:` are specific instructions:
        if in_spec_comment.is_none() && comment.trim().starts_with(SPECIFICATION_INSTRUCTION) {
            let instruction = comment
                // get part after spec:
                .split_once(SPECIFICATION_INSTRUCTION)
                .unwrap()
                .1
                // remove anything after the instruction
                .split(' ')
                .next()
                .unwrap();

            match instruction {
                //~~ - a comment starting with `//~ spec:startcode` will print
                //~       every line afterwards, up until a `//~ spec:endcode` statement
                "startcode" if extract_code.is_none() => {
                    let column = line.find("startcode").unwrap();
                    writeln!(&mut result, "```{lang}").unwrap();
                    extract_code = Some(byte_offset_for_errors + column);
                }
                "startcode" if extract_code.is_some() => {
                    let column = line.find("startcode").unwrap();
                    return Err(SpecError::DoubleStartcode {
                        src: NamedSource::new(file_name.to_string_lossy(), source.to_string()),
                        bad_bit: (byte_offset_for_errors + column, "startcode".len()),
                    })
                    .into_diagnostic();
                }
                // spec:endcode ends spec:startcode
                "endcode" if extract_code.is_some() => {
                    writeln!(&mut result, "```").unwrap();
                    extract_code = None;
                }
                "endcode" if extract_code.is_none() => {
                    let column = line.find("endcode").unwrap();
                    return Err(SpecError::MissingStartcode {
                        src: NamedSource::new(file_name.to_string_lossy(), source.to_string()),
                        bad_bit: (byte_offset_for_errors + column, "endcode".len()),
                    })
                    .into_diagnostic();
                }
                //~~ - error on any other instructions
                _ => {
                    let column = line.find("spec:").unwrap();
                    let instruction = line.split_once("spec:").unwrap().1;
                    Err(SpecError::BadInstruction {
                        src: NamedSource::new(file_name.to_string_lossy(), source.to_string()),
                        bad_bit: (byte_offset_for_errors + column, 0),
                    })
                    .wrap_err_with(|| format!("the instruction you gave: {instruction}"))?;
                }
            };
        } else {
            //~ 5. if we are not seeing an instruction, figure out if:
            let comment = if let Some(end) = end_comment {
                if has_end(end, comment) {
                    //~~ - the comment is ending

                    in_spec_comment = None;
                    comment.trim_end_matches(end)
                } else {
                    //~~ - or goes on to the next line

                    if in_spec_comment.is_none() {
                        let offset = line.find(start_comment).unwrap() + start_comment.len();
                        in_spec_comment = Some(offset);
                    }

                    comment
                }
            } else {
                comment
            };

            //~ 6. Finally, extract the specification text.
            //~    Each `~` at the start of the comment,
            //~    not including the first one,
            //~    is converted to a tab.
            //~    This allows us to write `//~~ *` for nested list items.
            let no_more_tilde = comment.trim_start_matches("~");

            let mut indented = {
                let indentation = comment.len() - no_more_tilde.len();
                let indentation: Vec<_> = (0..indentation).map(|_| "\t").collect();
                indentation.join("")
            };

            let comment = no_more_tilde.strip_prefix(' ').unwrap_or(comment);
            indented.push_str(comment);

            writeln!(&mut result, "{indented}").unwrap();
        }

        byte_offset_for_errors += line.len() + 1; // +1 for the newline character
    }

    //~ 7. at the end, make sure that every startcode instruction
    //~    is matched with a endcode instruction
    if let Some(offset) = extract_code {
        return Err(SpecError::MissingEndcode {
            src: NamedSource::new(file_name.to_string_lossy(), source.to_string()),
            bad_bit: (offset, 0),
        })
        .into_diagnostic();
    }

    //~ 8. return the result
    Ok(result)
}
