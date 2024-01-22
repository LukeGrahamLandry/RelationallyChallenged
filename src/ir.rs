use crate::vm;
use crate::vm::{CompErr, RowsIter, StrId};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct Relation {
    pub name: StrId,
    pub schema: Schema,
    pub rows: Vec<Row>,
}

// TODO: it would be very nice if these didn't need to allocate
#[derive(Clone, Debug, Default, PartialEq, Hash, Eq)]
pub struct Row {
    pub values: Vec<Value>,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Schema {
    pub names: Vec<StrId>,
    pub types: Vec<VType>,
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub struct FieldName {
    parent: Option<StrId>,
    name: StrId,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Value {
    Num(f64),
    Str(StrId),
    Bool(bool),
    Null, // or NaN
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct FieldId(pub usize);

#[derive(Debug)]
pub enum IrExpr {
    GetNamedTable(StrId),
    TakeFields(Box<IrExpr>, Vec<FieldId>),
    Filter(Box<IrExpr>, IrPred),
    Bin {
        op: TableBinOp,
        lhs: Box<Self>,
        rhs: Box<Self>,
    },
    Join {
        // bools = is that side open
        lhs: (Box<Self>, bool),
        rhs: (Box<Self>, bool),
        cond: IrPred,
    },
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
        input: Box<Loc<Self>>,
        fields: Vec<&'src str>,
    },
    Filter {
        input: Box<Loc<Self>>,
        cond: AstPred<'src>,
    },
    Bin {
        op: TableBinOp,
        lhs: Box<Loc<Self>>,
        rhs: Box<Loc<Self>>,
    },
    Join {
        // bools = is that side open
        lhs: (Box<Loc<Self>>, bool),
        rhs: (Box<Loc<Self>>, bool),
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
    Or,
    And,
    Add,
    Sub,
    Mul,
    Div,
    Gt,
    Lt,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TableBinOp {
    Union,
    Intersection,
    Difference,
    Product,
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
    pub fn get_index(&self, id: StrId) -> vm::Res<FieldId> {
        // TODO: this check wont be required once i support name spacing (but should still be done when declaring a table)
        if self.names.iter().filter(|&&check| check == id).count() > 1 {
            return Err(CompErr::AmbiguousFieldName(id, self.clone()));
        }

        self.names
            .iter()
            .position(|&check| check == id)
            .map(FieldId)
            .ok_or_else(|| CompErr::BadFieldName(id, self.clone()))
    }

    pub fn get_type(&self, id: StrId) -> vm::Res<VType> {
        // unwrap cant fail if get_index didnt
        Ok(self.types.get(self.get_index(id)?.0).copied().unwrap())
    }
}

impl Relation {
    // TODO: its unfortunate that this clone happens before any filtering.
    pub fn iter(&self) -> RowsIter {
        RowsIter::new(self.rows.iter().cloned())
    }
}

impl Value {
    // TODO: null is kinda any type? idk
    pub fn ty(&self) -> VType {
        match self {
            Value::Num(_) | Value::Null => VType::Num,
            Value::Str(_) => VType::Str,
            Value::Bool(_) => VType::Bool,
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            &Value::Bool(b) => b,
            Value::Null => false,
            _ => panic!("expect bool but {self:?}"),
        }
    }

    pub fn as_num(&self) -> f64 {
        match self {
            &Value::Num(n) => n,
            Value::Null => f64::NAN,
            _ => panic!("expect num but {self:?}"),
        }
    }
}

// f64: !Hash so do it myself with some random garbage for the tag
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Num(n) => {
                debug_assert!(!n.is_nan());
                state.write_i64(30197513748900);
                state.write_u64(n.to_bits());
            }
            Value::Str(s) => {
                state.write_i64(98711294340598);
                state.write_usize(s.0);
            }
            Value::Bool(true) => state.write_i64(78219104210329751),
            Value::Bool(false) => state.write_i64(1831294217),
            Value::Null => state.write_i64(8210726170476),
        }
    }
}

// Can't derive because f64: !Eq because NaN!=NaN,
// but as long as I always carefully represent those with Value::Nan, then its fine.
impl Eq for Value {}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        if value.is_nan() {
            Value::Null
        } else {
            Value::Num(value)
        }
    }
}
