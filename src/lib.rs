mod assemble;
mod parser;

pub mod machine;

use assemble::assemble;
use machine::Operations;
use machine::{
    errors::{MResult, MachineError},
    Machine,
};

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
    machine.install_operations(operations);
    let (insts, labels) =
        assemble(controller_text).map_err(|msg: String| MachineError::UnableAssemble(msg))?;
    machine.install_instructions(insts);
    machine.install_labels(labels);
    Ok(machine)
}
