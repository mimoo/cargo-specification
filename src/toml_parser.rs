//~ ## TOML Parser
//~ A specification is all the concatenated comments from a list of files
//~ The list of files is maintained by a Specification.toml file.

use indexmap::IndexMap;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;

//~ Below are the different structures that we use to organize the specification:
//~ spec:startcode

/// A specification file contains a specification, as well as sections of (title, text)
#[derive(Deserialize, Debug)]
pub struct Specification {
    /// information about a specification
    pub metadata: Metadata,
    /// vec of files
    pub sections: IndexMap<String, Vec<String>>,
}

/// Metadata about a specification
#[derive(Deserialize, Debug)]
pub struct Metadata {
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
pub fn parse_toml_spec(spec_file: &str) -> Specification {
    let mut file = File::open(spec_file).unwrap_or_else(|e| panic!("cannot open the specification file {}, make sure you pass a specification toml file via --specification-path", e));
    let mut content = String::new();
    file.read_to_string(&mut content)
        .unwrap_or_else(|e| panic!("{}", e));
    toml::from_str(&content).unwrap_or_else(|e| panic!("{}", e))
}
