use std::borrow::Cow;
use std::fmt::Debug;
use std::ops::Range;
use crate::vm::{RowsIter, StrId};

#[derive(Clone, Debug)]
pub struct Relation {
    pub name: StrId,
    pub schema: Schema,
    pub rows: Vec<Row>
}

// TODO: it would be very nice if these didn't need to allocate
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
    Bool(bool),
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FieldId(pub usize);

#[derive(Debug)]
pub enum IrExpr {
    GetNamedTable(StrId),
    Projection(Box<IrExpr>, Vec<FieldId>)
}

#[derive(Debug)]
pub enum AstExpr<'src> {
    GetNamedTable(&'src str),
    /// Filter and reorder fields
    Projection {
        input: Box<Loc<AstExpr<'src>>>,
        prev: Schema,
        fields: Vec<&'src str>,
    }
}

/// Remember a span in the src for error messages.
pub type Loc<T> = (T, Range<usize>);  // TODO: range not actually used yet

impl Schema {
    pub fn get_index(&self, id: StrId) -> Option<FieldId> {
        self.names.iter().position(|&check| check == id).map(FieldId)
    }
}

impl Relation {
    pub fn iter(&self) -> RowsIter {
        RowsIter::new(self.rows.iter().cloned(), Cow::Borrowed(&self.schema))
    }
}
