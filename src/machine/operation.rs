//! Module to help with passing around functions of arbitrary parameters
//! ref: https://github.com/osohq/oso

use std::fmt::Debug;
use std::sync::Arc;

use super::errors::Result;
use super::function::Function;
use super::value::{FromValueList, Value};

type TypeErasedFunction<R> = Arc<dyn Fn(Vec<Value>) -> Result<R> + Send + Sync>;
/// Container for a `Function` to be executed
#[derive(Clone)]
pub struct Operation(TypeErasedFunction<Value>);
impl Operation {
    pub fn new<Args, F>(f: F) -> Self
    where
        Args: FromValueList,
        F: Function<Args>,
        F::Result: Debug + PartialEq + Send + Sync + 'static,
    {
        Self(Arc::new(move |args: Vec<Value>| {
            Args::from_value_list(&args).map(|args| Value::new(f.invoke(args)))
        }))
    }

    /// Execute the inner function with parameters `args`
    pub fn perform(&self, args: Vec<Value>) -> Result<Value> {
        self.0(args)
    }
}

#[cfg(test)]
mod operation_mod_tests {
    use super::*;

    #[test]
    fn test_perform_operation() {
        let op = Operation::new(|| 1 + 1);
        let res = op.perform(vec![]);
        assert!(res.is_ok());
        assert_eq!(Value::Integer(2), res.unwrap());
    }
}
