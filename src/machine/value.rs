use std::{any::Any, fmt};

use impl_trait_for_tuples::*;

use super::errors::{MResult, MachineError, TypeError};
use super::operation::Operation;

/// An enum of the possible value types that can be sent to an operation.
#[derive(Clone, PartialEq)]
pub enum Value {
    Num(f64),
    Symbol(String),
    String(String),
    Boolean(bool),
    List(Vec<Value>),
    Op(Operation),
    Unit,
    EnvPtr(usize),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Boolean(v) => write!(f, "<Boolean {}>", v),
            Value::Num(v) => write!(f, "<Num {}>", v),
            Value::List(v) => write!(f, "<List {:?}>", v.type_id()),
            Value::Symbol(v) => write!(f, "<Symbol {}>", v),
            Value::String(v) => write!(f, r#"<String "{}">"#, v),
            Value::Op(v) => write!(f, "<Operation {:?}>", v.type_id()),
            Value::EnvPtr(v) => write!(f, "<EnvPtr {}>", v),
            Value::Unit => write!(f, "<Unit>"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Boolean(v) => write!(f, "{}", v),
            Value::Num(v) => write!(f, "{}", v),
            Value::Symbol(v) => write!(f, "{}", v),
            Value::List(l) => write!(
                f,
                "({})",
                l.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Value::String(v) => write!(f, r#""{}""#, v),
            Value::Op(_) => write!(f, "Operation"),
            Value::EnvPtr(v) => write!(f, "EnvPtr {}", v),
            Value::Unit => write!(f, "()"),
        }
    }
}

impl Value {
    pub fn new<T: ToValue>(val: T) -> Self {
        val.to_value()
    }
}

pub trait ToValue: Sized {
    fn to_value(&self) -> Value;
}

impl ToValue for Value {
    fn to_value(&self) -> Value {
        self.clone()
    }
}

impl ToValue for i32 {
    fn to_value(&self) -> Value {
        Value::Num(*self as f64)
    }
}

impl ToValue for f64 {
    fn to_value(&self) -> Value {
        Value::Num(*self)
    }
}

impl ToValue for u64 {
    fn to_value(&self) -> Value {
        Value::Num(*self as f64)
    }
}

impl ToValue for usize {
    fn to_value(&self) -> Value {
        Value::Num(*self as f64)
    }
}

impl ToValue for bool {
    fn to_value(&self) -> Value {
        Value::Boolean(*self)
    }
}

impl ToValue for String {
    fn to_value(&self) -> Value {
        let string = self.to_string();
        if string.starts_with('"') {
            Value::String(string)
        } else {
            Value::Symbol(string)
        }
    }
}

impl ToValue for &dyn ToString {
    fn to_value(&self) -> Value {
        let string = self.to_string();
        if string.starts_with('"') {
            Value::String(string)
        } else {
            Value::Symbol(string)
        }
    }
}

impl ToValue for &'static str {
    fn to_value(&self) -> Value {
        let string = self.to_string();
        if string.starts_with('"') {
            Value::String(string)
        } else {
            Value::Symbol(string)
        }
    }
}

impl ToValue for Vec<Value> {
    fn to_value(&self) -> Value {
        Value::List(self.clone())
    }
}

impl ToValue for () {
    fn to_value(&self) -> Value {
        Value::Unit
    }
}

impl ToValue for Operation {
    fn to_value(&self) -> Value {
        Value::Op(self.clone())
    }
}

pub trait ToNumValue: ToValue {}
impl ToNumValue for i32 {}
impl ToNumValue for f64 {}
impl ToNumValue for u64 {}
impl ToNumValue for usize {}

pub trait TryFromValue: Sized {
    fn try_from(v: Value) -> Result<Self, TypeError>;
}

impl TryFromValue for Value {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        Ok(v)
    }
}

impl TryFromValue for i32 {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        let expected = TypeError::expected("Value::Num");
        match v {
            Value::Num(val) => Ok(val as i32),
            Value::Symbol(val) => val
                .parse::<i32>()
                .map_err(|_| expected.got(format!("Symbol {}", val))),
            _ => Err(expected.got(v.to_string())),
        }
    }
}

impl TryFromValue for f64 {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        let expected = TypeError::expected("Value::Num");
        match v {
            Value::Num(val) => Ok(val),
            Value::Symbol(val) => val
                .parse::<f64>()
                .map_err(|_| expected.got(format!("Symbol {}", val))),
            _ => Err(expected.got(v.to_string())),
        }
    }
}

impl TryFromValue for u64 {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        let expected = TypeError::expected("Value::Num");
        match v {
            Value::Num(val) => Ok(val as u64),
            Value::Symbol(val) => val
                .parse::<u64>()
                .map_err(|_| expected.got(format!("Symbol {}", val))),
            _ => Err(expected.got(v.to_string())),
        }
    }
}

impl TryFromValue for usize {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        let expected = TypeError::expected("Value::Num");
        match v {
            Value::Num(val) => Ok(val as usize),
            Value::EnvPtr(val) => Ok(val),
            Value::Symbol(val) => val
                .parse::<usize>()
                .map_err(|_| expected.got(format!("Symbol {}", val))),
            _ => Err(expected.got(v.to_string())),
        }
    }
}

