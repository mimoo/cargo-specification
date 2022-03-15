# Cargo spec &emsp; ![](https://img.shields.io/crates/v/cargo-spec.svg)

This is a tool to use your code in your specifications.
To see it in action, just look at the code in this codebase, as well as [the spec it produced](https://mimoo.github.io/cargo-specification/).

## Usage

You'll need to have a [`Specification.toml`](#configuration) file in your project. See the [configuration section](#configuration) for more information.

```console
$ cargo install cargo-spec
$ cargo spec

=> html output saved at ./specification.md
```

You can also watch for any changes:

```console
$ cargo spec watch
```

## How to write a specification?

cargo-spec's philosophy stems from the fact that most protocols  often come from a reference implementation. That reference implementation tends to change, and as you want your spec to be up to date you will want to keep parts of the spec as close to the code as possible.

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
abstract_modules = "src/module.rs"
```

### Template

A template is simply a markdown file that contains placeholders. The path of the template must be specified in the `Specification.toml` file.

```markdown
# Consensus specification

Here's the consensus spec

## Data structures

{sections.data_structures}

## Abstract modules

{sections.abstract_modules}
```

### Spec comments in your code

Cargo-spec recognizes comments starting with `//~ `. For example:

```rust
//~ some specification text
```

**While cargo-spec is language-agnostic, it is currently not detect other types of comments like `#`, `(* *)`, etc. If you need this feature please [post an issue](https://github.com/mimoo/cargo-specification/issues/new)**.

You can also import blocks of code by surrounding them with `//~ spec:startcode` and `//~ spec:endcode`:


```rust
//~ spec:startcode
struct SomeStruct {
  a: u8,
  b: u64,
}
//~ spec:encode
```

## Continuous Integration

You'll most likely want to enforce that PRs contains up-to-date specification files checked-in:

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
          cargo install cargo-spec --version 0.1.3
          cd <spec_folder>
          cargo spec
          git diff --exit-code
```

## Projects making use of cargo-spec

* [kimchi](https://github.com/o1-labs/proof-systems/blob/master/book/specifications/README.md) ([spec](https://o1-labs.github.io/proof-systems/specs/kimchi.html))
