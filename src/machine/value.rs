use std::{any::Any, collections::HashMap, fmt};

use impl_trait_for_tuples::*;

use super::errors::{MResult, MachineError, TypeError};
use super::operation::Operation;

/// An enum of the possible value types that can be sent to an operation.
#[derive(Clone, PartialEq)]
pub enum Value {
    Integer(i32),
    Float(f64),
    BigNum(u64),
    Pointer(usize),
    String(String),
    Boolean(bool),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Op(Operation),
    Unit,
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Boolean(v) => write!(f, "<Boolean {}>", v),
            Value::BigNum(v) => write!(f, "<BigNum {}>", v),
            Value::Float(v) => write!(f, "<Float {}>", v),
            Value::Integer(v) => write!(f, "<Integer {}>", v),
            Value::List(v) => write!(f, "<List {:?}>", v.type_id()),
            Value::Map(v) => write!(f, "<Map {:?}>", v.type_id()),
            Value::Pointer(v) => write!(f, "<Pointer {}>", v),
            Value::String(v) => write!(f, r#"<String "{}">"#, v),
            Value::Op(_) => write!(f, "<Operation>"),
            Value::Unit => write!(f, "<Unit>"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Boolean(v) => write!(f, "{}", v),
            Value::BigNum(v) => write!(f, "{}", v),
            Value::Float(v) => write!(f, "{}", v),
            Value::Integer(v) => write!(f, "{}", v),
            Value::List(l) => write!(
                f,
                "({})",
                l.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Value::Map(m) => write!(
                f,
                "({})",
                m.iter()
                    .map(|(k, v)| format!("({} {})", k, v.to_string()))
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
            Value::Pointer(v) => write!(f, "{}", v),
            Value::String(v) => write!(f, "{}", v),
            Value::Op(_) => write!(f, "Operation"),
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
        Value::Integer(*self)
    }
}

impl ToValue for f64 {
    fn to_value(&self) -> Value {
        Value::Float(*self)
    }
}

impl ToValue for u64 {
    fn to_value(&self) -> Value {
        Value::BigNum(*self)
    }
}

impl ToValue for usize {
    fn to_value(&self) -> Value {
        Value::Pointer(*self)
    }
}

impl ToValue for bool {
    fn to_value(&self) -> Value {
        Value::Boolean(*self)
    }
}

impl ToValue for String {
    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl ToValue for &dyn ToString {
    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl ToValue for &'static str {
    fn to_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl ToValue for Vec<Value> {
    fn to_value(&self) -> Value {
        Value::List(self.clone())
    }
}

impl ToValue for HashMap<String, Value> {
    fn to_value(&self) -> Value {
        Value::Map(self.clone())
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
        let expected = TypeError::expected("Value::Integer");
        match v {
            Value::Integer(val) => Ok(val),
            Value::Float(val) => Ok(val as i32),
            Value::BigNum(val) => Ok(val as i32),
            Value::Pointer(val) => Ok(val as i32),
            Value::String(val) => val
                .parse::<i32>()
                .map_err(|_| expected.got(format!("String {}", val))),
            _ => Err(expected.got(v.to_string())),
        }
    }
}

impl TryFromValue for f64 {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        let expected = TypeError::expected("Value::Float");
        match v {
            Value::Float(val) => Ok(val),
            Value::Integer(val) => Ok(val as f64),
            Value::BigNum(val) => Ok(val as f64),
            Value::Pointer(val) => Ok(val as f64),
            Value::String(val) => val
                .parse::<f64>()
                .map_err(|_| expected.got(format!("String {}", val))),
            _ => Err(expected.got(v.to_string())),
        }
    }
}

impl TryFromValue for u64 {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        let expected = TypeError::expected("Value::BigNum");
        match v {
            Value::BigNum(val) => Ok(val),
            Value::Integer(val) => Ok(val as u64),
            Value::Float(val) => Ok(val as u64),
            Value::Pointer(val) => Ok(val as u64),
            Value::String(val) => val
                .parse::<u64>()
                .map_err(|_| expected.got(format!("String {}", val))),
            _ => Err(expected.got(v.to_string())),
        }
    }
}

impl TryFromValue for usize {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        let expected = TypeError::expected("Value::Pointer");
        match v {
            Value::Pointer(val) => Ok(val),
            Value::BigNum(val) => Ok(val as usize),
            Value::Integer(val) => Ok(val as usize),
            Value::Float(val) => Ok(val as usize),
            Value::String(val) => val
                .parse::<usize>()
                .map_err(|_| expected.got(format!("String {}", val))),
            _ => Err(expected.got(v.to_string())),
        }
    }
}

impl TryFromValue for bool {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        let expected = TypeError::expected("Value::Boolean");
        match v {
            Value::Boolean(val) => Ok(val),
            Value::String(val) => {
                if val == "true" {
                    Ok(true)
                } else if val == "false" {
                    Ok(false)
                } else {
                    Err(expected.got(format!("String {}", val)))
                }
            }
            _ => Err(expected.got(v.to_string())),
        }
    }
}

impl TryFromValue for String {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        Ok(v.to_string())
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

impl TryFromValue for HashMap<String, Value> {
    fn try_from(v: Value) -> Result<Self, TypeError> {
        if let Value::Map(val) = v {
            Ok(val)
        } else {
            Err(TypeError::expected("Value::Map").got(v.to_string()))
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
        Ok((for_tuples!(
            #( Tuple::try_from(iter.next().ok_or(
                MachineError::ToTupleError
            )?.clone())? ),*
        )))
    }
}

#[cfg(test)]
mod value_mod_tests {
    use super::*;

    #[test]
    fn test_value_constructor() {
        assert_eq!(Value::Integer(1), Value::new(1));
        assert_eq!(Value::Float(1.0), Value::new(1.0));
        assert_eq!(Value::BigNum(1), Value::new(1u64));
        assert_eq!(Value::Pointer(1), Value::new(1usize));
        assert_eq!(Value::Boolean(true), Value::new(true));
        assert_eq!(Value::Boolean(false), Value::new(false));
        assert_eq!(Value::String("test".into()), Value::new("test"));
        assert_eq!(
            Value::String("test".into()),
            Value::new(String::from("test"))
        );
        assert_eq!(Value::List(Vec::<Value>::new()), Value::new(vec![]));
        assert_eq!(
            Value::Map(HashMap::<String, Value>::new()),
            Value::new(HashMap::<String, Value>::new())
        );
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
            Ok(HashMap::<String, Value>::new()),
            HashMap::<String, Value>::try_from(Value::new(HashMap::<String, Value>::new()))
        );
    }
}
