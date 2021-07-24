//! The register machine

use std::collections::HashMap;
use std::sync::Arc;

use log::{debug, info, trace, warn};

use super::{
    errors::{MResult, MachineError, ProcedureError, RegisterError, TypeError},
    procedure::Procedure,
    register::Register,
    stack::Stack,
    value::{values_to_str, ToValue, Value},
};
use crate::{parser::RMLNode, rmlvalue_to_value};

pub struct Machine {
    pc: Register,
    flag: Register,
    stack: Stack,
    the_inst_seq: Vec<RMLNode>,
    the_labels: HashMap<String, Vec<RMLNode>>,
    the_procedures: HashMap<String, Procedure>,
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
            the_procedures: HashMap::new(),
            register_table: HashMap::new(),
        }
    }

    fn initialize_stack(&mut self) {
        self.stack.initialize();
    }

    fn print_stack_statistics(&self) {
        self.stack.print_statistics();
    }

    pub fn install_procedure(&mut self, proc: Procedure) {
        self.the_procedures.insert(proc.get_name(), proc);
    }

    pub fn install_procedures(&mut self, procedures: &Vec<Procedure>) {
        self.the_procedures.extend(
            procedures
                .into_iter()
                .map(|proc| (proc.get_name(), proc.clone())),
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

    pub fn get_register_content<S: Into<String>>(&self, reg_name: S) -> MResult<Value> {
        trace!("get register content");
        let reg_name = reg_name.into();
        if let Some(reg) = self.register_table.get(&reg_name) {
            debug!("reg: {}, content: {}", reg_name, reg.get());
            Ok(reg.get())
        } else {
            warn!("unknown register: {}", reg_name);
            Err(RegisterError::LookupFailure(reg_name))?
        }
    }

    pub fn set_register_content<S, T>(&mut self, reg_name: S, value: T) -> MResult<&'static str>
    where
        S: Into<String>,
        T: ToValue + std::fmt::Display,
    {
        trace!("set register content");
        let reg_name = reg_name.into();
        if let Some(reg) = self.register_table.get_mut(&reg_name) {
            debug!("set reg: {} to val: {}", reg_name, value);
            reg.set(value.to_value());
            Ok("Done")
        } else {
            warn!("unknown register: {}", reg_name);
            Err(RegisterError::LookupFailure(reg_name))?
        }
    }

    pub fn total_registers(&self) -> usize {
        self.register_table.len() + 2
    }

    pub fn total_procedures(&self) -> usize {
        self.the_procedures.len() + 2
    }

    pub fn call_procedure<S: Into<String>>(&mut self, name: S, args: Vec<Value>) -> MResult<Value> {
        trace!("call a procedure");
        let name = name.into();
        let res = Ok(Value::new("Done".to_string()));
        match name.as_str() {
            "initialize-stack" => {
                debug!("call a builtin procedure: initialize-stack");
                self.initialize_stack();
                res
            }
            "print-stack-statistics" => {
                debug!("call a builtin procedure: print-stack-statistics");
                self.print_stack_statistics();
                res
            }
            _ => {
                debug!(
                    "call a procedure: {} with args: {}",
                    name,
                    values_to_str(&args)
                );
                self.the_procedures.get(&name).map_or_else(
                    || Err(ProcedureError::NotFound(name).into()),
                    |op| op.execute(args),
                )
            }
        }
    }

    pub fn stack(&self) -> &Stack {
        &self.stack
    }

    pub fn procedures(&self) -> &HashMap<String, Procedure> {
        &self.the_procedures
    }

    pub fn install_instructions(&mut self, insts: Vec<RMLNode>) {
        self.the_inst_seq = insts;
    }

    pub fn install_labels(&mut self, labels: HashMap<String, Vec<RMLNode>>) {
        self.the_labels = labels;
    }

    pub fn start(&mut self) -> MResult<&'static str> {
        trace!("start");
        info!("machine starting");
        self.reset_pc();
        self.execute()
    }

    pub fn execute(&mut self) -> MResult<&'static str> {
        trace!("execute instructions");
        loop {
            if let Value::Pointer(pointer) = self.pc.get() {
                debug!("current pc: {}", pointer);
                if pointer == self.the_inst_seq.len() {
                    info!("finished");
                    return Ok("Done");
                } else if pointer > self.the_inst_seq.len() {
                    warn!("no more instructions");
                    return Err(MachineError::NoMoreInsts);
                }
                debug!("current inst: {}", &self.the_inst_seq[pointer]);
                match self.the_inst_seq[pointer].clone() {
                    RMLNode::Assignment(reg_name, op) => self.execute_assignment(reg_name, op)?,
                    RMLNode::Branch(label) => self.execute_branch(label)?,
                    RMLNode::GotoLabel(label) => self.execute_goto(label)?,
                    RMLNode::PerformOp(op) => self.execute_perform(op)?,
                    RMLNode::Restore(reg_name) => self.execute_restore(reg_name)?,
                    RMLNode::Save(reg_name) => self.execute_save(reg_name)?,
                    RMLNode::TestOp(op) => self.execute_test(op)?,
                    _ => unreachable!(),
                };
            } else {
                warn!("unexpected type: {:?}", self.pc.get());
                return Err(RegisterError::UnmatchedContentType {
                    reg_name: "pc".to_string(),
                    type_name: "usize".to_string(),
                })?;
            }
        }
    }

    fn advance_pc(&mut self) -> MResult<&'static str> {
        trace!("increment the pc register");
        if let Value::Pointer(p) = self.pc.get() {
            self.pc.set(Value::Pointer(p + 1));
            debug!("new pc: {}", self.pc.get());
            Ok("Done")
        } else {
            warn!("unexpected type: {:?}", self.pc.get());
            Err(RegisterError::UnmatchedContentType {
                reg_name: "pc".to_string(),
                type_name: "usize".to_string(),
            })?
        }
    }

    fn reset_pc(&mut self) {
        trace!("reset the pc register");
        debug!("reset pc: {} to 0", self.pc.get());
        self.pc.set(Value::Pointer(0));
    }

    fn execute_assignment(
        &mut self,
        reg_name: String,
        operation: Arc<RMLNode>,
    ) -> MResult<&'static str> {
        trace!("assignment");
        match &*operation {
            RMLNode::Reg(name) => {
                debug!("assign reg: {} as reg: {}", &reg_name, name);
                self.get_register_content(name)
                    .and_then(|value| self.set_register_content(&reg_name, value))?;
            }
            RMLNode::Constant(r) => {
                debug!("assign reg: {} as val: {}", &reg_name, r);
                self.set_register_content(&reg_name, rmlvalue_to_value(r))?;
            }
            RMLNode::Label(s) | RMLNode::Symbol(s) => {
                debug!("assign reg: {} as symbol: {}", &reg_name, s);
                self.set_register_content(&reg_name, Value::Symbol(s.to_string()))?;
            }
            RMLNode::List(l) => {
                debug!("assign reg: {} as list: {:?}", &reg_name, l);
                self.set_register_content(
                    &reg_name,
                    Value::List(l.iter().map(rmlvalue_to_value).collect()),
                )?;
            }
            RMLNode::Operation(op_name, args) => {
                debug!(
                    "assign reg: {} as the result of operating op: {}",
                    &reg_name, op_name
                );
                self.perform_operation(op_name, args)
                    .and_then(|value| self.set_register_content(&reg_name, value))?;
            }
            _ => unreachable!(),
        }
        self.advance_pc()
    }

    fn extract_label_name(&self, label: Arc<RMLNode>) -> MResult<String> {
        trace!("extract label name");
        match &*label {
            RMLNode::Reg(reg_name) => {
                debug!("extract from a register: {}", reg_name);
                let value = self.get_register_content(reg_name)?;
                if let Value::Symbol(label) = value {
                    debug!("label: {}", label);
                    Ok(label)
                } else {
                    warn!("unexpected type: {}", value);
                    Err(RegisterError::UnmatchedContentType {
                        reg_name: reg_name.to_string(),
                        type_name: "Value::Symbol".into(),
                    })?
                }
            }
            RMLNode::Label(label_name) => {
                debug!("label: {}", label_name);
                Ok(label_name.to_string())
            }
            _ => unreachable!(),
        }
    }

    fn execute_branch(&mut self, label: Arc<RMLNode>) -> MResult<&'static str> {
        trace!("branch");
        let label_name = self.extract_label_name(label)?;
        if let Some(insts) = self.the_labels.get(&label_name) {
            if let Value::Boolean(true) = self.flag.get() {
                debug!("jump to {}", &label_name);
                self.the_inst_seq = insts.clone();
                self.reset_pc();
                Ok("Done")
            } else {
                debug!("don't jump, go on");
                self.advance_pc()
            }
        } else {
            warn!("unknown label: {}", &label_name);
            Err(MachineError::UnknownLabel(label_name))
        }
    }

    fn execute_goto(&mut self, label: Arc<RMLNode>) -> MResult<&'static str> {
        trace!("goto");
        let label_name = self.extract_label_name(label)?;
        if let Some(insts) = self.the_labels.get(&label_name) {
            debug!("go to label: {}", &label_name);
            self.the_inst_seq = insts.clone();
            self.reset_pc();
            Ok("Done")
        } else {
            warn!("unknown label: {}", &label_name);
            Err(MachineError::UnknownLabel(label_name))
        }
    }

    fn execute_perform(&mut self, operation: Arc<RMLNode>) -> MResult<&'static str> {
        trace!("perform");
        match &*operation {
            RMLNode::Operation(op_name, args) => {
                debug!("to be performed: {}", op_name);
                self.perform_operation(op_name, args).and_then(|v| {
                    debug!("performed result: {}", v);
                    self.advance_pc()
                })
            }
            _ => unreachable!(),
        }
    }

    fn execute_restore(&mut self, reg_name: String) -> MResult<&'static str> {
        trace!("restore");
        let value = self
            .stack
            .pop()
            .map_err(|s: &str| MachineError::StackError(s.to_string()))?;
        debug!("reg: {} restore to val: {}", reg_name, value);
        self.set_register_content(&reg_name, value)?;
        self.advance_pc()
    }

    fn execute_save(&mut self, reg_name: String) -> MResult<&'static str> {
        trace!("save");
        let value = self.get_register_content(&reg_name)?;
        debug!("reg: {}, value: {}, saved", reg_name, value);
        self.stack.push(value);
        self.advance_pc()
    }

    fn execute_test(&mut self, operation: Arc<RMLNode>) -> MResult<&'static str> {
        trace!("test");
        match &*operation {
            RMLNode::Operation(op_name, args) => {
                debug!("test op: {}", op_name);
                self.perform_operation(op_name, args).and_then(|value| {
                    debug!("test result: {}", value);
                    if value.is_bool() {
                        self.flag.set(value);
                        self.advance_pc()
                    } else {
                        warn!("unexpected type: {}", value);
                        Err(TypeError::expected("bool"))?
                    }
                })
            }
            _ => unreachable!(),
        }
    }

    fn perform_operation<S: Into<String>>(
        &mut self,
        op_name: S,
        args: &Vec<RMLNode>,
    ) -> MResult<Value> {
        trace!("perform an operation");
        let op_name = op_name.into();
        let mut op_args: Vec<Value> = vec![];
        for arg in args.iter() {
            match arg {
                RMLNode::Reg(r) => {
                    let value = self.get_register_content(r)?;
                    op_args.push(value);
                }
                RMLNode::Constant(value) => op_args.push(rmlvalue_to_value(value)),
                _ => unreachable!(),
            }
        }
        debug!(
            "op: {} performs with args: ({})",
            op_name,
            values_to_str(&op_args)
        );
        self.call_procedure(op_name, op_args)
    }
}

