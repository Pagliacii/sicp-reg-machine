//! A register structure to save something.

use super::value::Value;

#[derive(Clone, Debug)]
pub struct Register {
    contents: Value,
}

impl Register {
    pub fn new() -> Self {
        Self {
            contents: Value::String("*unassigned*".into()),
        }
    }

    pub fn get(&self) -> Value {
        self.contents.clone()
    }

    pub fn set(&mut self, value: Value) {
        self.contents = value;
    }
}

#[cfg(test)]
mod register_tests {
    use super::*;

    #[test]
    fn test_get_register_contents() {
        let reg: Register = Register::new();
        let expected = Value::String("*unassigned*".into());
        let actual = reg.get();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_set_register_contents() {
        let mut reg: Register = Register::new();
        let expected = Value::Integer(12345678);
        reg.set(expected.clone());
        let actual = reg.get();
        assert_eq!(expected, actual);
    }
}
