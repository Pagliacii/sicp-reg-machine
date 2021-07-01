//! The register machine

use std::collections::HashMap;
use std::fmt;

use super::errors::{MachineError, OperationError, RegisterError, Result};
use super::function::Function;
use super::operation::Operation;
use super::register::Register;
use super::stack::Stack;
use super::value::{FromValueList, Value};
use super::BaseType;

pub struct Machine {
    pc: Register,
    flag: Register,
    stack: Stack,
    the_inst_seq: Vec<String>,
    the_ops: HashMap<String, Operation>,
    register_table: HashMap<String, Register>,
}

impl Machine {
    fn new() -> Self {
        Self {
            pc: Register::new(),
            flag: Register::new(),
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
        R: Send + Sync + fmt::Debug + 'static,
        S: Into<String>,
    {
        self.the_ops.insert(name.into(), Operation::new(f));
    }

    fn allocate_register<S: Into<String>>(&mut self, name: S) -> Result<&'static str> {
        let name = name.into();
        if name.eq("pc") && name.eq("flag") && self.register_table.contains_key(&name) {
            Err(RegisterError::AllocateFailure(name))?
        } else {
            self.register_table.insert(name, Register::new());
            Ok("register-allocated")
        }
    }

    fn lookup_register<S: Into<String>>(&self, name: S) -> Result<BaseType> {
        let name = name.into();
        match name.as_str() {
            "pc" => Ok(self.pc.get()),
            "flag" => Ok(self.flag.get()),
            _ => {
                if let Some(v) = self.register_table.get(&name) {
                    Ok(v.get())
                } else {
                    Err(RegisterError::LookupFailure(name))?
                }
            }
        }
    }

    fn total_registers(&self) -> usize {
        self.register_table.len() + 2
    }

    fn total_operations(&self) -> usize {
        self.the_ops.len() + 2
    }

    fn total_instructions(&self) -> usize {
        self.the_inst_seq.len()
    }

    fn call_operation<S: Into<String>>(&mut self, name: S, args: Vec<Value>) -> Result<Value> {
        let name = name.into();
        let res = Ok(Value::new("done".to_string()));
        match name.as_str() {
            "initialize-stack" => {
                self.initialize_stack();
                res
            }
            "print-stack-statistics" => {
                self.print_stack_statistics();
                res
            }
            _ => self
                .the_ops
                .get(&name)
                .map(|op| Value::new(op.perform(args)))
                .ok_or(OperationError::NotFound(name).into()),
        }
    }

    pub fn stack(&self) -> &Stack {
        &self.stack
    }

    pub fn operations(&self) -> &HashMap<String, Operation> {
        &self.the_ops
    }

    pub fn execute(&mut self) -> Result<&'static str> {
        if let Ok(insts_string) = self.pc.get().downcast::<String>() {
            let insts: Vec<&str> = insts_string.as_str().split("\n").collect();
            if insts.is_empty() || insts[0] == "*unassigned*" {
                return Ok("done");
            }
            Ok("TODO")
        } else {
            Err(MachineError::UnrecognizedInsts)
        }
    }
}

#[cfg(test)]
mod machine_tests {
    use super::*;
    use crate::machine::errors::MachineError;

    #[test]
    fn test_make_new_machine() {
        let m = Machine::new();
        assert!(m.stack.is_empty());
        assert_eq!(m.total_registers(), 2);
        assert_eq!(m.total_operations(), 2);
        assert_eq!(m.total_instructions(), 0);
    }

    #[test]
    fn test_lookup_register() {
        let m = Machine::new();
        let expected = "*unassigned*".to_string();
        let actual = m.lookup_register("pc");
        assert!(actual.is_ok());
        let actual = actual.unwrap();
        assert_eq!(Some(&expected), actual.as_ref().downcast_ref::<String>());

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
        let mut m = Machine::new();
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
        let expected = Value::new("done".to_string());
        let mut m = Machine::new();
        let res = m.call_operation("print-stack-statistics", vec![]);
        assert!(res.is_ok());
        assert_eq!(expected, res.unwrap());

        let res = m.call_operation("initialize-stack", vec![]);
        assert!(res.is_ok());
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_execute_instructions() {
        let mut m = Machine::new();
        let res = m.execute();
        assert_eq!(Ok("done"), res);

        m.pc.set(1);
        let res = m.execute();
        assert_eq!(Err(MachineError::UnrecognizedInsts), res);

        m.pc.set("Some instructions".to_string());
        let res = m.execute();
        assert_eq!(Ok("TODO"), res);
    }
}
