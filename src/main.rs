use relations::ir::AstExpr;
use relations::lex::Lexer;
use relations::parse::{ParseErr, Parser};
use relations::vm::Vm;
use std::io;
use std::io::Write;
use std::time::Instant;

fn main() {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut in_progress = String::new();
    let mut vm = Vm::default();

    println!("Relational Algebra REPL");
    println!("- Enter relations/queries then an empty line to evaluate.");
    println!("- Vm state is retained between evaluations.");
    print!(">> ");
    io::stdout().flush().unwrap();
    loop {
        stdin.read_line(&mut buffer).expect("Read input");
        if buffer.trim().is_empty() && !in_progress.trim().is_empty() {
            let start_time = Instant::now();
            // TODO: kinda copy-paste
            let mut parse = Parser::new(Lexer::new(&in_progress));
            let mut next = parse.next();
            let mut result = String::new();
            while next.is_ok() {
                let ast = next.unwrap();
                let is_interesting = !matches!(ast.0, AstExpr::CreateNamedTable { .. });
                match vm.compile_impure(ast) {
                    Ok((q, _)) => {
                        if is_interesting {
                            let res = vm.eval(&q);
                            result += "==============\n";
                            result += &format!("{}\n", vm.print_rows_iter(res));
                        }
                    }
                    Err(e) => {
                        result += &format!("{e:?}\n");
                    }
                }
                next = parse.next();
            }
            match next {
                Err(ParseErr::EOF) => {}
                Err(e) => {
                    result += &format!("{e:?}\n");
                }
                Ok(_) => unreachable!(), // TODO: move match into loop so this is obvious
            }
            let end_time = Instant::now();

            println!("{result}");
            println!(
                "Parse + Eval finished in {}ms.",
                (end_time - start_time).as_millis()
            );
            print!(">> ");
            io::stdout().flush().unwrap();
            in_progress.clear();
        } else {
            in_progress.push_str(&buffer);
            buffer.clear();
        }
    }
}
