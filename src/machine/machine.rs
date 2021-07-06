//! The register machine

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use super::{
    errors::{MResult, MachineError, OperationError, RegisterError},
    function::Function,
    operation::Operation,
    register::Register,
    stack::Stack,
    value::{CompoundValue, FromValueList, Value},
};
use crate::parser::RMLNode;

pub struct Machine {
    pc: Register,
    flag: Register,
    stack: Stack,
    the_inst_seq: Vec<RMLNode>,
    the_labels: HashMap<String, Vec<RMLNode>>,
    the_ops: HashMap<String, Operation>,
    register_table: HashMap<String, Register>,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            pc: Register::new(),
            flag: Register::new(),
            stack: Stack::new(),
            the_inst_seq: Vec::new(),
            the_labels: HashMap::new(),
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

    pub fn install_operation<F, Args, R, S>(&mut self, name: S, f: F)
    where
        F: Function<Args, Result = R>,
        Args: FromValueList,
        R: Debug + PartialEq + Send + Sync + 'static,
        S: Into<String>,
    {
        self.the_ops.insert(name.into(), Operation::new(f));
    }

    pub fn install_operations(&mut self, operations: &HashMap<&str, Operation>) {
        self.the_ops.extend(
            operations
                .into_iter()
                .map(|(&name, op)| (name.to_string(), op.clone())),
        );
    }

