# Stackyy
### A stack based, virtual machine language written in [Rust](https://rust-lang.org/)

## Description:

Stackyy is a stack based, virtual machine language inspired by 
[Forth](https://en.wikipedia.org/wiki/Forth_(programming_language)) and 
[Porth](https://gitlab.com/tsoding/porth).

## Usage:

1. Download the repo
2. Run ``cargo run -- --help`` for a list of all commands


## ToDo:

- [x] Parsing completed
- [x] Basic language features
- [x] Type checking
- [x] Control flow (e.g. if while etc.)
- [x] Functions
- [ ] Compiling to byte code and running
- [ ] Included libraries (e.g. processes, io etc.)
### Maybe
- [ ] Compile to ELF
- [ ] Self-hosted

## Example programs:

Hello world:

```shell
"Hello world" println
```

Stack manipulation

```shell
"x2" dup println println
```