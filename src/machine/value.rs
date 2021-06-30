use std::any::{type_name, Any, TypeId};
use std::convert::{From, TryFrom};
use std::fmt;
use std::sync::Arc;

use impl_trait_for_tuples::*;

use super::errors::{MachineError, Result, TypeError};
use super::BaseType;

/// An enum of the possible value types that can be sent to an operation.
#[derive(Clone, Debug)]
pub enum Value {
    Integer(i32),
    Float(f64),
    String(String),
    Boolean(bool),
    Compound(CompoundValue),
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
            (Value::Float(f1), Value::Float(f2)) => f1 == f2,
            (Value::Integer(i1), Value::Integer(i2)) => i1 == i2,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            (Value::Compound(c1), Value::Compound(c2)) => c1 == c2,
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
            Value::Compound(c) => c.to_string(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Value {
    pub fn new<T>(val: T) -> Self
    where
        T: Any + Send + Sync + fmt::Debug,
    {
        let type_id = val.type_id();
        if TypeId::of::<i32>() == type_id {
            Self::Integer(*(&val as &dyn Any).downcast_ref::<i32>().unwrap())
        } else if TypeId::of::<f64>() == type_id {
            Self::Float(*(&val as &dyn Any).downcast_ref::<f64>().unwrap())
        } else if TypeId::of::<bool>() == type_id {
            Self::Boolean(*(&val as &dyn Any).downcast_ref::<bool>().unwrap())
        } else if TypeId::of::<String>() == type_id {
            Self::String(
                (&val as &dyn Any)
                    .downcast_ref::<String>()
                    .unwrap()
                    .to_owned(),
            )
        } else {
            Self::Compound(CompoundValue::new(val))
        }
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

/// Container for the composite value.
#[derive(Clone)]
pub struct CompoundValue {
    /// actual value
    inner: BaseType,
    /// type name of the actual value
    inner_type_name: &'static str,
    /// string format for comparing
    inner_string_format: String,
}

impl fmt::Debug for CompoundValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CompoundValue<{}>", self.inner_type_name)
    }
}

impl fmt::Display for CompoundValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompoundValue({})", self.inner_string_format)
    }
}

impl PartialEq for CompoundValue {
    fn eq(&self, other: &Self) -> bool {
        self.type_id() == other.type_id() && self.to_string() == other.to_string()
    }
}

impl FromValue for CompoundValue {
    fn from_value(val: Value) -> Result<Self> {
        if let Value::Compound(v) = val {
            Ok(v)
        } else {
            Err(TypeError::expected("CompoundValue").got(val))?
        }
    }
}

impl CompoundValue {
    pub fn new<T>(result: T) -> Self
    where
        T: Any + Send + Sync + fmt::Debug,
    {
        let string_format = format!("{:?}", result);
        Self {
            inner: Arc::new(result),
            inner_type_name: type_name::<T>(),
            inner_string_format: string_format,
        }
    }

    pub fn type_id(&self) -> TypeId {
        self.inner.as_ref().type_id()
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
                .got(type_name::<T>())
                .into()
        })
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
        let a = Value::new(1);
        let b = Value::Integer(2);
        let c = Value::new(3.0);
        let d = Value::Float(4.0);
        let e = Value::new(true);
        let f = Value::Boolean(false);
        let g = Value::new(String::from("Hello"));
        let h = Value::String(String::from("World"));
        let i = Value::new(CompoundValue::new(1));
        let j = Value::Compound(CompoundValue::new("hello"));
        // Comparing Value::Integer tests
        compare_value(&a, &b);
        // Comparing Value::Float tests
        compare_value(&c, &d);
        // Comparing Value::Boolean tests
        compare_value(&e, &f);
        // Comparing Value::String tests
        compare_value(&g, &h);
        // Comparing Value::Compound tests
        compare_value(&i, &j);
        // Comparing Value::Integer and Value::Float
        assert_ne!(a, c);
        assert_ne!(d, b);
    }

    #[test]
    fn test_compound_value() {
        let v = CompoundValue::new(1);
        let actual = v.value().downcast::<i32>();
        assert!(actual.is_ok());
        assert_eq!(Arc::new(1), actual.unwrap());
    }

    #[test]
    fn test_compound_value_downcast() {
        let v = CompoundValue::new(1);
        let actual = v.downcast_ref::<i32>();
        assert!(actual.is_ok());
        assert_eq!(&1, actual.unwrap());
        assert_eq!(
            Err(TypeError::expected("i32").got("f32").into()),
            v.downcast_ref::<f32>(),
        );
    }

    #[test]
    fn test_compound_value_compare() {
        let c1 = CompoundValue::new(1);
        let c2 = CompoundValue::new("hello");
        let c3 = CompoundValue::new("hello".to_string());
        let c4 = CompoundValue::new(1);
        assert_ne!(c1, c2);
        assert_ne!(c2, c3);
        assert_eq!(c1, c4);
    }

    #[test]
    fn test_from_value_trait() {
        let i = Value::new(1);
        let ii = i32::from_value(i);
        assert!(ii.is_ok());
        assert_eq!(1, ii.unwrap());

        let f = Value::new(1.0);
        let ff = f64::from_value(f);
        assert!(ff.is_ok());
        assert_eq!(1.0, ff.unwrap());

        let b = Value::new(false);
        let bb = bool::from_value(b);
        assert!(bb.is_ok());
        assert_eq!(false, bb.unwrap());

        let s = Value::new("hello".to_owned());
        let ss = String::from_value(s);
        assert!(ss.is_ok());
        assert_eq!("hello".to_string(), ss.unwrap());
    }

    #[test]
    fn test_from_value_for_compound_value() {
        #[derive(Debug)]
        struct Foo {}
        let c = Value::new(Foo {});
        let cc = CompoundValue::from_value(c);
        assert!(cc.is_ok());
        assert_eq!(CompoundValue::new(Foo {}), cc.unwrap());
    }
}
