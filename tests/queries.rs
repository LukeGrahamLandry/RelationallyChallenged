use relations::ir::Value;
use relations::lex::{Lexer, Token, TokenType};
use relations::parse::{ParseErr, Parser};
use relations::vm::Vm;

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

// TODO: the other syntax. Name (Headers) = { Rows }
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

const SCHOOL_TABLE_2: &str = r#"
Student = {
id, name, email
1, 'Alex', 'alex@carleton.ca'
2, 'John', 'john@carleton.ca'
3, 'Mo', 'mo@carleton.ca'
}

Course = {
name, hours
'Math', 3
'Pythics', 2
'Network', 3
}

takes = {
sid, cname
1, 'Math'
1, 'Pythics'
1, 'Network'
2, 'Network'
3, 'Math'
}
"#;

const SELECT_FOUR: &str = "pi Square (select Name='Four' (pi Square, Name (Numbers)))";

// Use with make_numbers()
const FOUR_AND_FIVE_QUERY: &str = r#"
(pi Value 
(
    (select Name='Four' (Numbers)) 
    union 
    (select Name='Five' (Numbers))
) 
union 
(
    select Name='Four' (Numbers)
)
)
"#;

const JOBS: &str = r#"
Employee = { id, name, email, Dept
1, "Alex", "a@c", "Sales"
2, "John", "j@c", "Finance"
3, "Mo", "m@c", "HR"
}

Department = { dname, budget
"Finance", 20000
"Sales", 30000
"IT", 40000
}
"#;

#[test]
fn select_pi() {
    let four_squared = run_query(&format!("{}\n {SELECT_FOUR}", make_numbers()));
    assert_eq!(four_squared, vec![vec![Value::Num(16.0)]]);
}

#[test]
fn union_simple() {
    let four_and_five = run_query(&format!("{}{FOUR_AND_FIVE_QUERY}", make_numbers()));
    // Order doesn't matter but require no dups.
    let a = four_and_five == vec![vec![Value::Num(4.0)], vec![Value::Num(5.0)]];
    let b = four_and_five == vec![vec![Value::Num(5.0)], vec![Value::Num(4.0)]];
    assert!(a || b);
}

#[test]
fn set_subtract() {
    let five = run_query(&format!(
        "{}({FOUR_AND_FIVE_QUERY}) - pi Value (select ((Name='Four') or (Name='Ten')) Numbers)",
        make_numbers()
    ));
    assert_eq!(five, vec![vec![Value::Num(5.0)]]);
}

// TODO: intersection, math/cmp/logic, all the joins

#[test]
fn inner_join_school() {
    let res = run_query(&format!("{SCHOOL_TABLE_2}(Student) ⨝ id=sid (takes)"));
    assert_eq!(res.len(), 5);
    assert_eq!(res.iter().filter(|row| row[0] == row[3]).count(), res.len());
    assert_eq!(res.iter().filter(|row| row[0].as_num() == 1.0).count(), 3);
    assert_eq!(res.iter().filter(|row| row[0].as_num() == 2.0).count(), 1);
    assert_eq!(res.iter().filter(|row| row[0].as_num() == 3.0).count(), 1);
    assert_eq!(count_nulls(&res), 0);
}

// TODO: support namespacing so can make JOBS:Department:"dname" back to "name" for all the join tests below

#[test]
fn inner_join() {
    // TODO: fix parser so dont need
    let res = run_query(&format!("{JOBS} (Employee) ⨝Dept=dname (Department)"));
    assert_eq!(res.len(), 2);
    assert_eq!(count_nulls(&res), 0);
}

#[test]
fn lex() {
    lex_span_sanity(&make_numbers(), 0);
    lex_span_sanity(EMPLOYEES_TABLE, 0);
    lex_span_sanity(SCHOOL_TABLE, 5);
    lex_span_sanity(SCHOOL_TABLE_2, 0);
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
            Ok(Token(TokenType::EOF, _)) => break,
            Ok(token) => {
                assert!(token.1.start >= prev_end, "tokens overlap");
                found_count += token.1.end - token.1.start;
                prev_end = token.1.end;
            }
            e => panic!("{:?}", e),
        }
    }
    assert_eq!(interesting_count, found_count);
}

// returns result of last query
// any before are just compiled for side effects (ie. setup named tables)
fn run_query(src: &str) -> Vec<Vec<Value>> {
    let mut vm = Vm::default();
    let mut parse = Parser::new(Lexer::new(src));
    let mut next = parse.next();
    let mut q = None;
    while next.is_ok() {
        q = Some(vm.compile_impure(next.unwrap()).unwrap().0);
        next = parse.next();
    }
    match next {
        Err(ParseErr::EOF) => {}
        e => panic!("{e:?}"),
    }
    let q = q.unwrap();
    let res = vm.eval(&q);
    res.0.map(|r| r.values).collect()
}

fn count_nulls(rows: &[Vec<Value>]) -> usize {
    rows.iter()
        .map(|row| row.iter().filter(|v| matches!(v, Value::Null)).count())
        .sum()
}