    pub fn allocate_register<S: Into<String>>(&mut self, name: S) -> MResult<&'static str> {
        let name = name.into();
        if name.eq("pc") && name.eq("flag") && self.register_table.contains_key(&name) {
            Err(RegisterError::AllocateFailure(name))?
        } else {
            self.register_table.insert(name, Register::new());
            Ok("register-allocated")
        }
    }

    pub fn total_registers(&self) -> usize {
        self.register_table.len() + 2
    }

    pub fn total_operations(&self) -> usize {
        self.the_ops.len() + 2
    }

    pub fn call_operation<S: Into<String>>(&mut self, name: S, args: Vec<Value>) -> MResult<Value> {
        let name = name.into();
        let res = Ok(Value::new("Done".to_string()));
        match name.as_str() {
            "initialize-stack" => {
                self.initialize_stack();
                res
            }
            "print-stack-statistics" => {
                self.print_stack_statistics();
                res
            }
            _ => self.the_ops.get(&name).map_or_else(
                || Err(OperationError::NotFound(name).into()),
                |op| op.perform(args),
            ),
        }
    }

    pub fn stack(&self) -> &Stack {
        &self.stack
    }

    pub fn operations(&self) -> &HashMap<String, Operation> {
        &self.the_ops
    }

    pub fn install_instructions(&mut self, insts: Vec<RMLNode>) {
        self.the_inst_seq = insts;
    }

    pub fn install_labels(&mut self, labels: HashMap<String, Vec<RMLNode>>) {
        self.the_labels = labels;
    }

    pub fn start(&mut self) -> MResult<&'static str> {
        self.reset_pc();
        self.execute()
    }

    pub fn execute(&mut self) -> MResult<&'static str> {
        if let Value::Pointer(pointer) = *self.pc.get() {
            if pointer == self.the_inst_seq.len() {
                return Ok("Done");
            } else if pointer > self.the_inst_seq.len() {
                return Err(MachineError::NoMoreInsts);
            }
            match self.the_inst_seq[pointer].clone() {
                RMLNode::Assignment(reg_name, op) => self.execute_assignment(reg_name, op),
                RMLNode::Branch(label) => self.execute_branch(label),
                RMLNode::GotoLabel(label) => self.execute_goto(label),
                RMLNode::PerformOp(op) => self.execute_perform(op),
                RMLNode::RestoreFrom(reg_name) => self.execute_restore(reg_name),
                RMLNode::SaveTo(reg_name) => self.execute_save(reg_name),
                RMLNode::TestOp(op) => self.execute_test(op),
                _ => unreachable!(),
            }
        } else {
            Err(RegisterError::UnmatchedContentType {
                reg_name: "pc".to_string(),
                type_name: "usize".to_string(),
            })?
        }
    }

    fn advance_pc(&mut self) -> MResult<&'static str> {
        if let Value::Pointer(value) = *self.pc.get() {
            self.pc.set(Value::Pointer(value + 1));
            Ok("Done")
        } else {
            Err(RegisterError::UnmatchedContentType {
                reg_name: "pc".to_string(),
                type_name: "usize".to_string(),
            })?
        }
    }

    fn reset_pc(&mut self) {
        self.pc.set(Value::Pointer(0));
    }

    fn get_register_content(&self, reg_name: &String) -> MResult<Value> {
        if let Some(reg) = self.register_table.get(reg_name) {
            Ok(reg.get().clone())
        } else {
            Err(RegisterError::LookupFailure(reg_name.to_string()))?
        }
    }

    fn set_register_content(&mut self, reg_name: &String, value: Value) -> MResult<&'static str> {
        if let Some(reg) = self.register_table.get_mut(reg_name) {
            reg.set(value);
            Ok("Done")
        } else {
            Err(RegisterError::LookupFailure(reg_name.to_string()))?
        }
    }

    fn execute_assignment(
        &mut self,
        reg_name: String,
        operation: Arc<RMLNode>,
    ) -> MResult<&'static str> {
        match &*operation {
            RMLNode::Reg(name) => {
                let value = self.get_register_content(name)?;
                self.set_register_content(&reg_name, value)?;
            }
            RMLNode::Num(n) => {
                self.set_register_content(&reg_name, Value::Integer(*n))?;
            }
            RMLNode::Label(s) | RMLNode::Str(s) | RMLNode::Symbol(s) => {
                self.set_register_content(&reg_name, Value::String(s.to_string()))?;
            }
            RMLNode::List(l) => {
                self.set_register_content(
                    &reg_name,
                    Value::Compound(CompoundValue::new(l.clone())),
                )?;
            }
            RMLNode::Operation(op_name, args) => {
                let value = self.execute_operation(&op_name, &args)?;
                self.set_register_content(&reg_name, value)?;
            }
            _ => unreachable!(),
        }
        self.advance_pc()
    }

    fn extract_label_name(&self, label: Arc<RMLNode>) -> MResult<String> {
        match &*label {
            RMLNode::Reg(reg_name) => {
                let value = self.get_register_content(reg_name)?;
                if let Value::String(label) = value {
                    Ok(label.to_string())
                } else {
                    Err(RegisterError::UnmatchedContentType {
                        reg_name: reg_name.to_string(),
                        type_name: "String".into(),
                    })?
                }
            }
            RMLNode::Label(label_name) => Ok(label_name.to_string()),
            _ => unreachable!(),
        }
    }

    fn execute_branch(&mut self, label: Arc<RMLNode>) -> MResult<&'static str> {
        let label_name = self.extract_label_name(label)?;
        if let Some(insts) = self.the_labels.get(&label_name) {
            if let Value::Boolean(true) = self.flag.get() {
                self.the_inst_seq = insts.clone();
                self.reset_pc();
                Ok("Done")
            } else {
                self.advance_pc()
            }
        } else {
            Err(MachineError::UnknownLabel(label_name))
        }
    }

    fn execute_goto(&mut self, label: Arc<RMLNode>) -> MResult<&'static str> {
        let label_name = self.extract_label_name(label)?;
        if let Some(insts) = self.the_labels.get(&label_name) {
            self.the_inst_seq = insts.clone();
            self.reset_pc();
            Ok("Done")
        } else {
            Err(MachineError::UnknownLabel(label_name))
        }
    }

    fn execute_perform(&self, operation: Arc<RMLNode>) -> MResult<&'static str> {
        unimplemented!()
    }

    fn execute_restore(&mut self, reg_name: String) -> MResult<&'static str> {
        let value = self
            .stack
            .pop()
            .map_err(|s: &str| MachineError::StackError(s.to_string()))?;
        self.set_register_content(&reg_name, value)?;
        self.advance_pc()
    }

    fn execute_save(&mut self, reg_name: String) -> MResult<&'static str> {
        let value = self.get_register_content(&reg_name)?;
        self.stack.push(value);
        self.advance_pc()
    }

    fn execute_test(&mut self, operation: Arc<RMLNode>) -> MResult<&'static str> {
        unimplemented!()
    }

    fn execute_operation(&self, op_name: &String, args: &Vec<RMLNode>) -> MResult<Value> {
        unimplemented!()
    }
}

