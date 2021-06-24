use std::any::Any;
use std::cmp;

#[derive(Debug)]
pub struct Stack<'a> {
    stack: Vec<&'a dyn Any>,
    num_pushes: i32,
    max_depth: i32,
    curr_depth: i32,
}

impl<'a> Stack<'a> {
    pub fn new() -> Self {
        Stack {
            stack: Vec::new(),
            num_pushes: 0,
            max_depth: 0,
            curr_depth: 0,
        }
    }

    pub fn push(&mut self, item: &'a dyn Any) {
        self.stack.push(item);
        self.num_pushes += 1;
        self.curr_depth += 1;
        self.max_depth = cmp::max(self.curr_depth, self.max_depth);
    }

    pub fn pop(&mut self) -> &dyn Any {
        if let Some(item) = self.stack.pop() {
            self.curr_depth -= 1;
            item
        } else {
            panic!("Empty stack: POP");
        }
    }

    pub fn initialize(&mut self) {
        self.stack.clear();
        self.num_pushes = 0;
        self.max_depth = 0;
        self.curr_depth = 0;
    }

    pub fn print_statistics(&self) {
        println!("\ntotal-pushes = {} maximum-depth = {}", self.num_pushes, self.max_depth);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_item() {
        let mut stack: Stack = Stack::new();
        let right: i32 = 42;
        stack.push(&right);
        assert_eq!(stack.num_pushes, 1);
        assert_eq!(stack.curr_depth, 1);
        assert_eq!(stack.max_depth, 1);
    }

    #[test]
    fn test_pop_item() {
        let mut stack: Stack = Stack::new();
        let right: i32 = 42;
        stack.push(&right);
        assert_eq!(stack.num_pushes, 1);
        assert_eq!(stack.curr_depth, 1);
        assert_eq!(stack.max_depth, 1);
        if let Some(&left) = stack.pop().downcast_ref::<i32>() {
            assert_eq!(left, right);
        } else {
            assert!(false);
        }
        assert_eq!(stack.num_pushes, 1);
        assert_eq!(stack.curr_depth, 0);
        assert_eq!(stack.max_depth, 1);
    }

    #[test]
    fn test_initialize() {
        let mut stack: Stack = Stack::new();
        stack.push(&"Hello!");
        stack.push(&42);
        stack.pop();
        stack.initialize();
        assert!(stack.stack.is_empty());
        assert_eq!(stack.num_pushes, 0);
        assert_eq!(stack.curr_depth, 0);
        assert_eq!(stack.max_depth, 0);
    }
}