#[cfg(test)]
mod machine_tests {
    use super::*;
    use crate::make_proc;

    #[test]
    fn test_make_new_machine() {
        let m = Machine::new();
        assert!(m.stack.is_empty());
        assert_eq!(m.total_registers(), 2);
        assert_eq!(m.total_procedures(), 2);
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
    fn test_builtin_procedures() {
        let expected = Value::new("Done".to_string());
        let mut m = Machine::new();
        let res = m.call_procedure("print-stack-statistics", vec![]);
        assert!(res.is_ok());
        assert_eq!(expected, res.unwrap());

        let res = m.call_procedure("initialize-stack", vec![]);
        assert!(res.is_ok());
        assert_eq!(expected, res.unwrap());
    }

    #[test]
    fn test_install_procedure() {
        let mut m = Machine::new();
        m.install_procedure(make_proc!("add", 2, |a: i32, b: i32| a + b));
        let res = m.call_procedure("add", vec![Value::new(1), Value::new(1)]);
        assert_eq!(Ok(Value::new(2)), res);
    }

    #[test]
    fn test_install_procedures() {
        let mut procedures: Vec<Procedure> = vec![];
        procedures.push(make_proc!("add", 2, |a: i32, b: i32| a + b));
        procedures.push(make_proc!("sub", 2, |a: i32, b: i32| a - b));
        procedures.push(make_proc!("mul", 2, |a: i32, b: i32| a * b));
        procedures.push(make_proc!("div", 2, |a: i32, b: i32| a / b));

        let mut m = Machine::new();
        m.install_procedures(&procedures);

        let res = m.call_procedure("add", vec![Value::new(1), Value::new(1)]);
        assert_eq!(Ok(Value::new(2)), res);
        let res = m.call_procedure("sub", vec![Value::new(1), Value::new(1)]);
        assert_eq!(Ok(Value::new(0)), res);
        let res = m.call_procedure("mul", vec![Value::new(1), Value::new(1)]);
        assert_eq!(Ok(Value::new(1)), res);
        let res = m.call_procedure("div", vec![Value::new(1), Value::new(1)]);
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
        assert_eq!(Value::Pointer(1), actual);
    }

    #[test]
    fn test_manipulate_register_content() {
        let mut m = Machine::new();
        let name = String::from("test");
        let res = m.allocate_register(&name);
        assert_eq!(Ok("register-allocated"), res);

        let actual = m.get_register_content(&name);
        assert_eq!(Ok(Value::Symbol("*unassigned*".to_string())), actual);
        let res = m.set_register_content(&name, 1);
        assert_eq!(Ok("Done"), res);
        let actual = m.get_register_content(&name);
        assert_eq!(Ok(Value::Num(1.0)), actual);
    }
}
