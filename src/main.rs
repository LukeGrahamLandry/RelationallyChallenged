use crate::ir::{AstExpr, Relation, Row, Schema, Value, Vm};

mod ir;
mod vm;
mod print;

fn main() {
    let mut vm = Vm::default();
    let table = Relation {
        name: vm.add_str("Numbers"),
        schema: Schema {
            names: vec![vm.add_str("Value"), vm.add_str("Name"), vm.add_str("Square")],
        },
        rows: vec![
            Row {
                values: vec![Value::Num(1.0), Value::Str(vm.add_str("One")), Value::Num(1.0)],
            },
            Row {
                values: vec![Value::Num(2.0), Value::Str(vm.add_str("Two")), Value::Num(4.0)],
            },
            Row {
                values: vec![Value::Num(3.0), Value::Str(vm.add_str("Three")), Value::Num(9.0)],
            }
        ],
    };
    println!("{}", vm.print_table(&table));
    let name = vm.add_str("Result");
    let prev = table.schema.clone();
    vm.tables.insert(table.name, table);
    let query = vm.compile(AstExpr::Projection {
        input: Box::new(AstExpr::GetNamedTable("Numbers".into())),
        prev,
        fields: vec!["Square".into(), "Name".into()],
    });

    println!("{}", vm.print_rows_iter(vm.eval(&query), name));
}
