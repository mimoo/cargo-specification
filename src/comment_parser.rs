use std::fmt::Write as FmtWrite;
use std::path::Path;

use miette::{IntoDiagnostic, NamedSource, Result, WrapErr};

use crate::errors::SpecError;

const SPECIFICATION_INSTRUCTION: &str = "spec:";

/// Parse a file and return the specification-related content
pub fn parse_file(file_name: &Path) -> Result<String> {
    //~ parsing is based on the extension of the file:
    match Path::new(file_name)
        .extension()
        .ok_or(SpecError::CantParseFile(file_name.to_path_buf()))?
        .to_str()
        .expect("couldn't convert the extension to a string")
    {
        //~ - for markdown files, we retrieve the entire content
        "md" => std::fs::read_to_string(file_name)
            .into_diagnostic()
            .wrap_err_with(|| format!("could not read file {}", file_name.display())),

        //~ - for python files we look for comments starting with `#~`
        "py" => parse_code("#~", None, file_name),

        //~ - for python files we look for comments starting with `#~`
        "ml" | "mli" => parse_code("(*~", Some("*)"), file_name),

        //~ - for other files we look for comments starting with `//~`
        _ => parse_code("//~", None, file_name),
    }
}

/// detects if a comment ends on this same line
fn has_end(end: &str, comment: &str) -> bool {
    comment.trim().ends_with(end)
}

/// Parse code to return the specification-related content
/// (comments that start with a special delimiter, by default `~`)
pub fn parse_code(
    start_comment: &str,
    end_comment: Option<&str>,
    file_name: &Path,
) -> Result<String> {
    let mut extract_code = None; // set if we're waiting for an encode instruction
    let mut in_spec_comment = false; // set if we're within a multi-line in a comment
    let mut result = String::new();

    // read file
    let source = std::fs::read_to_string(file_name)
        .into_diagnostic()
        .wrap_err_with(|| format!("could not read file {}", file_name.display()))?;

    // go over file line by line
    let mut byte_offset_for_errors = 0;
    for line in source.lines() {
        // if this is a normal line...
        if !line.trim_start().starts_with(start_comment) && !in_spec_comment {
            // only print a normal line if it is between `//~ spec:startcode` and `//~spec:endcode` statements
            if extract_code.is_some() {
                // TODO: reset indentation
                writeln!(&mut result, "{}", line).unwrap();
            }

            byte_offset_for_errors += line.len() + 1; // +1 for the newline character
            continue;
        }

        //~ detect spec comment
        let comment = line.split_once(start_comment).unwrap().1;

        //~ lines starting with `//~ spec:instruction` are specific instructions
        if !in_spec_comment && comment.trim().starts_with(SPECIFICATION_INSTRUCTION) {
            match comment.split_once(SPECIFICATION_INSTRUCTION).unwrap().1 {
                //~ a comment starting with `//~ spec:startcode` will print
                //~ every line afterwards, up until a `//~ spec:endcode` statement
                "startcode" if extract_code.is_none() => {
                    let column = line.find("startcode").unwrap();
                    writeln!(&mut result, "```rust").unwrap();
                    extract_code = Some(byte_offset_for_errors + column);
                }
                "startcode" if extract_code.is_some() => {
                    let column = line.find("startcode").unwrap();
                    Err(SpecError::DoubleStartcode {
                        src: NamedSource::new(file_name.to_string_lossy(), source.to_string()),
                        bad_bit: (byte_offset_for_errors + column, "startcode".len()),
                    })?;
                }
                // spec:endcode ends spec:startcode
                "endcode" if extract_code.is_some() => {
                    writeln!(&mut result, "```").unwrap();
                    extract_code = None;
                }
                "endcode" if extract_code.is_none() => {
                    let column = line.find("endcode").unwrap();
                    Err(SpecError::MissingStartcode {
                        src: NamedSource::new(file_name.to_string_lossy(), source.to_string()),
                        bad_bit: (byte_offset_for_errors + column, "endcode".len()),
                    })?;
                }
                //
                _ => {
                    let column = line.find("spec:").unwrap();
                    Err(SpecError::BadInstruction {
                        src: NamedSource::new(file_name.to_string_lossy(), source.to_string()),
                        bad_bit: (byte_offset_for_errors + column, 0),
                    })?;
                }
            };
        } else {
            // extract the specification text
            let comment = if let Some(end) = end_comment {
                if has_end(end, comment) {
                    // either the comment is ending
                    in_spec_comment = false;
                    comment.trim_end_matches(end)
                } else {
                    // or it goes on to the next line
                    in_spec_comment = true;
                    comment
                }
            } else {
                comment
            };

            let comment = comment.strip_prefix(' ').unwrap_or(comment);
            writeln!(&mut result, "{}", comment).unwrap();
        }

        byte_offset_for_errors += line.len() + 1; // +1 for the newline character
    }

    // check state is consistent
    if let Some(offset) = extract_code {
        Err(SpecError::MissingEndcode {
            src: NamedSource::new(file_name.to_string_lossy(), source.to_string()),
            bad_bit: (offset, 0),
        })?;
    }

    // return the result
    Ok(result)
}
