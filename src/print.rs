use crate::ir::{Relation, Row, Schema, StrId, Value, Vm};
use crate::vm::RowsIter;

impl Vm {
    pub fn print_table(&self, e: &Relation) -> String {
        self.print_rows_iter(RowsIter(Box::new(e.rows.iter().cloned()), e.schema.clone()), e.name)
    }

    pub fn print_rows_iter(&self, e: RowsIter, name: StrId) -> String {
        let rows = e.0.map(|r| self.print_row(&r)).fold(String::new(), |a, b| a + &b + "\n");
        format!("{0} = {{ \n{1}\n{2} }}", self.get_str(name), self.print_schema(&e.1), rows)
    }

    pub fn print_schema(&self, e: &Schema) -> String {
        let mut s =  e.names.iter().map(|&v| self.get_str(v)).fold(String::new(), |a, b| a + &b + ", ");
        s.pop(); s.pop(); // comma space
        s
    }

    pub fn print_row(&self, e: &Row) -> String {
        let mut s = e.values.iter().map(|v| self.print_value(v)).fold(String::new(), |a, b| a + &b + ", ");
        s.pop();  // comma
        s
    }

    pub fn print_value(&self, e: &Value) -> String {
        match e {
            Value::Num(n) => format!("\"{n}\""),
            &Value::Str(s) => self.get_str(s).to_string(),
        }
    }

    pub fn get_str(&self, e: StrId) -> &str {
        &self.strings[e.0]
    }

    pub fn add_str(&mut self, e: &str) -> StrId {
        match self.string_cache.get(e) {
            None => {
                let id = StrId(self.strings.len());
                self.string_cache.insert(e.to_string(), id);
                self.strings.push(e.to_string());
                id
            }
            Some(&id) => id,
        }
    }
}
