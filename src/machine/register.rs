//! A register structure to save something.

use std::any::Any;
use std::sync::Arc;

use super::BaseType;

#[derive(Debug)]
pub struct Register {
    contents: BaseType,
}

impl Register {
    pub fn new() -> Self {
        Self {
            contents: Arc::new(String::from("*unassigned*")),
        }
    }

    pub fn get(&self) -> BaseType {
        Arc::clone(&self.contents)
    }

    pub fn set<T>(&mut self, value: T)
    where
        T: Any + Send + Sync,
    {
        self.contents = Arc::new(value);
    }
}

#[cfg(test)]
mod register_tests {
    use super::*;

    #[test]
    fn test_get_register_contents() {
        let reg: Register = Register::new();
        let expected = Arc::new(String::from("*unassigned*"));
        let actual = reg.get();
        assert_eq!(expected, actual.downcast::<String>().unwrap());
    }

    #[test]
    fn test_set_register_contents() {
        let mut reg: Register = Register::new();
        let expected: i32 = 12345678;
        reg.set(expected);
        let actual = reg.get();
        assert_eq!(Arc::new(expected), actual.downcast::<i32>().unwrap());
    }
}
