use std::{
    any::{type_name, Any, TypeId},
    convert::From,
    fmt,
    sync::Arc,
};

use impl_trait_for_tuples::*;

use super::errors::{MResult, MachineError, TypeError};

/// An enum of the possible value types that can be sent to an operation.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(i32),
    BigNum(u64),
    Pointer(usize),
    String(String),
    Boolean(bool),
    Compound(CompoundValue),
}

impl From<Value> for String {
    fn from(item: Value) -> Self {
        match item {
            Value::Boolean(b) => format!("Boolean ({})", b),
            Value::Integer(i) => format!("Integer ({})", i),
            Value::BigNum(n) => format!("BigNum ({})", n),
            Value::Pointer(p) => format!("Pointer ({})", p),
            Value::String(s) => format!("String ({})", s),
            Value::Compound(c) => c.to_string(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from(self.clone()))
    }
}

impl Value {
    pub fn new<T>(val: T) -> Self
    where
        T: Any + fmt::Debug + PartialEq + Send + Sync,
    {
        let type_id = val.type_id();
        if TypeId::of::<i32>() == type_id {
            Self::Integer(*(&val as &dyn Any).downcast_ref::<i32>().unwrap())
        } else if TypeId::of::<usize>() == type_id {
            Self::Pointer(*(&val as &dyn Any).downcast_ref::<usize>().unwrap())
        } else if TypeId::of::<u64>() == type_id {
            Self::BigNum(*(&val as &dyn Any).downcast_ref::<u64>().unwrap())
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
    fn from_value(val: Value) -> MResult<Self>;
}

impl FromValue for Value {
    fn from_value(val: Value) -> MResult<Self> {
        Ok(val)
    }
}

impl FromValue for bool {
    fn from_value(val: Value) -> MResult<Self> {
        match val {
            Value::Boolean(v) => Ok(v),
            Value::String(s) => Ok(s == "true".to_string()),
            _ => Err(TypeError::expected("bool").got(val))?,
        }
    }
}

impl FromValue for i32 {
    fn from_value(val: Value) -> MResult<Self> {
        match val {
            Value::Integer(i) => Ok(i),
            Value::String(s) => s.parse::<i32>().map_err(|_| MachineError::ConvertError {
                value: s,
                src: "String".to_string(),
                dst: "i32".to_string(),
            }),
            _ => Err(TypeError::expected("i32").got(val))?,
        }
    }
}

impl FromValue for usize {
    fn from_value(val: Value) -> MResult<Self> {
        match val {
            Value::Pointer(p) => Ok(p),
            Value::Integer(i) => Ok(i as usize),
            Value::String(s) => s.parse::<usize>().map_err(|_| MachineError::ConvertError {
                value: s,
                src: "String".to_string(),
                dst: "usize".to_string(),
            }),
            _ => Err(TypeError::expected("usize").got(val))?,
        }
    }
}

impl FromValue for u64 {
    fn from_value(val: Value) -> MResult<Self> {
        match val {
            Value::BigNum(n) => Ok(n),
            Value::Integer(i) => Ok(i as u64),
            Value::Pointer(p) => Ok(p as u64),
            Value::String(s) => s.parse::<u64>().map_err(|_| MachineError::ConvertError {
                value: s,
                src: "String".to_string(),
                dst: "u64".to_string(),
            }),
            _ => Err(TypeError::expected("u64").got(val))?,
        }
    }
}

impl FromValue for String {
    fn from_value(val: Value) -> MResult<Self> {
        match val {
            Value::Boolean(v) => Ok(v.to_string()),
            Value::Integer(v) => Ok(v.to_string()),
            Value::Pointer(v) => Ok(v.to_string()),
            Value::String(v) => Ok(v.to_string()),
            _ => Err(TypeError::expected("String").got(val))?,
        }
    }
}

/// Convert Vec<Value> to designated types.
pub trait FromValueList {
    fn from_value_list(values: &[Value]) -> MResult<Self>
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
    fn from_value_list(values: &[Value]) -> MResult<Self> {
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
    inner: Arc<dyn Any + Send + Sync>,
    vtable: VTable,
}

impl fmt::Debug for CompoundValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CompoundValue<{}>", self.vtable.type_name)
    }
}

impl fmt::Display for CompoundValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompoundValue({:?})", (self.vtable.debug)(&*self.inner))
    }
}

