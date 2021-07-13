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
use parser::RMLValue;

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
    machine.install_operation("print", |s: String| println!("{}", s));
    machine.install_operations(operations);
    let (insts, labels) =
        assemble(controller_text).map_err(|msg: String| MachineError::UnableAssemble(msg))?;
    machine.install_instructions(insts);
    machine.install_labels(labels);
    Ok(machine)
}

fn read_line_buffer() -> String {
    // Read one line of input buffer-style
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

pub fn rmlvalue_to_value(r: &RMLValue) -> Value {
    match r {
        RMLValue::Float(f) => Value::Float(*f),
        RMLValue::Num(n) => Value::Integer(*n),
        RMLValue::Str(s) | RMLValue::Symbol(s) => Value::String(s.to_string()),
        RMLValue::List(l) => Value::List(l.iter().map(rmlvalue_to_value).collect::<Vec<Value>>()),
    }
}
