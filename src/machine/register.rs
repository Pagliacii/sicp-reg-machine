//! A register structure to save something.

use super::value::{ToValue, Value};

#[derive(Clone, Debug)]
pub struct Register {
    contents: Value,
}

impl Register {
    pub fn new() -> Self {
        Self {
            contents: Value::Symbol("*unassigned*".into()),
        }
    }

    pub fn get(&self) -> Value {
        self.contents.clone()
    }

    pub fn set<T: ToValue>(&mut self, value: T) {
        self.contents = value.to_value();
    }
}

#[cfg(test)]
mod register_tests {
    use super::*;

    #[test]
    fn test_get_register_contents() {
        let reg: Register = Register::new();
        let expected = Value::Symbol("*unassigned*".into());
        let actual = reg.get();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_set_register_contents() {
        let mut reg: Register = Register::new();
        let expected = 12345678;
        reg.set(expected);
        let actual = reg.get();
        assert_eq!(Value::new(expected), actual);
    }
}
