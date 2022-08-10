use std::path::PathBuf;

use miette::{Diagnostic, NamedSource};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum SpecError {
    #[error("A specification already exist at `{0}`")]
    #[diagnostic(help("the path you passed already has a specification"))]
    SpecAlreadyExists(PathBuf),

    #[error("Invalid directory `{0}`")]
    #[diagnostic(help("the path you passed is invalid"))]
    BadPath(PathBuf),

    #[error("Error parsing file `{0}`")]
    #[diagnostic(help("cargo-specification can only parse files that have an extension"))]
    CantParseFile(PathBuf),

    #[error("Error parsing file")]
    #[diagnostic(help("missing a startcode instruction before the endcode"))]
    MissingStartcode {
        #[source_code]
        src: NamedSource,

        #[label("This bit here")]
        bad_bit: (usize, usize),
    },

    #[error("Error parsing file")]
    #[diagnostic(help("missing endcode instruction"))]
    MissingEndcode {
        #[source_code]
        src: NamedSource,

        #[label("this startcode instruction is not terminated")]
        bad_bit: (usize, usize),
    },

    #[error("Error parsing file")]
    #[diagnostic(help("we are already in a startcode instruction"))]
    DoubleStartcode {
        #[source_code]
        src: NamedSource,

        #[label("this startcode instruction is invalid")]
        bad_bit: (usize, usize),
    },

    #[error("Error parsing file")]
    #[diagnostic(help("unrecognized instruction"))]
    BadInstruction {
        #[source_code]
        src: NamedSource,

        #[label("try spec:startcode or spec:endcode instead")]
        bad_bit: (usize, usize),
    },

    #[error("This is not a git repository, you can't use `@/` in the path of section {0}")]
    NotGitRepo(String),
}
