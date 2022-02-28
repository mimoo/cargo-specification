# The code is the spec

This is a tool to transform your code in a specification!
This tool should be agnostic to the code you use.

To see it in action, just look at the code in this codebase as well as [the spec it produced](https://mimoo.github.io/cargo-specification/).

## Usage

```
$ cargo install cargo-spec
$ cargo spec -o specification.md
```

## How to write a specification?

With cargo-spec, you write your specification as a markdown file and extract part of your text from your code.

### Order your specification

A specification will get generated from concatenated comments, obtained from your code.
To list and order the partial specifications, we use a toml file:

```toml
[specification]
name = "Consensus"
version = "0.1.0"
authors = ["David Wong"]

[config]
template = "spec/template.md"

[sections]
data_structures = "src/data_structures.rs"
abstract_modules = "src/module.rs"
```

Note that the name of each section is only relevant for clarity in the toml file.

### Comment in your code

Cargo-specification recognizes comments starting with `//~ `. For example:

```
//~ some specification text
```

If you don't like this, or it doesn't work with the language you use (although we should recognize automatically the extension of your files and change the comment accordingly), you can change it via the command line, just make sure that it is a valid command:

```
$ cargo specification path/to/Specification.toml -d "#~ " -o specification.html
```

You can also import blocks of code by surrounding them with `//~ spec:startcode` and `//~ spec:endcode`:


```
//~ spec:startcode
struct SomeStruct {
  a: u8,
  b: u64,
}
//~ spec:encode
```
