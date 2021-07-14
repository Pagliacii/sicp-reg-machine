mod assemble;

pub mod machine;
pub mod parser;

use assemble::assemble;
use machine::Operations;
use machine::{
    errors::{MResult, MachineError},
    value::Value,
    Machine,
};
use parser::{rml_value, RMLValue};

/// Constructs and returns a model of the machine with
/// the given registers, operations, and controller.
pub fn make_machine(
    register_names: Vec<&str>,
    operations: &Operations,
    controller_text: &str,
) -> MResult<Machine> {
    let mut machine = Machine::new();
    for &reg_name in register_names.iter() {
        machine.allocate_register(reg_name)?;
    }
    // Provides a `read` operation to read inputs from user,
    // and a `print` operation to print outputs on the screen.
    machine.install_operation("read", read_line_buffer);
    machine.install_operation("print", |v: Value| match v {
        Value::String(s) => println!("{}", s),
        _ => println!("{}", v),
    });
    machine.install_operations(operations);
    let (insts, labels) =
        assemble(controller_text).map_err(|msg: String| MachineError::UnableAssemble(msg))?;
    machine.install_instructions(insts);
    machine.install_labels(labels);
    Ok(machine)
}

fn read_line_buffer() -> Value {
    // Read one line of input buffer-style
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let (_, values) = rml_value(input.trim()).unwrap();
    rmlvalue_to_value(&values)
}

pub fn rmlvalue_to_value(r: &RMLValue) -> Value {
    match r {
        RMLValue::Float(f) => Value::Num(*f),
        RMLValue::Num(n) => Value::Num(*n as f64),
        RMLValue::Str(s) => Value::String(s.to_string()),
        RMLValue::Symbol(s) => Value::Symbol(s.to_string()),
        RMLValue::List(l) => Value::List(l.iter().map(rmlvalue_to_value).collect::<Vec<Value>>()),
    }
}
