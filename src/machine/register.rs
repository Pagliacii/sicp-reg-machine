use std::any::Any;

#[derive(Debug)]
pub struct Register {
    contents: Box<dyn Any>,
}

impl Register {
    pub fn new() -> Self {
        Register { contents: Box::new(String::from("*unassigned*")) }
    }

    pub fn get(&self) -> &dyn Any {
        self.contents.as_ref()
    }

    pub fn set<T: Any + Copy>(&mut self, value: &T) {
        self.contents = Box::new(value.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_register_contents() {
        let reg: Register = Register::new();
        let right: String = String::from("*unassigned*");
        if let Some(left) = reg.get().downcast_ref::<String>() {
            assert_eq!(left, &right);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_set_register_contents() {
        let mut reg: Register = Register::new();
        let right: i32 = 12345678;
        reg.set(&right);
        if let Some(&left) = reg.get().downcast_ref::<i32>() {
            assert_eq!(left, right);
        } else {
            assert!(false);
        }
    }
}
