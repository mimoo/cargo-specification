use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use toml::de::Error;

//~ spec:startcode
/// A specification file contains a specification, as well as sections of (title, text)
#[derive(Serialize, Deserialize, Debug)]
pub struct Specification {
    /// information about a specification
    pub metadata: Metadata,
    /// configuration of the specification
    pub config: Config,
    /// files to use for the specification's content
    pub sections: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// main template file
    pub template: String,
}

/// Metadata about a specification
#[derive(Serialize, Deserialize, Debug)]
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

/// Parse a `Specification.toml` file into a [Specification] struct.
pub fn parse_toml_spec(spec_file: &Path) -> Result<Specification, Error> {
    let mut file = File::open(spec_file).unwrap_or_else(|e| panic!("cannot open the specification file {}, make sure you pass a specification toml file via --specification-path", e));
    let mut content = String::new();
    file.read_to_string(&mut content)
        .unwrap_or_else(|e| panic!("{}", e));
    toml::from_str(&content)
}
