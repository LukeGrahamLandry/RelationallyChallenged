use crate::ir::{
    AstExpr, AstPred, AstValue, BinOp, IrExpr, IrPred, Loc, Relation, Row, Schema, TableBinOp,
    VType, Value,
};
use std::collections::{HashMap, HashSet};
use std::iter::from_fn;
use std::rc::Rc;

// TODO: you can never remove strings from the cache to reclaim memory because someone might have a StrId referencing it.
#[derive(Default, Debug, Clone)]
pub struct Vm {
    strings: Vec<Rc<str>>,
    string_cache: HashMap<Rc<str>, StrId>,
    pub tables: HashMap<StrId, Relation>,
}

// TODO: need to include context
pub type Res<T> = Result<T, CompErr>;
#[derive(Debug, Clone)]
pub enum CompErr {
    TypeMismatch(VType, VType),
    BadFieldName(StrId, Schema),
    SchemaMismatch(Schema, Schema),
    BadTableName(StrId),
    EmptyTable,
}

// TODO: maybe row should be a cow or better would be wrap the whole thing in an enum so you switch over borrowed vs owned once when creating the closure, not each row when running it
pub struct RowsIter<'a>(pub Box<dyn Iterator<Item = Row> + 'a>);

impl Vm {
    // TODO: if i was feeling fancy, the Vm could have a pool of row vecs and impl drop to return them
    //       so you dont have to do an allocation for every row. (same for the closure boxes but that's harder to think about)
    /// The expr must have been compiled on THIS vm instance.
    #[must_use = "RowsIter is lazy and must be polled."]
    pub fn eval<'a>(&'a self, expr: &'a IrExpr) -> RowsIter<'a> {
        match expr {
            IrExpr::GetNamedTable(name) => self
                .tables
                .get(name)
                .expect("ICE: eval(GetNamedTable)")
                .iter(),
            IrExpr::TakeFields(prev, fields) => {
                let prev = self.eval(prev);
                RowsIter::new(prev.0.map(|row| Row {
                    // TODO: could filter and reorder the existing vec?
                    values: fields.iter().map(|i| row.values[i.0]).collect(),
                }))
            }
            IrExpr::Filter(prev, pred) => {
                let pred = self.eval_pred(pred);
                let prev = self.eval(prev);
                RowsIter::new(prev.0.filter(move |row| pred(row).as_bool()))
            }
            IrExpr::Bin { op, lhs, rhs } => {
                let lhs = self.eval(lhs);
                let rhs = self.eval(rhs);
                match op {
                    TableBinOp::Union => RowsIter::new(lazy_init(move || {
                        // TODO: sad day that i have to allocate the whole thing but idk how else you'd do this
                        let all: HashSet<_> = lhs.0.chain(rhs.0).collect();
                        Box::new(all.into_iter())
                    })),
                    TableBinOp::Intersection => RowsIter::new(lazy_init(move || {
                        // TODO: use size_hint and pick the smaller one to be fully allocated instead of arbitrarily left
                        let left: HashSet<_> = lhs.0.collect();
                        if left.is_empty() {
                            return Box::new(left.into_iter());
                        }
                        Box::new(rhs.0.filter(move |row| left.contains(&row)))
                    })),
                    TableBinOp::Difference => RowsIter::new(lazy_init(move || {
                        let right: HashSet<_> = rhs.0.collect();
                        if right.is_empty() {
                            return lhs.0; // dont remove anything
                        }
                        Box::new(lhs.0.filter(move |row| !right.contains(&row)))
                    })),
                    TableBinOp::Product => RowsIter::new(lazy_init(move || {
                        // TODO: use size_hint, choose smaller
                        let right: Vec<_> = rhs.0.collect();
                        Box::new(lhs.0.flat_map(move |a| {
                            let right = right.clone();
                            right.into_iter().map(move |b| {
                                let mut both = a.clone();
                                both.values.extend(b.values.into_iter());
                                both
                            })
                        }))
                    })),
                }
            }
        }
    }

