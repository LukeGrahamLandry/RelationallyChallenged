# COMP 3005 Challenge: relational algebra interpreter

- Luke Graham Landry 101226905
- [Try it online!](https://lukegrahamlandry.ca/relalg/)
- Building from source requires installing [the Rust Compiler](https://www.rust-lang.org/tools/install) and the wasm32-unknown-unknown toolchain. 

## Language

- See `tests` folder for examples.
- It's mostly a subset of the syntax of relaX. Same weird symbols and sane word alternatives. 
- Join requires 
- Join follows the semantics of the examples in lecture 2 slides instead of relax. 
So you can use table names as qualifiers in join conditions and two columns with the same name (but different qualifiers) appear in the result. 
Similarly, product with duplicate names is allowed if table names can disambiguate.

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
