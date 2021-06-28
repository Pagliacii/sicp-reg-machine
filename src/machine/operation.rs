//! Module to help with passing around functions of arbitrary parameters
//! ref: https://github.com/osohq/oso

use std::any::Any;
use std::fmt;
use std::sync::Arc;

use super::errors::{Result, TypeError};
use super::function::Function;
use super::value::{FromValueList, Value};
use super::BaseType;

/// Container for the executed result of a `Function`
#[derive(Clone)]
pub struct OpResult {
    inner: BaseType,
    /// The type name of the OpResult.
    inner_type_name: &'static str,
}

impl fmt::Debug for OpResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OpResult inner type: {}", self.inner_type_name)
    }
}

impl OpResult {
    pub fn new<T>(result: T) -> Self
    where
        T: Any + Send + Sync,
    {
        Self {
            inner: Arc::new(result),
            inner_type_name: std::any::type_name::<T>(),
        }
    }

    pub fn value(&self) -> BaseType {
        Arc::clone(&self.inner)
    }

    pub fn downcast_ref<T>(&self) -> Result<&T>
    where
        T: Any + Send + Sync,
    {
        self.inner.as_ref().downcast_ref().ok_or_else(|| {
            TypeError::expected(self.inner_type_name)
                .got(std::any::type_name::<T>())
                .into()
        })
    }
}

type TypeErasedFunction<R> = Arc<dyn Fn(Vec<Value>) -> Result<R> + Send + Sync>;
/// Container for a `Function` to be executed
#[derive(Clone)]
pub struct Operation(TypeErasedFunction<OpResult>);
impl Operation {
    pub fn new<Args, F>(f: F) -> Self
    where
        Args: FromValueList,
        F: Function<Args>,
        F::Result: Send + Sync + 'static,
    {
        Self(Arc::new(move |args: Vec<Value>| {
            Args::from_value_list(&args).map(|args| OpResult::new(f.invoke(args)))
        }))
    }

    /// Execute the inner function with parameters `args`
    pub fn perform(&self, args: Vec<Value>) -> Result<OpResult> {
        self.0(args)
    }
}

#[cfg(test)]
mod operation_mod_tests {
    use super::*;

    #[test]
    fn test_opresult_downcast() {
        let res = OpResult::new(1);
        assert_eq!(res.downcast_ref::<i32>().ok(), Some(&1));
        assert_eq!(
            res.downcast_ref::<f32>(),
            Err(TypeError::expected("i32").got("f32").into())
        );
    }

    #[test]
    fn test_opresult_value() {
        let res = OpResult::new(2);
        let expect = Arc::new(2);
        let actual = res.value();
        assert_eq!(expect, actual.downcast::<i32>().unwrap())
    }

    #[test]
    fn test_perform_operation() {
        let op = Operation::new(|| 1 + 1);
        let res = op.perform(vec![]);
        assert!(res.is_ok());
        let res = res.unwrap();
        assert_eq!(res.downcast_ref::<i32>(), Ok(&2));
    }
}
