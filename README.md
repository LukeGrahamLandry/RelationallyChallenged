# COMP 3005 Challenge: relational algebra interpreter

## Architecture 

```
[user]
src code string
[lex.rs]
lazy stream of tokens
[parse.rs]
AstExpr: has references into src
[vm.rs:compile]
IrExpr: has references into intern pool & field names resolved to indices 
[vm.rs:eval]
lazy stream of rows
[user]
```

## Build

- [Install the Rust Compiler](https://www.rust-lang.org/tools/install)
- `cargo test`
- `cargo build --release`
- `target/release/relations`
