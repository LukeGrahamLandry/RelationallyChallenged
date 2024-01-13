use relations::lex::{LexErr, Lexer};
use relations::parse::Parser;

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

const EMPLOYEES_TABLE: &str = r#"Employees = {EID, Name, Age
E1, John, 32
E2, Alice, 28
E3, Bob, 29
}"#;

const SCHOOL_TABLE: &str = r#"
Student = {ID, Name, Age, Major
"1", "Alice", 20, "Computer Science"
"2", "Bob", 22, "Physics"
"3", "Charlie", 21, "Mathematics"
}

Courses = {CourseID, CourseName, Professor
"C101", "Databases", "Dr. Smith"
"C102", "Quantum Physics", "Dr. Doe"
"C103", "Calculus", "Dr. White"
}"#;

#[test]
fn lex() {
    lex_span_sanity(&make_numbers(), 0);
    lex_span_sanity(EMPLOYEES_TABLE, 0);
    lex_span_sanity(SCHOOL_TABLE, 5);
}

// If there's no whitespace in quoted strings and no comments,
// The total size of tokens should equal the number of non-whitespace characters.
fn lex_span_sanity(src: &str, hack_quoted_whitespace: usize) {
    let interesting_count =
        src.chars().filter(|c| !c.is_whitespace()).count() + hack_quoted_whitespace;
    let mut lex = Lexer::new(src);
    let mut found_count = 0;
    let mut prev_end = 0;
    loop {
        match lex.next() {
            Ok(token) => {
                assert!(token.1.start >= prev_end, "tokens overlap");
                found_count += token.1.end - token.1.start;
                prev_end = token.1.end;
            }
            Err(LexErr::EOF) => break,
            e => panic!("{:?}", e),
        }
    }
    assert_eq!(interesting_count, found_count);
}

#[test]
fn parse() {
    let mut parse = Parser::new(Lexer::new(EMPLOYEES_TABLE));
    println!("{:?}", parse.next());
}
