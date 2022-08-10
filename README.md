# Cargo spec &emsp; ![](https://img.shields.io/crates/v/cargo-spec.svg)

This is a tool to turn your code into a specification.
It is language agnostic, but you need Cargo (which you can install via [Rustup](https://rustup.rs/)) to use it.
To see it in action, just look at the code in this codebase, as well as [the spec it produced](https://mimoo.github.io/cargo-specification/).
You can read more about the concepts behind it in the blogpost [The code is the specification: introducing cargo-spec](https://cryptologie.net/article/553/the-code-is-the-specification-introducing-cargo-spec/).

## Usage

**initialize**. To start your specification, you need two files: a [`Specification.toml`](#configuration) configuration file and a template containing your specification written in [markdown](https://daringfireball.net/projects/markdown/).
You can create these in your current directory via `cargo spec new <NAME>`, or on a given path via `cargo spec init <PATH>`:

```console
$ cargo install cargo-spec
$ cargo spec new

Created new specification as Specification.toml and specification_template.md
You can now run `cargo spec build` to create the specification file
```

**build**. Building your specification will convert your template into the desired format (markdown by default), at some path (`specification.md` by default):

```console
$ cargo spec build

=> html output saved at ./specification.md
```

You can also watch for any changes:

```console
$ cargo spec watch

=> html output saved at ./specification.md
```

## How to write a specification?

cargo-spec's philosophy stems from the fact that most protocols often come from a reference implementation. That reference implementation tends to change, and as you want your spec to be up to date you will want to keep parts of the spec as close to the code as possible.

cargo-spec allows you to write your specification as a markdown file, and extract special "spec comments" from your code to fill in sections of your spec.

### Configuration

To run cargo-spec, you need a `Specification.toml` file.
Although you can specify a different filename via the `--specification-path` option.

```toml
[specification]
# some metadata
name = "Consensus"
description = "This specification describes the consensus protocol."
version = "0.1.0"
authors = ["David Wong"]

[config]
# the path to your template
template = "template.md"

[sections]
# all the files you want to extract "spec comments" from
data_structures = "src/data_structures.rs"
abstract_modules = "@/src/module.rs" # you can also use absolute paths (you need to be in a git repo)
```

### Template

A template is simply a markdown file that contains placeholders. The path of the template must be specified in the `Specification.toml` file. 
By default the `cargo spec new <NAME>` (or `cargo spec init <PATH>`) command will use `specification_template.md` as template.

```markdown
# Consensus specification

Here's the consensus spec

## Data structures

{sections.data_structures}

## Abstract modules

{sections.abstract_modules}
```

### Spec comments in your code

Cargo-spec recognizes comments starting with the tilde `~`. 
For example, in rust:

```rust
//~ some specification text
```

in Python:

```python
#~ here's some spec
```

or in OCaml:

```ocaml
(*~ some spec *)
```

> While cargo-spec is language-agnostic, it does not support all type of comments. [Post an issue](https://github.com/mimoo/cargo-specification/issues/new) if it does not work for the language you're using.

## Nested lists

It can be tiring to indent manually your comments to create nested lists:

```rust
//~ - a list
//~   - a nested list
```

instead, simply add `~` at the start of your spec comment to add indentation:

```rust
//~ - a list
//~~ - a nested list
```

## Importing code

You can import blocks of code by surrounding them with `//~ spec:startcode` and `//~ spec:endcode`:


```rust
//~ spec:startcode
struct SomeStruct {
  a: u8,
  b: u64,
}
//~ spec:endcode
```

## Continuous Integration

You'll most likely want to enforce that PRs contains up-to-date specification files checked-in. 
You can do this for example with this Github Action:

```yml
name: Check specifications

on:
  pull_request:

jobs:
  run_checks:
    runs-on: ubuntu-latest
    name: Enforce up-to-date specification files
    steps:

      - name: Checkout PR
        uses: actions/checkout@v2

      - name: Set up cargo/rust
        uses: actions-rs/toolchain@v1

      - name: Check that up-to-date specification is checked in
        run: |
          cargo install cargo-spec
          cd <spec_folder>
          cargo spec build
          git diff --exit-code
```

## Projects making use of cargo-spec

* [cargo-spec]() ([spec](https://mimoo.github.io/cargo-specification/))
* [kimchi](https://github.com/o1-labs/proof-systems/blob/master/book/specifications/README.md) ([spec](https://o1-labs.github.io/proof-systems/specs/kimchi.html))

## License

The Cargo spec Project is dual-licensed under Apache 2.0 and MIT terms. See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
