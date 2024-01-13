use crate::ir::{Relation, Row, Schema, Value};
use crate::vm::{RowsIter, StrId, Vm};

impl Vm {
    pub fn print_table(&self, e: &Relation) -> String {
        self.print_rows_iter(e.iter(), e.name)
    }

    pub fn print_rows_iter(&self, e: RowsIter, name: StrId) -> String {
        let rows =
            e.0.map(|r| self.print_row(&r))
                .fold(String::new(), |a, b| a + &b + "\n");
        format!(
            "{0} = {{ \n{1}\n{2} }}",
            self.get_str(name),
            self.print_schema(&e.1),
            rows
        )
    }

    pub fn print_schema(&self, e: &Schema) -> String {
        let mut s = e
            .names
            .iter()
            .map(|&v| self.get_str(v))
            .fold(String::new(), |a, b| a + &b + ", ");
        s.pop();
        s.pop(); // comma space
        s
    }

    pub fn print_row(&self, e: &Row) -> String {
        let mut s = e
            .values
            .iter()
            .map(|v| self.print_value(v))
            .fold(String::new(), |a, b| a + &b + ", ");
        s.pop();
        s.pop(); // comma space
        s
    }

    pub fn print_value(&self, e: &Value) -> String {
        match e {
            Value::Num(n) => format!("{n}"),
            &Value::Str(s) => format!("\"{}\"", self.get_str(s)),
            Value::Bool(b) => format!("{b}"),
        }
    }
}
