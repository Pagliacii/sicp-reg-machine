mod errors;
mod function;
mod value;

mod machine;
mod operation;
mod register;
mod stack;

use std::collections::HashMap;

use errors::Result;
use machine::Machine;
use operation::Operation;

type BaseType = std::sync::Arc<dyn std::any::Any + Send + Sync>;

/// Constructs and returns a model of the machine with
/// the given registers, operations, and controller.
#[allow(dead_code)]
pub fn make_machine<F, Args, R>(
    register_names: Vec<&str>,
    operations: &HashMap<&str, Operation>,
    _controller_text: &str,
) -> Result<Machine> {
    let mut machine = Machine::new();
    for &reg_name in register_names.iter() {
        machine.allocate_register(reg_name)?;
    }
    machine.install_operations(operations);
    Ok(machine)
}
