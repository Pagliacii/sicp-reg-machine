//! Module to help with passing around functions of arbitrary parameters
//! Inspired by [oso](https://github.com/osohq/oso/blob/e569e424d05b1fe9ff0c72b60e6248b79f3ced33/languages/rust/oso/src/host/class_method.rs#L35-L53).

use std::sync::Arc;

use super::errors::{MResult, ProcedureError};
use super::value::{ToValue, Value};

/// Procedure for a `Fn(Vec<Value>) -> MResult<Value>` to be executed
pub struct Procedure {
    name: String,
    proc: Arc<dyn Fn(Vec<Value>) -> Value + Send + Sync>,
    min_arg_num: usize,
}

impl Procedure {
    pub fn new<F, S, R>(name: S, num: usize, f: F) -> Self
    where
        F: Fn(Vec<Value>) -> R + Send + Sync + 'static,
        R: ToValue,
        S: Into<String>,
    {
        Self {
            name: name.into(),
            proc: Arc::new(move |args: Vec<Value>| f(args).to_value()),
            min_arg_num: num,
        }
    }

    pub fn duplicate<S: Into<String>>(src: &Self, name: S) -> Self {
        let mut duplicate = src.clone();
        duplicate.name = name.into();
        duplicate
    }

    /// Execute the inner function with parameters `args`
    pub fn execute(&self, args: Vec<Value>) -> MResult<Value> {
        if args.len() < self.min_arg_num {
            Err(ProcedureError::ArgsTooFew {
                name: self.get_name(),
                expected: self.min_arg_num,
                got: args.len(),
            })?
        } else {
            Ok((self.proc)(args))
        }
    }

    pub fn get_name(&self) -> String {
        self.name.to_string()
    }

    pub fn get_arg_num(&self) -> usize {
        self.min_arg_num
    }
}

impl PartialEq for Procedure {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.min_arg_num == other.min_arg_num
    }
}

impl Clone for Procedure {
    fn clone(&self) -> Self {
        Self {
            name: self.get_name(),
            proc: self.proc.clone(),
            min_arg_num: self.min_arg_num,
        }
    }
}

impl ToValue for Procedure {
    fn to_value(self) -> Value {
        Value::Procedure(self)
    }
}

#[macro_export]
macro_rules! make_proc {
    ( $name:literal, |_| { $exps:expr }) => {
        Procedure::new($name, 0, |_| $exps)
    };
    ( $name:literal, $num:literal, |$($arg_name:ident:$arg_type:ty),+| { $exps:expr } ) => {
        Procedure::new($name, $num, |args| {
            let mut idx = 0usize;
            $(
                let $arg_name: $arg_type = $crate::machine::value::TryFromValue::try_from(&args[idx]).unwrap();
                #[allow(unused_assignments)]
                {
                    idx += 1;
                }
            )+
            $exps
        })
    };
    ( $name:literal, |_| $exps:expr ) => {
        make_proc!($name, |_| { $exps })
    };
    ( $name:literal, $num:literal, |$($arg_name:ident:$arg_type:ty),+| $exps:expr ) => {
        make_proc!($name, $num, |$($arg_name:$arg_type),+| { $exps })
    }
}

#[cfg(test)]
mod procedure_tests {
    use super::*;

    #[test]
    fn test_procedure_constructor() {
        let proc = Procedure::new("test", 0, |_| Value::Num(1.0));
        let res = proc.execute(vec![]);
        assert_eq!(Ok(Value::Num(1.0)), res);
    }

    #[test]
    fn test_execute_procedure() {
        let proc = Procedure::new("add", 2, |args: Vec<Value>| {
            args[0].clone() + args[1].clone()
        });
        let res = proc.execute(vec![1.to_value(), 2.to_value()]);
        assert_eq!(Ok(3.to_value()), res);
    }

    #[test]
    fn test_procedure_macro() {
        let proc = make_proc!("test", |_| Value::Num(1.0));
        let res = proc.execute(vec![]);
        assert_eq!(Ok(1.to_value()), res);
        let proc = make_proc!("add", 2, |augend: i32, addend: i32| augend + addend);
        let res = proc.execute(vec![1.to_value(), 2.to_value()]);
        assert_eq!(Ok(3.to_value()), res);
    }
}
