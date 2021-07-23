use std::{
    any::Any,
    cmp::Ordering,
    fmt,
    ops::{Add, Div, Mul, Neg, Sub},
};

use super::errors::{MResult, ProcedureError, TypeError};
use super::procedure::Procedure;

/// An enum of the possible value types that can be sent to an operation.
#[derive(Clone, PartialEq)]
pub enum Value {
    Num(f64),
    Symbol(String),
    String(String),
    Boolean(bool),
    List(Vec<Value>),
    Nil,
    Pointer(usize),
    Procedure(Procedure),
}

impl Value {
    pub fn new<T: ToValue>(val: T) -> Self {
        val.to_value()
    }

    pub fn zero() -> Self {
        Value::Num(0.0)
    }

    pub fn one() -> Self {
        Value::Num(1.0)
    }

    pub fn nil() -> Self {
        Value::Nil
    }

    pub fn empty_list() -> Self {
        Value::List(vec![])
    }

    pub fn perform(&self, args: Vec<Value>) -> MResult<Self> {
        if let Self::Procedure(p) = self {
            p.execute(args)
        } else {
            Err(ProcedureError::UnablePerform(self.to_string()))?
        }
    }

    pub fn eq_num<F: Into<f64>>(&self, num: F) -> bool {
        if let Self::Num(f) = self {
            f.eq(&num.into())
        } else {
            false
        }
    }

    pub fn eq_pointer(&self, num: usize) -> bool {
        if let Self::Pointer(p) = self {
            *p == num
        } else {
            false
        }
    }

