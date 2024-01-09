use std::iter::from_fn;
use crate::ir::{AstExpr, Expr, FieldId, IterRows, Relation, Row, Schema, StrId, Vm};

pub struct RowsIter<'a>(pub Box<dyn Iterator<Item=Row> + 'a>, pub Schema);

impl Vm {
    pub fn eval<'a>(&'a self, expr: &'a Expr) -> RowsIter {
        match expr {
            Expr::GetNamedTable(name) => {
                let table = self.tables.get(name).unwrap();
                RowsIter(Box::new(table.rows.iter().cloned()), table.schema.clone())
            },
            Expr::Project(prev, fields) => {
                let mut prev = self.eval(prev);
                let schema = Schema {
                    names: fields.iter().map(|i| prev.1.names[i.0]).collect(),
                };
                RowsIter(Box::new(from_fn(move || {
                    let row = prev.0.next()?;
                    let result = Row {
                        values: fields.iter().map(|i| row.values[i.0]).collect(),
                    };
                    Some(result)
                })), schema)
            }
        }
    }

    // TODO: result
    pub fn compile(&mut self, expr: AstExpr) -> Expr {
        match expr {
            AstExpr::GetNamedTable(name) => Expr::GetNamedTable(self.add_str(&name)),
            AstExpr::Projection { input, prev, fields } => {
                let input = self.compile(*input);
                let fields = fields.iter().map(|name| {
                    let id = self.add_str(name);
                    prev.get_index(id).unwrap()
                }).collect();
                Expr::Project(Box::new(input), fields)
            }
        }
    }
}

impl Schema {
    pub fn get_index(&self, id: StrId) -> Option<FieldId> {
        self.names.iter().position(|&check| check == id).map(FieldId)
    }
}