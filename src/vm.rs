use crate::ir::{
    AstExpr, AstPred, AstValue, BinOp, IrExpr, IrPred, Loc, Relation, Row, Schema, VType, Value,
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::BitAnd;
use std::rc::Rc;

// TODO: you can never remove strings from the cache to reclaim memory because someone might have a StrId referencing it.
#[derive(Default, Debug, Clone)]
pub struct Vm {
    strings: Vec<Rc<str>>,
    string_cache: HashMap<Rc<str>, StrId>,
    pub tables: HashMap<StrId, Relation>,
}

#[derive(Debug, Clone)]
pub enum CompErr {
    // TODO
}

pub type Res<T> = Result<T, CompErr>;

// TODO: maybe row should be a cow or better would be wrap the whole thing in an enum so you switch over borrowed vs owned once when creating the closure, not each row when running it
pub struct RowsIter<'a>(pub Box<dyn Iterator<Item = Row> + 'a>);

impl Vm {
    pub fn exec_from_src(&mut self, src: &str) -> Option<RowsIter> {
        todo!()
    }

    // TODO: if i was feeling fancy, the Vm could have a pool of row vecs and impl drop to return them
    //       so you dont have to do an allocation for every row. (same for the closure boxes but that's harder to think about)
    /// If this ever returns None, its a bug in compile() so I don't care about error messages but I don't want to unwrap and crash.
    /// The expr must have been compiled on THIS vm instance.
    #[must_use = "RowsIter is lazy and must be polled."]
    pub fn eval<'a>(&'a self, expr: &'a IrExpr) -> Option<RowsIter<'a>> {
        Some(match expr {
            IrExpr::GetNamedTable(name) => self.tables.get(name)?.iter(),
            IrExpr::TakeFields(prev, fields) => {
                let prev = self.eval(prev)?;
                RowsIter::new(prev.0.map(|row| Row {
                    // TODO: could filter and reorder the existing vec?
                    values: fields.iter().map(|i| row.values[i.0]).collect(),
                }))
            }
            IrExpr::Filter(prev, pred) => {
                let pred = self.eval_pred(pred);
                let prev = self.eval(prev)?;
                RowsIter::new(prev.0.filter(move |row| pred(row).as_bool().unwrap()))
            }
        })
    }

    fn eval_pred<'a>(&self, pred: &'a IrPred) -> Box<dyn Fn(&Row) -> Value + 'a> {
        match pred {
            IrPred::Literal(lit) => Box::new(|_| *lit),
            IrPred::Field(f) => Box::new(|row| row.values[f.0]),
            IrPred::Bin(op, lhs, rhs) => {
                let lhs = self.eval_pred(lhs);
                let rhs = self.eval_pred(rhs);
                match op {
                    BinOp::Eq => Box::new(move |row| Value::Bool(lhs(row) == rhs(row))),
                }
            }
        }
    }

    // TODO: result instead of assert/unwrap
    // TODO: infer type so you dont have to specify schema on projection
    /// Preprocess and type check an ast.
    /// The result is only valid if eval-ed on THIS vm instance. TODO: enforce with lifetime
    /// Note: CreateNamedTable runs NOW, not during eval.
    pub fn compile_impure(&mut self, (expr, _): Loc<AstExpr>) -> (IrExpr, Schema) {
        match expr {
            AstExpr::GetNamedTable(name) => {
                let name = self.intern(name);
                let schema = match self.tables.get(&name) {
                    None => todo!("err, table not defined"),
                    Some(t) => t.schema.clone(),
                };
                (IrExpr::GetNamedTable(name), schema)
            }
            AstExpr::TakeFields { input, fields } => {
                let (input, prev) = self.compile_impure(*input);
                assert!(prev.names.len() >= fields.len());
                let names: Vec<_> = fields.iter().map(|name| self.intern(name)).collect();
                let sc = Schema {
                    types: names.iter().map(|&n| prev.get_type(n).unwrap()).collect(),
                    names,
                };
                let fields = fields
                    .into_iter()
                    .map(|name| {
                        let id = self.intern(name);
                        prev.get_index(id).expect("Field lookup")
                    })
                    .collect();

                (IrExpr::TakeFields(Box::new(input), fields), sc)
            }
            AstExpr::CreateNamedTable {
                name,
                headers,
                data,
            } => {
                let name = self.intern(name);
                let rows = self.load_table(data, headers.len());
                // TODO: err if no rows
                let schema = Schema {
                    names: headers.iter().map(|n| self.intern(n)).collect(),
                    types: rows
                        .first()
                        .expect("some rows so can infer types")
                        .values
                        .iter()
                        .map(|v| v.ty())
                        .collect(),
                };
                let table = Relation {
                    rows,
                    name,
                    schema: schema.clone(),
                };
                // TODO: check for stomping?
                self.tables.insert(name, table);
                (IrExpr::GetNamedTable(name), schema)
            }
            AstExpr::Filter { input, cond } => {
                let (prev, schema) = self.compile_impure(*input);
                let (cond, ty) = self.compile_pred(cond, &schema);
                assert_eq!(ty, VType::Bool, "filter predicate must be bool");
                (IrExpr::Filter(Box::new(prev), cond), schema)
            }
        }
    }

    fn compile_pred(&mut self, expr: AstPred, schema: &Schema) -> (IrPred, VType) {
        match expr {
            AstPred::Literal(v) => {
                let v = self.compile_value(v);
                (IrPred::Literal(v), v.ty())
            }
            AstPred::Field(name) => {
                let i = schema.get_index(self.intern(name)).unwrap(); // TODO
                (IrPred::Field(i), schema.types[i.0])
            }
            AstPred::Bin(op, lhs, rhs) => {
                let (l, lt) = self.compile_pred(*lhs, &schema);
                let (r, rt) = self.compile_pred(*rhs, &schema);
                assert_eq!(lt, rt);
                assert_eq!(op, BinOp::Eq);
                match op {
                    BinOp::Eq => (IrPred::Bin(op, Box::new(l), Box::new(r)), VType::Bool),
                }
            }
        }
    }

    // TODO: schema type checking
    fn load_table(&mut self, data: Vec<Vec<AstValue>>, count: usize) -> Vec<Row> {
        data.into_iter()
            .map(|row| {
                assert_eq!(count, row.len()); // TODO: result
                Row {
                    values: row
                        .into_iter()
                        .map(|value| self.compile_value(value))
                        .collect(),
                }
            })
            .collect()
    }

    fn compile_value(&mut self, value: AstValue) -> Value {
        match value {
            AstValue::Num(n) => Value::Num(n),
            AstValue::Str(s) => Value::Str(self.intern(s)),
            AstValue::Bool(b) => Value::Bool(b),
        }
    }

    pub fn get_str(&self, e: StrId) -> &str {
        &self.strings[e.0]
    }

    pub fn intern(&mut self, e: &str) -> StrId {
        match self.string_cache.get(e) {
            None => {
                let id = StrId(self.strings.len());
                let s: Rc<str> = e.into();
                self.string_cache.insert(s.clone(), id);
                self.strings.push(s);
                id
            }
            Some(&id) => id,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct StrId(usize);

impl<'a> RowsIter<'a> {
    pub fn new(f: impl Iterator<Item = Row> + 'a) -> Self {
        Self(Box::new(f))
    }

    // This is awkward to call cause you can't add a name while the iter references the vm. TODO: make name optional
    pub fn collect(self, name: StrId, schema: Schema) -> Relation {
        Relation {
            name,
            schema,
            rows: self.0.collect(),
        }
    }
}
