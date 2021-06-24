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