    fn eval_pred<'a>(&self, pred: &'a IrPred) -> Box<dyn Fn(&Row) -> Value + 'a> {
        match pred {
            IrPred::Literal(lit) => Box::new(|_| *lit),
            IrPred::Field(f) => Box::new(|row| row.values[f.0]),
            IrPred::Bin(op, lhs, rhs) => {
                let lhs = self.eval_pred(lhs);
                let rhs = self.eval_pred(rhs);
                // the Value.as___() methods crash on type error but compile_impure would have caught it first.
                match op {
                    // TODO: this handles nans wrong
                    BinOp::Eq => Box::new(move |row| Value::Bool(lhs(row) == rhs(row))),
                    BinOp::Or => {
                        Box::new(move |row| Value::Bool(lhs(row).as_bool() || rhs(row).as_bool()))
                    }
                    BinOp::And => {
                        Box::new(move |row| Value::Bool(lhs(row).as_bool() && rhs(row).as_bool()))
                    }
                    BinOp::Add => {
                        Box::new(move |row| Value::Num(lhs(row).as_num() + rhs(row).as_num()))
                    }
                    BinOp::Sub => {
                        Box::new(move |row| Value::Num(lhs(row).as_num() - rhs(row).as_num()))
                    }
                    BinOp::Mul => {
                        Box::new(move |row| Value::Num(lhs(row).as_num() * rhs(row).as_num()))
                    }
                    // TODO: this handles nans wrong
                    BinOp::Div => {
                        Box::new(move |row| Value::Num(lhs(row).as_num() / rhs(row).as_num()))
                    }
                    BinOp::Gt => {
                        Box::new(move |row| Value::Bool(lhs(row).as_num() > rhs(row).as_num()))
                    }
                    BinOp::Lt => {
                        Box::new(move |row| Value::Bool(lhs(row).as_num() < rhs(row).as_num()))
                    }
                }
            }
        }
    }

    /// Preprocess and type check an ast.
    /// The result is only valid if eval-ed on THIS vm instance. TODO: enforce with lifetime
    /// Note: CreateNamedTable runs NOW, not during eval.
    pub fn compile_impure(&mut self, (expr, _): Loc<AstExpr>) -> Res<(IrExpr, Schema)> {
        Ok(match expr {
            AstExpr::GetNamedTable(name) => {
                let name = self.intern(name);
                let schema = match self.tables.get(&name) {
                    None => return Err(CompErr::BadTableName(name)),
                    Some(t) => t.schema.clone(),
                };
                (IrExpr::GetNamedTable(name), schema)
            }
            AstExpr::TakeFields { input, fields } => {
                let (input, prev) = self.compile_impure(*input)?;
                assert!(prev.names.len() >= fields.len());
                let names: Vec<_> = fields.iter().map(|name| self.intern(name)).collect();
                let types: Res<Vec<_>> = names.iter().map(|&n| prev.get_type(n)).collect();
                let sc = Schema {
                    types: types?,
                    names,
                };
                let fields: Res<Vec<_>> = fields
                    .into_iter()
                    .map(|name| {
                        let id = self.intern(name);
                        prev.get_index(id)
                    })
                    .collect();

                (IrExpr::TakeFields(Box::new(input), fields?), sc)
            }
            AstExpr::CreateNamedTable {
                name,
                headers,
                data,
            } => {
                let name = self.intern(name);
                // TODO: require no dups? currently only ensuring that in set op output
                let rows = self.load_table(data, headers.len());
                let schema = Schema {
                    names: headers.iter().map(|n| self.intern(n)).collect(),
                    types: rows
                        .first()
                        .ok_or(CompErr::EmptyTable)?
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
                let (prev, schema) = self.compile_impure(*input)?;
                let (cond, ty) = self.compile_pred(cond, &schema)?;
                if ty != VType::Bool {
                    return Err(CompErr::TypeMismatch(ty, VType::Bool));
                }
                (IrExpr::Filter(Box::new(prev), cond), schema)
            }
            AstExpr::Bin { op, lhs, rhs } => {
                let (lhs, mut lt) = self.compile_impure(*lhs)?;
                let (rhs, rt) = self.compile_impure(*rhs)?;
                let expr = IrExpr::Bin {
                    op,
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                };

                let schema = match op {
                    TableBinOp::Union | TableBinOp::Intersection | TableBinOp::Difference => {
                        // TODO: only require same types. but which names get used?
                        if lt != rt {
                            return Err(CompErr::SchemaMismatch(lt, rt));
                        }
                        lt
                    }
                    TableBinOp::Product => {
                        // This one's special, it changes the schema.
                        // TODO: how are duplicate names dealt with? namespacing?
                        lt.names.extend(rt.names.into_iter());
                        lt.types.extend(rt.types.into_iter());
                        lt
                    }
                };

                (expr, schema)
            }
        })
    }

    fn compile_pred(&mut self, expr: AstPred, schema: &Schema) -> Res<(IrPred, VType)> {
        Ok(match expr {
            AstPred::Literal(v) => {
                let v = self.compile_value(v);
                (IrPred::Literal(v), v.ty())
            }
            AstPred::Field(name) => {
                let name = self.intern(name);
                let i = schema.get_index(name)?;
                (IrPred::Field(i), schema.types[i.0])
            }
            AstPred::Bin(op, lhs, rhs) => {
                let (l, lt) = self.compile_pred(*lhs, &schema)?;
                let (r, rt) = self.compile_pred(*rhs, &schema)?;
                if lt != rt {
                    return Err(CompErr::TypeMismatch(lt, rt));
                }
                let output = match op {
                    BinOp::Eq => VType::Bool,
                    BinOp::Or | BinOp::And => {
                        if lt != VType::Bool {
                            return Err(CompErr::TypeMismatch(lt, VType::Bool));
                        }
                        VType::Bool
                    }
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                        if lt != VType::Num {
                            return Err(CompErr::TypeMismatch(lt, VType::Num));
                        }
                        VType::Num
                    }
                    BinOp::Gt | BinOp::Lt => {
                        if lt != VType::Num {
                            return Err(CompErr::TypeMismatch(lt, VType::Num));
                        }
                        VType::Bool
                    }
                };
                (IrPred::Bin(op, Box::new(l), Box::new(r)), output)
            }
        })
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
            AstValue::Num(n) => {
                if n.is_nan() {
                    Value::Nan
                } else {
                    Value::Num(n)
                }
            }
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
pub struct StrId(pub(crate) usize);

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

fn lazy_init<'a, T: 'a>(
    f: impl FnOnce() -> Box<dyn Iterator<Item = T> + 'a> + 'a,
) -> impl Iterator<Item = T> + 'a {
    // Super ugly because I only want to drain lhs&rhs after the caller tries to poll
    // the iterator. Which is kinda pointless but I like the idea of eval being as lazy as possible.

    // This has to be an option because the iter is an FnMut so you can't consume lhs&rhs directly
    // because the compiler doesn't trust that I'll only do it once.
    // When you call take on the Option<FnOnce>, it replaces it with none so it doesn't actually move the variable out of the closure.
    let mut lazy = Some(f);

    // The first time this is called, compute the union and save that iterator.
    // Future calls will just poll from it.
    let mut cached = None;
    from_fn(move || match &mut cached {
        None => {
            cached = Some(lazy.take().unwrap()());
            cached.as_mut().unwrap().next()
        }
        Some(iter) => iter.next(),
    })
}
