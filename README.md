# The code is the spec

This is a tool to transform your code in a specification!
This tool should be agnostic to the code you use

## Usage

```
$ cargo install cargo-specification
$ cargo specification path/to/Specification.toml -o specification.html
```

## How to write a specification?

With cargo-specification, you write your specification in your code!

### Order your specification

A specification will get generated from concatenated comments, obtained from your code.
To list and order the partial specifications, we use a toml file:

```toml
[specification]
name = "Consensus"
version = "0.1.0"
authors = ["David Wong"]

[sections]
data_structures = ["src/data_structures.rs"]
abstract_modules = ["src/module1.rs", "src/module2.rs"]
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
