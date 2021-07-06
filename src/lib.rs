mod assemble;
mod machine;
mod parser;

use assemble::assemble;
use machine::Operations;
use machine::{
    errors::{MResult, MachineError},
    Machine,
};

/// Constructs and returns a model of the machine with
/// the given registers, operations, and controller.
pub fn make_machine<F, Args, R>(
    register_names: Vec<&str>,
    operations: &Operations,
    controller_text: &str,
) -> MResult<Machine> {
    let mut machine = Machine::new();
    for &reg_name in register_names.iter() {
        machine.allocate_register(reg_name)?;
    }
    machine.install_operations(operations);
    machine.install_instructions(
        assemble(controller_text).map_err(|msg: String| MachineError::UnableAssemble(msg))?,
    );
    Ok(machine)
}
