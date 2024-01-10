use relations::ir::{AstExpr, Relation, Row, Schema, Value};
use relations::vm::Vm;

fn main() {
    let mut vm = Vm::default();
    let table = Relation {
        name: vm.intern("Numbers"),
        schema: Schema {
            names: vec![vm.intern("Value"), vm.intern("Name"), vm.intern("Square")],
        },
        rows: vec![
            Row {
                values: vec![
                    Value::Num(1.0),
                    Value::Str(vm.intern("One")),
                    Value::Num(1.0),
                ],
            },
            Row {
                values: vec![
                    Value::Num(2.0),
                    Value::Str(vm.intern("Two")),
                    Value::Num(4.0),
                ],
            },
            Row {
                values: vec![
                    Value::Num(3.0),
                    Value::Str(vm.intern("Three")),
                    Value::Num(9.0),
                ],
            },
        ],
    };
    println!("{}", vm.print_table(&table));
    let name = vm.intern("Result");
    let prev = table.schema.clone();
    vm.tables.insert(table.name, table);
    let query = vm.compile_impure((
        AstExpr::Projection {
            input: Box::new((AstExpr::GetNamedTable("Numbers"), 0..1)),
            prev,
            fields: vec!["Square", "Name"],
        },
        0..1,
    ));

    println!("{}", vm.print_rows_iter(vm.eval(&query).unwrap(), name));
}
