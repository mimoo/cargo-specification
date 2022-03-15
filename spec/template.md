# {metadata.name}

author: {metadata.authors.0}

## overview

Building a specification is pretty straight forward. Cargo-spec follows these steps:

{sections.build}

## Toml parser

The toml parser expects a manifest specification file that follows the following configuration:

{sections.toml_parser}

The structures are deserialized using the [toml encoding](https://github.com/toml-lang/toml).

## Comment parser

Any placeholder in the template will get replaced by comments extracted from code.
The specification manifest file contains the list of these files.

{sections.parser}
