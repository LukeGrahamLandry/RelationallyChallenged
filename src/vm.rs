use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Rc;
use crate::ir::{AstExpr, IrExpr, Loc, Relation, Row, Schema};

// TODO: you can never remove strings from the cache to reclaim memory because someone might have a StrId referencing it.
#[derive(Default, Debug, Clone)]
pub struct Vm {
    strings: Vec<Rc<str>>,
    string_cache: HashMap<Rc<str>, StrId>,
    pub tables: HashMap<StrId, Relation>,
}

// TODO: maybe row should also be a cow
pub struct RowsIter<'a>(pub Box<dyn Iterator<Item=Row> + 'a>, pub Cow<'a, Schema>);

impl Vm {
    // TODO: if i was feeling fancy, the Vm could have a pool of row vecs and impl drop to return them
    //       so you dont have to do an allocation for every row. (same for the closure boxes but that's harder to think about)
    /// If this ever returns None, its a bug in compile() so I don't care about error messages but I don't want to unwrap and crash.
    /// The expr must have been compiled on THIS vm instance.
    #[must_use = "RowsIter is lazy and must be polled."]
    pub fn eval<'a>(&'a self, expr: &'a IrExpr) -> Option<RowsIter<'a>> {
        Some(match expr {
            IrExpr::GetNamedTable(name) => self.tables.get(name)?.iter(),
            IrExpr::Projection(prev, fields) => {
                let prev = self.eval(prev)?;
                let schema = Schema {
                    names: fields.iter().map(|i| prev.1.names[i.0]).collect(),
                };
                RowsIter::new(prev.0.map(|row| Row {  // TODO: could filter and reorder the existing vec?
                    values: fields.iter().map(|i| row.values[i.0]).collect(),
                }), Cow::Owned(schema))
            }
        })
    }

    // TODO: result instead of assert/unwrap
    // TODO: infer type so you dont have to specify schema on projection
    /// Preprocess and type check an ast.
    /// The result is only valid if eval-ed on THIS vm instance.
    pub fn compile(&mut self, (expr, _): Loc<AstExpr>) -> IrExpr {
        match expr {
            AstExpr::GetNamedTable(name) => IrExpr::GetNamedTable(self.intern(name)),
            AstExpr::Projection { input, prev, fields } => {
                assert!(prev.names.len() >= fields.len());
                let input = self.compile(*input);
                let fields = fields.into_iter().map(|name| {
                    let id = self.intern(name);
                    prev.get_index(id).expect("Field lookup")
                }).collect();
                IrExpr::Projection(Box::new(input), fields)
            }
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
    pub fn new(f: impl Iterator<Item=Row> + 'a, schema: Cow<'a, Schema>) -> Self {
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
