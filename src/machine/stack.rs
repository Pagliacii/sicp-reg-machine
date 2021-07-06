//! A stack structure

use super::value::Value;

#[derive(Debug)]
pub struct Stack {
    stack: Vec<Value>,
    num_pushes: i32,
    max_depth: i32,
    curr_depth: i32,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            stack: Vec::new(),
            num_pushes: 0,
            max_depth: 0,
            curr_depth: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.curr_depth == 0 && self.stack.is_empty()
    }

    pub fn push(&mut self, item: Value) {
        self.stack.push(item);
        self.num_pushes += 1;
        self.curr_depth += 1;
        self.max_depth = std::cmp::max(self.curr_depth, self.max_depth);
    }

    pub fn pop(&mut self) -> Result<Value, &'static str> {
        if let Some(item) = self.stack.pop() {
            self.curr_depth -= 1;
            Ok(item)
        } else {
            Err("Empty stack: POP")
        }
    }

    pub fn initialize(&mut self) {
        self.stack.clear();
        self.num_pushes = 0;
        self.max_depth = 0;
        self.curr_depth = 0;
    }

    pub fn print_statistics(&self) {
        println!(
            "\ntotal-pushes = {} maximum-depth = {}",
            self.num_pushes, self.max_depth
        );
    }
}

#[cfg(test)]
mod stack_tests {
    use super::*;

    #[test]
    fn test_push_item() {
        let mut stack: Stack = Stack::new();
        let right: i32 = 42;
        stack.push(Value::new(right));
        assert_eq!(stack.num_pushes, 1);
        assert_eq!(stack.curr_depth, 1);
        assert_eq!(stack.max_depth, 1);
    }

    #[test]
    fn test_pop_item() {
        let mut stack: Stack = Stack::new();
        let right = Value::Integer(42);
        stack.push(right.clone());
        assert_eq!(stack.num_pushes, 1);
        assert_eq!(stack.curr_depth, 1);
        assert_eq!(stack.max_depth, 1);

        let popped = stack.pop().unwrap();
        assert_eq!(popped, right);
        assert_eq!(stack.num_pushes, 1);
        assert_eq!(stack.curr_depth, 0);
        assert_eq!(stack.max_depth, 1);
    }

    #[test]
    fn test_initialize() {
        let mut stack: Stack = Stack::new();
        stack.push(Value::new("Hello!".to_string()));
        stack.push(Value::new(42));
        assert!(stack.pop().is_ok());
        stack.initialize();
        assert!(stack.is_empty());
        assert_eq!(stack.num_pushes, 0);
        assert_eq!(stack.curr_depth, 0);
        assert_eq!(stack.max_depth, 0);
    }

    #[test]
    fn test_is_empty() {
        let mut stack: Stack = Stack::new();
        assert!(stack.is_empty());

        stack.push(Value::new("Hello!".to_string()));
        stack.push(Value::new(42));
        assert!(!stack.is_empty());

        stack.pop().ok();
        assert!(!stack.is_empty());

        stack.initialize();
        assert!(stack.is_empty());
    }
}
