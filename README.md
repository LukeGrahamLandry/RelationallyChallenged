# COMP 3005 Challenge: relational algebra interpreter

## Language

It's a subset of the syntax of relaX. See `tests` folder for examples. 

```
Root => (Stmt)*
Stmt => '(' Stmt ')' | Ident | Select Expr Stmt | Pi Fields Stmt | Table
Table => Ident '{' Fields Rows '}'
Fields => (Ident ',')* Ident
Rows => ((Value ',')* Value)*
Expr => '(' Expr ')' | Value | Expr BinOp Expr
Value => quoted_string | number | Ident
BinOp => '='
Pi => 'pi' | 'Π' | 'π'
Select => 'select' | 'sigma' | 'σ'
```

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