    pub fn is_num(&self) -> bool {
        if let Self::Num(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_symbol(&self) -> bool {
        if let Self::Symbol(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_string(&self) -> bool {
        if let Self::String(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_pointer(&self) -> bool {
        if let Self::Pointer(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_bool(&self) -> bool {
        if let Self::Boolean(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_true(&self) -> bool {
        Self::Boolean(true) == *self
    }

    pub fn is_false(&self) -> bool {
        Self::Boolean(false) == *self
    }

    pub fn is_nil(&self) -> bool {
        Self::Nil == *self
    }

    pub fn is_list(&self) -> bool {
        if let Self::List(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_empty_list(&self) -> bool {
        if let Self::List(l) = self {
            l.is_empty()
        } else {
            false
        }
    }

    pub fn is_procedure(&self) -> bool {
        if let Self::Procedure(_) = self {
            true
        } else {
            false
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Boolean(v) => write!(f, "<Boolean {}>", v),
            Value::Num(v) => write!(f, "<Num {}>", v),
            Value::List(v) => write!(f, "<List {:?}>", v.type_id()),
            Value::Symbol(v) => write!(f, "<Symbol {}>", v),
            Value::String(v) => write!(f, r#"<String "{}">"#, v),
            Value::Procedure(v) => write!(f, "<Procedure {}>", v.get_name()),
            Value::Pointer(v) => write!(f, "<Pointer {}>", v),
            Value::Nil => write!(f, "<Nil>"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Boolean(v) => write!(f, "{}", if *v { "#t" } else { "#f" }),
            Value::Num(v) => write!(f, "{}", v),
            Value::Symbol(v) => write!(f, "{}", v),
            Value::List(l) => write!(f, "{}", values_to_str(l)),
            Value::String(v) => write!(f, r#""{}""#, v),
            Value::Procedure(p) => write!(f, "Procedure-{}", p.get_name()),
            Value::Pointer(v) => write!(f, "Pointer-{}", v),
            Value::Nil => write!(f, ""),
        }
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Num(augend), Value::Num(addend)) => Value::Num(augend + addend),
            (augend, addend) => panic!(
                "Unable to perform addition between {} and {}.",
                augend, addend
            ),
        }
    }
}

impl Sub for Value {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Num(minuend), Value::Num(subtrahend)) => Value::Num(minuend - subtrahend),
            (minuend, subtrahend) => panic!(
                "Unable to perform subtraction between {} and {}",
                minuend, subtrahend
            ),
        }
    }
}

impl Neg for Value {
    type Output = Self;

    fn neg(self) -> Self::Output {
        if let Self::Num(n) = self {
            Self::Num(-n)
        } else {
            panic!("Unable to perform negation with {}", self);
        }
    }
}

impl Mul for Value {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Num(multiplier), Value::Num(multiplicand)) => {
                Value::Num(multiplier * multiplicand)
            }
            (multiplier, multiplicand) => panic!(
                "Unable to perform multiplication between {} and {}",
                multiplier, multiplicand
            ),
        }
    }
}

impl Div for Value {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.eq_num(0) {
            panic!("Cannot divide by zero-valued `Value::Num`!")
        }
        match (self, rhs) {
            (Value::Num(dividend), Value::Num(divisor)) => Value::Num(dividend / divisor),
            (dividend, divisor) => panic!(
                "Unable to perform division between {} and {}",
                dividend, divisor
            ),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Num(l), Self::Num(r)) => l.partial_cmp(r),
            _ => None,
        }
    }
}

pub fn values_to_str(vals: &Vec<Value>) -> String {
    format!(
        "({})",
        vals.iter()
            .filter(|v| !v.is_nil())
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(" ")
    )
}

pub trait ToValue: Sized {
    fn to_value(self) -> Value;
}

impl ToValue for Value {
    fn to_value(self) -> Value {
        self
    }
}

pub trait NonValue: ToValue {}

impl NonValue for i32 {}
impl ToValue for i32 {
    fn to_value(self) -> Value {
        Value::Num(self as f64)
    }
}

impl NonValue for f64 {}
impl ToValue for f64 {
    fn to_value(self) -> Value {
        Value::Num(self)
    }
}

impl NonValue for u64 {}
impl ToValue for u64 {
    fn to_value(self) -> Value {
        Value::Num(self as f64)
    }
}

impl NonValue for usize {}
impl ToValue for usize {
    fn to_value(self) -> Value {
        Value::Num(self as f64)
    }
}

impl NonValue for bool {}
impl ToValue for bool {
    fn to_value(self) -> Value {
        Value::Boolean(self)
    }
}

impl NonValue for String {}
impl ToValue for String {
    fn to_value(self) -> Value {
        if self.starts_with('"') {
            Value::String(self)
        } else {
            Value::Symbol(self)
        }
    }
}

impl NonValue for &dyn ToString {}
impl ToValue for &dyn ToString {
    fn to_value(self) -> Value {
        let string = self.to_string();
        if string.starts_with('"') {
            Value::String(string)
        } else {
            Value::Symbol(string)
        }
    }
}

impl NonValue for &'static str {}
impl ToValue for &'static str {
    fn to_value(self) -> Value {
        let string = self.to_string();
        if string.starts_with('"') {
            Value::String(string)
        } else {
            Value::Symbol(string)
        }
    }
}

impl<T: ToValue> NonValue for Vec<T> {}
impl<T: ToValue> ToValue for Vec<T> {
    fn to_value(self) -> Value {
        Value::List(
            self.into_iter()
                .map(|v| v.to_value())
                .collect::<Vec<Value>>(),
        )
    }
}

impl ToValue for () {
    fn to_value(self) -> Value {
        Value::Nil
    }
}

pub trait TryFromValue: Sized {
    fn try_from(v: &Value) -> Result<Self, TypeError>;
}

impl TryFromValue for Value {
    fn try_from(v: &Value) -> Result<Self, TypeError> {
        Ok(v.clone())
    }
}

impl TryFromValue for i32 {
    fn try_from(v: &Value) -> Result<Self, TypeError> {
        let expected = TypeError::expected("Value::Num");
        match v {
            Value::Num(val) => Ok(*val as i32),
            Value::Symbol(val) => val
                .parse::<i32>()
                .map_err(|_| expected.got(format!("Symbol {}", val))),
            _ => Err(expected.got(v.to_string())),
        }
    }
}

impl TryFromValue for f64 {
    fn try_from(v: &Value) -> Result<Self, TypeError> {
        let expected = TypeError::expected("Value::Num");
        match v {
            Value::Num(val) => Ok(*val),
            Value::Symbol(val) => val
                .parse::<f64>()
                .map_err(|_| expected.got(format!("Symbol {}", val))),
            _ => Err(expected.got(v.to_string())),
        }
    }
}

impl TryFromValue for u64 {
    fn try_from(v: &Value) -> Result<Self, TypeError> {
        let expected = TypeError::expected("Value::Num");
        match v {
            Value::Num(val) => Ok(*val as u64),
            Value::Symbol(val) => val
                .parse::<u64>()
                .map_err(|_| expected.got(format!("Symbol {}", val))),
            _ => Err(expected.got(v.to_string())),
        }
    }
}

impl TryFromValue for usize {
    fn try_from(v: &Value) -> Result<Self, TypeError> {
        let expected = TypeError::expected("Value::Num");
        match v {
            Value::Num(val) => Ok(*val as usize),
            Value::Pointer(val) => Ok(*val),
            Value::Symbol(val) => val
                .parse::<usize>()
                .map_err(|_| expected.got(format!("Symbol {}", val))),
            _ => Err(expected.got(v.to_string())),
        }
    }
}

impl TryFromValue for bool {
    fn try_from(v: &Value) -> Result<Self, TypeError> {
        let expected = TypeError::expected("Value::Boolean");
        match v {
            Value::Boolean(val) => Ok(*val),
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
    fn try_from(v: &Value) -> Result<Self, TypeError> {
        match v {
            Value::List(_) => Ok(format!("({})", v.to_string())),
            _ => Ok(v.to_string()),
        }
    }
}

impl TryFromValue for Vec<Value> {
    fn try_from(v: &Value) -> Result<Self, TypeError> {
        match v {
            Value::List(val) => Ok(val.clone()),
            Value::Nil => Ok(vec![]),
            _ => Ok(vec![v.clone()]),
        }
    }
}

impl TryFromValue for Vec<i32> {
    fn try_from(v: &Value) -> Result<Self, TypeError> {
        match v {
            Value::List(val) => val.iter().map(|v| i32::try_from(v)).collect(),
            Value::Num(n) => Ok(vec![*n as i32]),
            _ => Err(TypeError::expected("Value::List | Value::Num").got(v.to_string())),
        }
    }
}

impl TryFromValue for Vec<f64> {
    fn try_from(v: &Value) -> Result<Self, TypeError> {
        match v {
            Value::List(val) => val.iter().map(|v| f64::try_from(v)).collect(),
            Value::Num(n) => Ok(vec![*n]),
            _ => Err(TypeError::expected("Value::List | Value::Num").got(v.to_string())),
        }
    }
}

impl TryFromValue for Vec<u64> {
    fn try_from(v: &Value) -> Result<Self, TypeError> {
        match v {
            Value::List(val) => val.iter().map(|v| u64::try_from(v)).collect(),
            Value::Num(n) => Ok(vec![*n as u64]),
            _ => Err(TypeError::expected("Value::List | Value::Num").got(v.to_string())),
        }
    }
}

impl TryFromValue for Vec<usize> {
    fn try_from(v: &Value) -> Result<Self, TypeError> {
        match v {
            Value::List(val) => val.iter().map(|v| usize::try_from(v)).collect(),
            Value::Num(n) => Ok(vec![*n as usize]),
            _ => Err(TypeError::expected("Value::List | Value::Num").got(v.to_string())),
        }
    }
}

impl TryFromValue for Vec<String> {
    fn try_from(v: &Value) -> Result<Self, TypeError> {
        match v {
            Value::List(val) => val.iter().map(|v| String::try_from(v)).collect(),
            Value::String(s) | Value::Symbol(s) => Ok(vec![s.to_string()]),
            _ => Err(
                TypeError::expected("Value::List | Value::String | Value::Symbol")
                    .got(v.to_string()),
            ),
        }
    }
}

impl TryFromValue for () {
    fn try_from(v: &Value) -> Result<Self, TypeError> {
        if v.is_nil() {
            Ok(())
        } else {
            Err(TypeError::expected("Value::Nil").got(v.to_string()))
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
        assert_eq!(
            Value::List(Vec::<Value>::new()),
            Value::new(Vec::<Value>::new())
        );
        assert_eq!(Value::Nil, Value::new(()));
    }

    #[test]
    fn test_try_from_value() {
        assert_eq!(Ok(1), i32::try_from(&Value::new(1)));
        assert_eq!(Ok(1.0), f64::try_from(&Value::new(1.0)));
        assert_eq!(Ok(2), u64::try_from(&Value::new(2u64)));
        assert_eq!(Ok(3), usize::try_from(&Value::new(3usize)));
        assert_eq!(Ok(false), bool::try_from(&Value::new(false)));
        assert_eq!(
            Ok("test".to_string()),
            String::try_from(&Value::new("test"))
        );
        assert_eq!(
            Ok(Vec::<Value>::new()),
            Vec::<Value>::try_from(&Vec::<Value>::new().to_value())
        );
        assert_eq!(
            Ok(vec![1, 2, 3]),
            Vec::<i32>::try_from(&Value::new(vec![
                Value::Num(1.0),
                Value::Num(2.0),
                Value::Num(3.0)
            ]))
        );
        assert_eq!(
            Ok(vec![1.0, 2.0, 3.0]),
            Vec::<f64>::try_from(&Value::new(vec![
                Value::Num(1.0),
                Value::Num(2.0),
                Value::Num(3.0)
            ]))
        );
    }

    #[test]
    fn test_eq_num() {
        assert!(Value::Num(1.0).eq_num(1.0));
        assert!(Value::Num(1.0).eq_num(1));
        assert!(!Value::Boolean(true).eq_num(1));
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(3.to_value(), 1.to_value() + 2.to_value());
        assert_eq!(1.to_value(), 2.to_value() - 1.to_value());
        assert_eq!(6.to_value(), 2.to_value() * 3.to_value());
        assert_eq!(2.to_value(), 4.to_value() / 2.to_value());
    }
}
