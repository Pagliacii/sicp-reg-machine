//! A stack structure

use std::any::Any;
use std::cmp;
use std::sync::Arc;

use super::BaseType;

#[derive(Debug)]
pub struct Stack {
    stack: Vec<BaseType>,
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

    pub fn push<T>(&mut self, item: T)
    where
        T: Any + Send + Sync,
    {
        self.stack.push(Arc::new(item));
        self.num_pushes += 1;
        self.curr_depth += 1;
        self.max_depth = cmp::max(self.curr_depth, self.max_depth);
    }

    pub fn pop(&mut self) -> Result<BaseType, &'static str> {
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
        stack.push(right);
        assert_eq!(stack.num_pushes, 1);
        assert_eq!(stack.curr_depth, 1);
        assert_eq!(stack.max_depth, 1);
    }

    #[test]
    fn test_pop_item() {
        let mut stack: Stack = Stack::new();
        let right: i32 = 42;
        stack.push(right);
        assert_eq!(stack.num_pushes, 1);
        assert_eq!(stack.curr_depth, 1);
        assert_eq!(stack.max_depth, 1);
        let popped = stack.pop().unwrap();
        assert_eq!(popped.downcast_ref::<i32>(), Some(&right));
        assert_eq!(stack.num_pushes, 1);
        assert_eq!(stack.curr_depth, 0);
        assert_eq!(stack.max_depth, 1);
    }

    #[test]
    fn test_initialize() {
        let mut stack: Stack = Stack::new();
        stack.push("Hello!");
        stack.push(42);
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

        stack.push("Hello!");
        stack.push(42);
        assert!(!stack.is_empty());

        stack.pop().ok();
        assert!(!stack.is_empty());

        stack.initialize();
        assert!(stack.is_empty());
    }
}
