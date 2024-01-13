use crate::ir::{AstExpr, AstValue, IrExpr, Loc, Relation, Row, Schema, Value};
use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Rc;

// TODO: you can never remove strings from the cache to reclaim memory because someone might have a StrId referencing it.
#[derive(Default, Debug, Clone)]
pub struct Vm {
    strings: Vec<Rc<str>>,
    string_cache: HashMap<Rc<str>, StrId>,
    pub tables: HashMap<StrId, Relation>,
}

// TODO: maybe row should also be a cow
// TODO: pull schema up into IrExpr since want to use indices everywhere anyway.
pub struct RowsIter<'a>(pub Box<dyn Iterator<Item = Row> + 'a>, pub Cow<'a, Schema>);

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
                // TODO: why do i care about knowing the schema here?
                //       the goal should be compile_impure mapping everything to indices
                //       and eval should just trust that it works out.
                let schema = Schema {
                    names: fields.iter().map(|i| prev.1.names[i.0]).collect(),
                };
                RowsIter::new(
                    prev.0.map(|row| Row {
                        // TODO: could filter and reorder the existing vec?
                        values: fields.iter().map(|i| row.values[i.0]).collect(),
                    }),
                    Cow::Owned(schema),
                )
            }
        })
    }

    // TODO: result instead of assert/unwrap
    // TODO: infer type so you dont have to specify schema on projection
    /// Preprocess and type check an ast.
    /// The result is only valid if eval-ed on THIS vm instance. TODO: enforce with lifetime
    /// Note: CreateNamedTable runs NOW, not during eval.
    pub fn compile_impure(&mut self, (expr, _): Loc<AstExpr>) -> IrExpr {
        match expr {
            AstExpr::GetNamedTable(name) => IrExpr::GetNamedTable(self.intern(name)),
            AstExpr::TakeFields {
                input,
                prev,
                fields,
            } => {
                assert!(prev.names.len() >= fields.len());
                let input = self.compile_impure(*input);
                let fields = fields
                    .into_iter()
                    .map(|name| {
                        let id = self.intern(name);
                        prev.get_index(id).expect("Field lookup")
                    })
                    .collect();
                IrExpr::TakeFields(Box::new(input), fields)
            }
            // TODO: untested
            AstExpr::CreateNamedTable {
                name,
                headers,
                data,
            } => {
                let name = self.intern(name);
                let schema = Schema {
                    names: headers.iter().map(|n| self.intern(n)).collect(),
                };
                let table = Relation {
                    rows: self.load_table(data, &schema),
                    name,
                    schema,
                };
                // TODO: check for stomping?
                self.tables.insert(name, table);
                IrExpr::GetNamedTable(name)
            }
        }
    }

    // TODO: schema type checking
    fn load_table(&mut self, data: Vec<Vec<AstValue>>, schema: &Schema) -> Vec<Row> {
        data.into_iter()
            .map(|row| {
                assert_eq!(schema.names.len(), row.len()); // TODO: result
                Row {
                    values: row
                        .into_iter()
                        .map(|value| match value {
                            AstValue::Num(n) => Value::Num(n),
                            AstValue::Str(s) => Value::Str(self.intern(s)),
                            AstValue::Bool(b) => Value::Bool(b),
                        })
                        .collect(),
                }
            })
            .collect()
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
    pub fn new(f: impl Iterator<Item = Row> + 'a, schema: Cow<'a, Schema>) -> Self {
        Self(Box::new(f), schema)
    }

    // This is awkward to call cause you can't add a name while the iter references the vm. TODO: make name optional
    pub fn collect(self, name: StrId) -> Relation {
        Relation {
            name,
            schema: self.1.into_owned(),
            rows: self.0.collect(),
        }
    }
}
