//! Module to help with passing around functions of arbitrary parameters
//! ref: https://github.com/osohq/oso

use std::sync::Arc;

use super::errors::MResult;
use super::function::Function;
use super::value::{FromValueList, ToValue, Value};

type TypeErasedFunction = Arc<dyn Fn(Vec<Value>) -> MResult<Value> + Send + Sync>;
/// Container for a `Function` to be executed
#[derive(Clone)]
pub struct Operation(TypeErasedFunction);
impl Operation {
    pub fn new<Args, F>(f: F) -> Self
    where
        Args: FromValueList,
        F: Function<Args>,
        F::Result: ToValue,
    {
        Self(Arc::new(move |args: Vec<Value>| {
            Args::from_value_list(&args).map(|args| f.invoke(args).to_value())
        }))
    }

    /// Execute the inner function with parameters `args`
    pub fn perform(&self, args: Vec<Value>) -> MResult<Value> {
        self.0(args)
    }
}

impl PartialEq for Operation {
    fn eq(&self, _: &Self) -> bool {
        false
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
        assert_eq!(Value::new(2), res.unwrap());
    }
}
