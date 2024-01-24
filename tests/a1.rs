use relations::ir::Value;
use relations::lex::{Lexer, Token, TokenType};
use relations::parse::{ParseErr, Parser};
use relations::vm::Vm;
use std::collections::HashSet;

const A1: &str = include_str!("a1.txt");

/// Select employees whose salaries are greater than $3000.
#[test]
fn q1() {
    let a = q("sigma (Salary > 3000) (Employees)");
    assert_eq!(a.len(), 3);
}

// TODO: nice way to get the strings out
/// Find the position of David.
#[test]
fn q2() {
    let a = q("pi Position (sigma (Name = 'David') (Employees))");
    assert_eq!(a.len(), 1); // TODO: assert Developer
    assert_eq!(a[0].len(), 1);
}

/// Find the names of employees who are managers in the IT department.
#[test]
fn q3() {
    // TODO: mine needs the join filter but relax wont accept it
    // relax: pi Name ((sigma (DeptName='IT') (Departments)) ⨝ (sigma (Position = 'Manager') (Employees)))
    let a = q("pi Name ((sigma (DeptName='IT') (Departments)) ⨝(Employees.DeptID = Departments.DeptID) (sigma (Position = 'Manager') (Employees)))");
    assert_eq!(a.len(), 1); // TODO: assert Alice
    assert_eq!(a[0].len(), 1);
}

// #[test]
// fn q4() {
//     let expensive = "(sigma (Budget > 1000000) (Projects))";
//     let proj_and_lead = format!("{expensive} ⨝(LeadEmpID =  EmpID) (Employees)");
//
//     let a = q("");
// }

// List projects (project's name only) led by employees from the Finance department.
#[test]
fn q5() {
    let proj_and_lead = "(Projects) ⨝(LeadEmpID =  EmpID) (Employees)";
    let finance = "sigma (DeptName='Finance') (Departments)";
    let interesting =
        format!("({proj_and_lead}) ⨝(Employees.DeptID =  Departments.DeptID) ({finance})");
    let names = format!("pi ProjectName ({interesting})");

    let a = q(&names);
    assert_eq!(a.len(), 1);
    assert_eq!(a[0].len(), 1);
}

fn q(query: &str) -> Vec<Vec<Value>> {
    println!("{}", query);
    run_query(&format!("{}\n {query}", include_str!("a1.txt")))
}

// TODO: cringe copy-paste from queries.rs
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
