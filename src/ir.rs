use crate::vm::{RowsIter, StrId};
use std::borrow::Cow;
use std::fmt::Debug;
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct Relation {
    pub name: StrId,
    pub schema: Schema,
    pub rows: Vec<Row>,
}

// TODO: it would be very nice if these didn't need to allocate
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Row {
    pub values: Vec<Value>,
}
#[derive(Clone, Debug, Default)]
pub struct Schema {
    pub names: Vec<StrId>,
    pub types: Vec<VType>,
}

// TODO: floats mean you can't hash
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
    TakeFields(Box<IrExpr>, Vec<FieldId>),
    Filter(Box<IrExpr>, IrPred),
}

#[derive(Debug, Clone)]
pub enum IrPred {
    Literal(Value),
    Field(FieldId),
    Bin(BinOp, Box<Self>, Box<Self>),
}

#[derive(Debug)]
pub enum AstExpr<'src> {
    // TODO: this has side effects when compiled not when evaluated which probably the wrong choice.
    // TODO: support type annotations on headers
    CreateNamedTable {
        name: &'src str,
        headers: Vec<&'src str>,
        data: Vec<Vec<AstValue<'src>>>,
    },
    GetNamedTable(&'src str),
    TakeFields {
        input: Box<Loc<AstExpr<'src>>>,
        fields: Vec<&'src str>,
    },
    Filter {
        input: Box<Loc<AstExpr<'src>>>,
        cond: AstPred<'src>,
    },
}

#[derive(Debug, Copy, Clone)]
pub enum AstValue<'src> {
    Num(f64),
    Str(&'src str),
    Bool(bool),
}

#[derive(Debug, Clone)]
pub enum AstPred<'src> {
    Literal(AstValue<'src>),
    Field(&'src str),
    Bin(BinOp, Box<Self>, Box<Self>),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BinOp {
    Eq,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VType {
    Num,
    Str,
    Bool,
}

/// Remember a span in the src for error messages.
pub type Loc<T> = (T, Range<usize>); // TODO: range not actually used yet.

impl Schema {
    pub fn get_index(&self, id: StrId) -> Option<FieldId> {
        self.names
            .iter()
            .position(|&check| check == id)
            .map(FieldId)
    }

    pub fn get_type(&self, id: StrId) -> Option<VType> {
        self.types.get(self.get_index(id)?.0).copied()
    }
}

impl Relation {
    // TODO: its unfortunate that this clone happens before any filtering.
    pub fn iter(&self) -> RowsIter {
        RowsIter::new(self.rows.iter().cloned())
    }
}

impl Value {
    pub fn ty(&self) -> VType {
        match self {
            Value::Num(_) => VType::Num,
            Value::Str(_) => VType::Str,
            Value::Bool(_) => VType::Bool,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            &Value::Bool(b) => Some(b),
            _ => None,
        }
    }
}
