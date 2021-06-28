//! The register machine

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::errors::{OperationError, RegisterError, Result};
use super::function::Function;
use super::operation::Operation;
use super::register::Register;
use super::stack::Stack;
use super::value::FromValueList;
use super::BaseType;

pub struct Machine {
    stack: Stack,
    the_inst_seq: Vec<String>,
    the_ops: HashMap<String, Operation>,
    register_table: HashMap<String, Register>,
}

impl Machine {
    fn new() -> Self {
        Self {
            stack: Stack::new(),
            the_inst_seq: Vec::new(),
            the_ops: HashMap::new(),
            register_table: HashMap::new(),
        }
    }

    fn initialize_stack(&mut self) {
        self.stack.initialize();
    }

    fn print_stack_statistics(&self) {
        self.stack.print_statistics();
    }

    fn install_operation<F, Args, R, S>(&mut self, name: S, f: F)
    where
        F: Function<Args, Result = R>,
        Args: FromValueList,
        R: Send + Sync + 'static,
        S: Into<String>,
    {
        self.the_ops.insert(name.into(), Operation::new(f));
    }

    fn allocate_register<S: Into<String>>(&mut self, name: S) -> Result<&'static str> {
        let name = name.into();
        if self.register_table.contains_key(&name) {
            Err(RegisterError::AllocateFailure(name))?
        } else {
            self.register_table.insert(name, Register::new());
            Ok("register-allocated")
        }
    }

    fn lookup_register<S: Into<String>>(&self, name: S) -> Result<BaseType> {
        let name = name.into();
        if let Some(v) = self.register_table.get(&name) {
            Ok(v.get())
        } else {
            Err(RegisterError::LookupFailure(name))?
        }
    }

    fn total_registers(&self) -> usize {
        self.register_table.len()
    }

    fn total_operations(&self) -> usize {
        self.the_ops.len()
    }

    fn total_instructions(&self) -> usize {
        self.the_inst_seq.len()
    }

    fn get_operation<S: Into<String>>(&self, name: S) -> Result<&Operation> {
        let name = name.into();
        if let Some(op) = self.the_ops.get(&name) {
            Ok(op)
        } else {
            Err(OperationError::NotFound(name))?
        }
    }

    pub fn stack(&self) -> &Stack {
        &self.stack
    }

    pub fn operations(&self) -> &HashMap<String, Operation> {
        &self.the_ops
    }
}

pub fn make_new_machine() -> Result<Arc<Mutex<Machine>>> {
    let mut machine = Machine::new();
    machine.allocate_register("pc")?;
    machine.allocate_register("flag")?;
    let m = Arc::new(Mutex::new(machine));
    m.lock().unwrap().install_operation("initialize-stack", {
        let m = Arc::clone(&m);
        move || {
            let mut m = m.lock().unwrap();
            m.initialize_stack();
        }
    });
    m.lock()
        .unwrap()
        .install_operation("print-stack-statistics", {
            let m = Arc::clone(&m);
            move || {
                let m = m.lock().unwrap();
                m.print_stack_statistics();
            }
        });
    Ok(m)
}

#[cfg(test)]
mod machine_tests {
    use super::*;
    use crate::machine::errors::MachineError;

    #[test]
    fn test_make_new_machine() {
        let machine = make_new_machine().unwrap();
        let m = machine.lock().unwrap();
        assert!(m.stack.is_empty());
        assert_eq!(m.total_registers(), 2);
        assert_eq!(m.total_operations(), 2);
        assert_eq!(m.total_instructions(), 0);
    }

    #[test]
    fn test_lookup_register() {
        let machine = make_new_machine().unwrap();
        let m = machine.lock().unwrap();
        let expected = Arc::new("*unassigned*".to_string());
        let actual = m.lookup_register("pc");
        assert!(actual.is_ok());
        assert_eq!(expected, actual.unwrap().downcast::<String>().unwrap());

        match m.lookup_register("not-found") {
            Err(e) => assert_eq!(
                MachineError::RegisterError(RegisterError::LookupFailure("not-found".to_string())),
                e,
            ),
            _ => (),
        };
    }

    #[test]
    fn test_allocate_register() {
        let machine = make_new_machine().unwrap();
        let mut m = machine.lock().unwrap();
        let res = m.allocate_register("test");
        assert_eq!(res, Ok("register-allocated"));

        match m.allocate_register("test") {
            Err(e) => assert_eq!(
                MachineError::RegisterError(RegisterError::AllocateFailure("test".to_string())),
                e,
            ),
            _ => (),
        }
    }

    #[test]
    fn test_builtin_operations() {
        let machine = make_new_machine().unwrap();
        let m = machine.lock().unwrap();
        let print_stack_op = m.get_operation("print-stack-statistics").unwrap();
        let res = print_stack_op.perform(vec![]);
        assert!(res.is_ok());
        let initialize = m.get_operation("initialize-stack").unwrap();
        let res = initialize.perform(vec![]);
        assert!(res.is_ok());
        assert!(m.stack.is_empty());
    }
}
