use std::collections::HashMap;
use std::fmt::{Debug};

#[derive(Default, Debug)]
pub struct Vm {
    pub strings: Vec<String>,
    pub string_cache: HashMap<String, StrId>,
    pub tables: HashMap<StrId, Relation>,
}

#[derive(Clone, Debug)]
pub struct Relation {
    pub name: StrId,
    pub schema: Schema,
    pub rows: Vec<Row>
}

#[derive(Clone, Debug, Default)]
pub struct Row {
    pub values: Vec<Value>
}

#[derive(Clone, Debug, Default)]
pub struct Schema {
    pub names: Vec<StrId>
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Value {
    Num(f64),
    Str(StrId),
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct StrId(pub usize);
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FieldId(pub usize);

#[derive(Debug)]
pub enum Expr {
    GetNamedTable(StrId),
    Project(Box<Expr>, Vec<FieldId>)
}

#[derive(Debug)]
pub enum AstExpr {
    GetNamedTable(String),
    Projection {
        input: Box<AstExpr>,
        prev: Schema,
        fields: Vec<String>,
    }
}

pub enum IterRows<'a> {
    Borrowed(&'a Relation, usize),
    FilterMap(Box<IterRows<'a>>, Box<dyn FnMut(Row) -> Option<Row>>)
}

impl<'a> Iterator for IterRows<'a> {
    type Item = Row;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            IterRows::Borrowed(rows, index) => {
                *index += 1;
                rows.rows.get(*index - 1).cloned()  // TODO: sad to realloc the vec if just want to filter
            }
            IterRows::FilterMap(prev,f) => {
                for r in prev {
                    if let Some(row) = f(r) {
                        return Some(row);
                    }
                }
                return None;
            }
        }
    }
}