impl PartialEq for CompoundValue {
    fn eq(&self, other: &Self) -> bool {
        (self.vtable.partial_eq)(&*self.inner, &*other.inner)
    }
}

impl FromValue for CompoundValue {
    fn from_value(val: Value) -> MResult<Self> {
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
        T: Any + fmt::Debug + PartialEq + Send + Sync,
    {
        Self {
            inner: Arc::new(result),
            vtable: VTable::for_type::<T>(),
        }
    }

    pub fn value(&self) -> Arc<dyn Any + Send + Sync> {
        Arc::clone(&self.inner)
    }

    pub fn downcast_ref<T>(&self) -> MResult<&T>
    where
        T: Any + Send + Sync,
    {
        self.inner.as_ref().downcast_ref().ok_or_else(|| {
            TypeError::expected(self.vtable.type_name)
                .got(type_name::<T>())
                .into()
        })
    }
}

/// See https://users.rust-lang.org/t/how-could-i-implement-a-more-accurate-comparison/61698/6
#[derive(Copy, Clone)]
struct VTable {
    type_name: &'static str,
    debug: fn(&dyn Any) -> &dyn fmt::Debug,
    partial_eq: fn(&dyn Any, &dyn Any) -> bool,
}

impl VTable {
    fn for_type<T>() -> Self
    where
        T: Any + fmt::Debug + PartialEq + 'static,
    {
        Self {
            type_name: type_name::<T>(),
            debug: |value: &dyn Any| -> &dyn fmt::Debug { value.downcast_ref::<T>().unwrap() },
            partial_eq: |left: &dyn Any, right: &dyn Any| -> bool {
                match (left.downcast_ref::<T>(), right.downcast_ref::<T>()) {
                    (Some(l), Some(r)) => l == r,
                    _ => false,
                }
            },
        }
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
        let c = Value::new(3usize);
        let d = Value::Pointer(4);
        let e = Value::new(true);
        let f = Value::Boolean(false);
        let g = Value::new(String::from("Hello"));
        let h = Value::String(String::from("World"));
        let i = Value::new(CompoundValue::new(1));
        let j = Value::Compound(CompoundValue::new("hello"));
        let k = Value::new(4294967296u64);
        let l = Value::BigNum(4294967296);
        // Comparing Value::Integer tests
        compare_value(&a, &b);
        // Comparing Value::Pointer tests
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
        // Comparing Value::BigNum
        assert_eq!(k, l);
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

        let p = Value::Pointer(1);
        let pp = usize::from_value(p);
        assert!(pp.is_ok());
        assert_eq!(1usize, pp.unwrap());

        let b = Value::new(false);
        let bb = bool::from_value(b);
        assert!(bb.is_ok());
        assert_eq!(false, bb.unwrap());

        let s = Value::new("hello".to_owned());
        let ss = String::from_value(s);
        assert!(ss.is_ok());
        assert_eq!("hello".to_string(), ss.unwrap());

        let n = Value::new(4294967296u64);
        let nn = u64::from_value(n);
        assert!(nn.is_ok());
        assert_eq!(4294967296, nn.unwrap());
    }

    #[test]
    fn test_from_value_for_compound_value() {
        #[derive(Debug, PartialEq)]
        struct Foo {}
        let c = Value::new(Foo {});
        let cc = CompoundValue::from_value(c);
        assert!(cc.is_ok());
        assert_eq!(CompoundValue::new(Foo {}), cc.unwrap());
    }
}
