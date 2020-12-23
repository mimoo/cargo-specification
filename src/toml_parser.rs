//! A specification is all the concatenated comments from a list of files
//! The list of files is maintained by a Specification.toml file

use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;

/// TKTK
#[derive(Deserialize, Debug)]
pub(crate) struct SpecificationFile {
    specification: Specification,
    sections: BTreeMap<String, Vec<String>>, // vec of files
}

/// TKTK
#[derive(Deserialize, Debug)]
pub(crate) struct Specification {
    name: String,
    version: Option<String>,
    authors: Vec<String>,
}

///
pub(crate) fn parse_toml_spec(spec_file: &str) -> SpecificationFile {
    let mut file = File::open(spec_file).unwrap_or_else(|e| panic!("{}", e));
    let mut content = String::new();
    file.read_to_string(&mut content)
        .unwrap_or_else(|e| panic!("{}", e));
    toml::from_str(&content).unwrap_or_else(|e| panic!("{}", e))
}
