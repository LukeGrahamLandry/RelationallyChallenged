use crate::ir::{Relation, Row, Schema, Value};
use crate::vm::{RowsIter, Vm};

impl Vm {
    pub fn print_table(&self, e: &Relation) -> String {
        let rows = e
            .rows
            .iter()
            .map(|r| self.print_row(&r))
            .fold(String::new(), |a, b| a + &b + "\n");
        format!(
            "{0} = {{ \n{1}\n{2} }}",
            self.get_str(e.name),
            self.print_schema(&e.schema),
            rows
        )
    }

    pub fn print_rows_iter(&self, e: RowsIter) -> String {
        let rows =
            e.0.map(|r| self.print_row(&r))
                .fold(String::new(), |a, b| a + &b + "\n");
        format!("Relation {{\n{}}}", rows)
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
            Value::Null => format!("NaN"),
        }
    }
}