#[cfg(test)]
mod machine_tests {
    use super::*;

    #[test]
    fn test_make_new_machine() {
        let m = Machine::new();
        assert!(m.stack.is_empty());
        assert_eq!(m.total_registers(), 2);
        assert_eq!(m.total_operations(), 2);
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
        let expected = Value::new("Done".to_string());
        let mut m = Machine::new();
        let res = m.call_operation("print-stack-statistics", vec![]);
        assert!(res.is_ok());
        assert_eq!(expected, res.unwrap());

        let res = m.call_operation("initialize-stack", vec![]);
        assert!(res.is_ok());
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_install_operation() {
        let mut m = Machine::new();
        m.install_operation("add", |a: i32, b: i32| a + b);
        let res = m.call_operation("add", vec![Value::new(1), Value::new(1)]);
        assert!(res.is_ok());
        assert_eq!(Ok(Value::new(2)), res);
    }

    #[test]
    fn test_install_operations() {
        let mut operations: HashMap<&str, Operation> = HashMap::new();
        operations.insert("add", Operation::new(|a: i32, b: i32| a + b));
        operations.insert("sub", Operation::new(|a: i32, b: i32| a - b));
        operations.insert("mut", Operation::new(|a: i32, b: i32| a * b));
        operations.insert("div", Operation::new(|a: i32, b: i32| a / b));

        let mut m = Machine::new();
        m.install_operations(&operations);

        let res = m.call_operation("add", vec![Value::new(1), Value::new(1)]);
        assert!(res.is_ok());
        assert_eq!(Ok(Value::new(2)), res);
        let res = m.call_operation("sub", vec![Value::new(1), Value::new(1)]);
        assert!(res.is_ok());
        assert_eq!(Ok(Value::new(0)), res);
        let res = m.call_operation("mut", vec![Value::new(1), Value::new(1)]);
        assert!(res.is_ok());
        assert_eq!(Ok(Value::new(1)), res);
        let res = m.call_operation("div", vec![Value::new(1), Value::new(1)]);
        assert!(res.is_ok());
        assert_eq!(Ok(Value::new(1)), res);
    }

    #[test]
    fn test_start_method() {
        let mut m = Machine::new();
        let res = m.start();
        assert_eq!(Ok("Done"), res);
    }

    #[test]
    fn test_advance_pc() {
        let mut m = Machine::new();
        m.pc.set(Value::Pointer(0));
        let res = m.advance_pc();
        assert_eq!(Ok("Done"), res);
        let actual = m.pc.get();
        assert_eq!(Value::Pointer(1), *actual);
    }

    #[test]
    fn test_manipulate_register_content() {
        let mut m = Machine::new();
        let name = String::from("test");
        let res = m.allocate_register(&name);
        assert_eq!(Ok("register-allocated"), res);

        let actual = m.get_register_content(&name);
        assert_eq!(Ok(Value::String("*unassigned*".to_string())), actual);
        let res = m.set_register_content(&name, Value::Integer(1));
        assert_eq!(Ok("Done"), res);
        let actual = m.get_register_content(&name);
        assert_eq!(Ok(Value::Integer(1)), actual);
    }
}
