use std::path::Path;

use miette::{Diagnostic, NamedSource, SourceCode, SourceSpan};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, SpecError>;

#[derive(Error, Diagnostic, Debug)]
pub enum SpecError {
    #[error("Error parsing file")]
    #[diagnostic(help("missing a startcode instruction before the endcode"))]
    MissingStartcode {
        #[source_code]
        src: NamedSource,

        #[label("This bit here")]
        bad_bit: (usize, usize),
    },

    #[error("Error parsing file")]
    #[diagnostic(help("missing an endcode instruction to close the last startcode instruction"))]
    MissingEndcode {
        #[source_code]
        src: NamedSource,

        #[label("This bit here")]
        bad_bit: (usize, usize),
    },

    #[error("Error parsing file")]
    #[diagnostic(help("we are already in a startcode instruction"))]
    DoubleStartcode {
        #[source_code]
        src: NamedSource,

        #[label("This bit here")]
        bad_bit: (usize, usize),
    },

    #[error("Error parsing file")]
    #[diagnostic(help("unrecognized instruction"))]
    BadInstruction {
        #[source_code]
        src: NamedSource,

        #[label(
            "this instruction is not recognized, try spec:startencode or spec:endcode instead"
        )]
        bad_bit: (usize, usize),
    },
}
