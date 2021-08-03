//~ ## TOML Parser
//~ A specification is all the concatenated comments from a list of files
//~ The list of files is maintained by a Specification.toml file.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;

//~ Below are the different structures that we use to organize the specification:
//~ spec:startcode

/// A specification file contains a specification, as well as sections of (title, text)
#[derive(Deserialize, Debug)]
pub(crate) struct SpecificationFile {
    /// information about a specification
    pub metadata: Metadata,
    /// vec of files
    pub sections: BTreeMap<String, Vec<String>>,
}

/// Metadata about a specification
#[derive(Deserialize, Debug)]
pub(crate) struct Metadata {
    /// Name of the specification
    pub name: String,
    /// A description
    pub description: String,
    /// Version of the spec
    pub version: Option<String>,
    /// Authors, if any
    pub authors: Vec<String>,
}

//~ spec:endcode

///
pub(crate) fn parse_toml_spec(spec_file: &str) -> SpecificationFile {
    let mut file = File::open(spec_file).unwrap_or_else(|e| panic!("{}", e));
    let mut content = String::new();
    file.read_to_string(&mut content)
        .unwrap_or_else(|e| panic!("{}", e));
    toml::from_str(&content).unwrap_or_else(|e| panic!("{}", e))
}
