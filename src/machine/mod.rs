mod function;
mod machine;
mod register;
mod stack;

pub mod errors;
pub mod operation;
pub mod value;
pub use machine::Machine;

pub type Operations = std::collections::HashMap<&'static str, operation::Operation>;
