use std::convert::{From, TryFrom};
use std::fmt;

use impl_trait_for_tuples::*;

use super::errors::{MachineError, Result, TypeError};

/// An enum of the possible value types that can be sent to an operation.
#[derive(Clone, Debug)]
pub enum Value {
    Integer(i32),
    Float(f64),
    String(String),
    Boolean(bool),
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
            (Value::Float(f1), Value::Float(f2)) => f1 == f2,
            (Value::Integer(i1), Value::Integer(i2)) => i1 == i2,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            _ => false,
        }
    }
}

impl From<Value> for String {
    fn from(item: Value) -> Self {
        match item {
            Value::Boolean(b) => format!("Boolean ({})", b),
            Value::Float(f) => format!("Float ({})", f),
            Value::Integer(i) => format!("Integer ({})", i),
            Value::String(s) => format!("String ({})", s),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Convert Value types to Rust types.
pub trait FromValue: Clone {
    fn from_value(val: Value) -> Result<Self>;
}

impl FromValue for Value {
    fn from_value(val: Value) -> Result<Self> {
        Ok(val)
    }
}

macro_rules! from_value_to {
    ( $src:tt $dst:ty ) => {
        impl FromValue for $dst {
            fn from_value(val: Value) -> Result<Self> {
                if let Value::$src(v) = val {
                    <$dst>::try_from(v.clone()).map_err(|_| {
                        TypeError::expected(stringify!($dst))
                            .got(v.to_string())
                            .into()
                    })
                } else {
                    Err(TypeError::expected(stringify!($dst)).got(val))?
                }
            }
        }
    };
}
from_value_to! { Integer i32 }
from_value_to! { Float f64 }
from_value_to! { Boolean bool }
from_value_to! { String String }

/// Convert Vec<Value> to designated types.
pub trait FromValueList {
    fn from_value_list(values: &[Value]) -> Result<Self>
    where
        Self: Sized;
}

/// Convert Vec<Value> to a Tuple type.
///
/// This `impl` use the `impl_for_tuples` macro to automatically support
/// a vector with zero element up to sixteen elements.
#[impl_for_tuples(16)]
#[tuple_types_custom_trait_bound(FromValue)]
impl FromValueList for Tuple {
    fn from_value_list(values: &[Value]) -> Result<Self> {
        let mut iter = values.iter();
        Ok((for_tuples!(
            #( Tuple::from_value(iter.next().ok_or(
                MachineError::ToTupleError
            )?.clone())? ),*
        )))
    }
}

#[cfg(test)]
mod value_mod_tests {
    use super::*;

    fn compare_value(v1: &Value, v2: &Value) {
        assert_eq!(v1, v1);
        assert_eq!(v2, v2);
        assert_ne!(v1, v2);
    }

    #[test]
    fn test_value_compare() {
        let a = Value::Integer(1);
        let b = Value::Integer(2);
        let c = Value::Float(3.0);
        let d = Value::Float(4.0);
        let e = Value::Boolean(true);
        let f = Value::Boolean(false);
        let g = Value::String(String::from("Hello"));
        let h = Value::String(String::from("World"));
        // Comparing Value::Integer tests
        compare_value(&a, &b);
        // Comparing Value::Float tests
        compare_value(&c, &d);
        // Comparing Value::Boolean tests
        compare_value(&e, &f);
        // Comparing Value::String tests
        compare_value(&g, &h);
        // Comparing Value::Integer and Value::Float
        assert_ne!(a, c);
        assert_ne!(d, b);
    }
}