impl TryFromValue for bool {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        let expected = TypeError::expected("Value::Boolean");
        match v {
            Value::Boolean(val) => Ok(val),
            Value::Symbol(val) => {
                if val == "true" {
                    Ok(true)
                } else if val == "false" {
                    Ok(false)
                } else {
                    Err(expected.got(format!("Symbol {}", val)))
                }
            }
            _ => Err(expected.got(v.to_string())),
        }
    }
}

impl TryFromValue for String {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        match v {
            Value::List(_) | Value::Op(_) => {
                Err(TypeError::expected("Variants compatible with String").got(v.to_string()))
            }
            _ => Ok(v.to_string()),
        }
    }
}

impl TryFromValue for Vec<Value> {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        if let Value::List(val) = v {
            Ok(val)
        } else {
            Err(TypeError::expected("Value::List").got(v.to_string()))
        }
    }
}

impl TryFromValue for Vec<i32> {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        if let Value::List(val) = v {
            val.iter().map(|v| i32::try_from(v.clone())).collect()
        } else {
            Err(TypeError::expected("Value::List").got(v.to_string()))
        }
    }
}

impl TryFromValue for Vec<f64> {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        if let Value::List(val) = v {
            val.iter().map(|v| f64::try_from(v.clone())).collect()
        } else {
            Err(TypeError::expected("Value::List").got(v.to_string()))
        }
    }
}

impl TryFromValue for Vec<u64> {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        if let Value::List(val) = v {
            val.iter().map(|v| u64::try_from(v.clone())).collect()
        } else {
            Err(TypeError::expected("Value::List").got(v.to_string()))
        }
    }
}

impl TryFromValue for Vec<usize> {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        if let Value::List(val) = v {
            val.iter().map(|v| usize::try_from(v.clone())).collect()
        } else {
            Err(TypeError::expected("Value::List").got(v.to_string()))
        }
    }
}

impl TryFromValue for () {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        if let Value::Unit = v {
            Ok(())
        } else {
            Err(TypeError::expected("Value::Unit").got(v.to_string()))
        }
    }
}

impl TryFromValue for Operation {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        if let Value::Op(op) = v {
            Ok(op)
        } else {
            Err(TypeError::expected("Value::Operation").got(v.to_string()))
        }
    }
}

/// Convert Value array to designated types.
pub trait FromValueList {
    fn from_value_list(values: &[Value]) -> MResult<Self>
    where
        Self: Sized;
}

/// Convert Vec<Value> to a Tuple type.
///
/// This `impl` use the `impl_for_tuples` macro to automatically support
/// a List with zero element up to sixteen elements.
#[impl_for_tuples(16)]
#[tuple_types_custom_trait_bound(TryFromValue)]
impl FromValueList for Tuple {
    fn from_value_list(values: &[Value]) -> MResult<Self> {
        let mut iter = values.iter();
        let result = Ok((for_tuples!(
            #( Tuple::try_from(iter.next().ok_or(
                MachineError::ToTupleError
            )?.clone())? ),*
        )));

        // I'm not sure that collecting the remaining items in this way is correct.
        if result.is_ok() && iter.len() > 0 {
            Ok((for_tuples!(
                #( Tuple::try_from(Value::new(values.to_vec()))? ),*
            )))
        } else {
            result
        }
    }
}

#[cfg(test)]
mod value_mod_tests {
    use super::*;

    #[test]
    fn test_value_constructor() {
        assert_eq!(Value::Num(1.0), Value::new(1));
        assert_eq!(Value::Num(1.0), Value::new(1.0));
        assert_eq!(Value::Num(1.0), Value::new(1u64));
        assert_eq!(Value::Num(1.0), Value::new(1usize));
        assert_eq!(Value::Boolean(true), Value::new(true));
        assert_eq!(Value::Boolean(false), Value::new(false));
        assert_eq!(Value::Symbol("test".into()), Value::new("test"));
        assert_eq!(Value::String("\"test\"".into()), Value::new(r#""test""#));
        assert_eq!(
            Value::Symbol("test".into()),
            Value::new(String::from("test"))
        );
        assert_eq!(Value::List(Vec::<Value>::new()), Value::new(vec![]));
        assert_eq!(Value::Unit, Value::new(()));
    }

    #[test]
    fn test_try_from_value() {
        assert_eq!(Ok(1), i32::try_from(Value::new(1)));
        assert_eq!(Ok(1.0), f64::try_from(Value::new(1.0)));
        assert_eq!(Ok(2), u64::try_from(Value::new(2u64)));
        assert_eq!(Ok(3), usize::try_from(Value::new(3usize)));
        assert_eq!(Ok(false), bool::try_from(Value::new(false)));
        assert_eq!(Ok("test".to_string()), String::try_from(Value::new("test")));
        assert_eq!(
            Ok(Vec::<Value>::new()),
            Vec::<Value>::try_from(Value::new(vec![]))
        );
        assert_eq!(
            Ok(vec![1, 2, 3]),
            Vec::<i32>::try_from(Value::new(vec![
                Value::Num(1.0),
                Value::Num(2.0),
                Value::Num(3.0)
            ]))
        );
        assert_eq!(
            Ok(vec![1.0, 2.0, 3.0]),
            Vec::<f64>::try_from(Value::new(vec![
                Value::Num(1.0),
                Value::Num(2.0),
                Value::Num(3.0)
            ]))
        );
    }
}
