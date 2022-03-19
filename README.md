# Stackyy
### A stack based, virtual machine language written in [Rust](https://rust-lang.org/)

## Description:

Stackyy is a stack based, virtual machine language inspired by
[Forth](https://en.wikipedia.org/wiki/Forth_(programming_language)) and
[Porth](https://gitlab.com/tsoding/porth).

## Usage:

1. Download the repo
2. Run ``cargo run -- --help`` for a list of all commands

## Building for release.

To use the standard optimizations run ``cargo build --release``.  
If the binary size is unsatifying run ``cargo build --profile=release-opt``

## ToDo:

- [x] Parsing completed
- [x] Basic language features
- [x] Type checking
- [x] Control flow (e.g. if while etc.) (Implemented via conditional functions])
- [x] Functions
- [x] Reflection (creating function handles from scratch)
- [ ] Included libraries (e.g. processes, io etc.)
- [ ] Speed up parsing
- [ ] Compiling to byte code and running
### Maybe
- [ ] Compile to ELF
- [ ] Self-hosted

## Example programs:

For example programs look in the examples directory