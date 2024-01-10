use relations::lex::Lexer;

fn make_numbers() -> String {
    let rows: String = [
        "Zero", "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten",
    ]
    .iter()
    .enumerate()
    .map(|(i, name)| format!("{}, \"{}\", {}\n", i, name, i.pow(2)))
    .collect();
    format!("Numbers = {{ Value, Name, Square\n{rows} }}")
}

#[test]
fn lex() {
    let src = make_numbers();
    println!("{src}");
    let mut lex = Lexer::new(&src);
    let mut token = lex.next();
    while token.is_ok() {
        println!("{:?}", token.unwrap());
        token = lex.next();
    }
}
