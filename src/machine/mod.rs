mod function;
mod machine;
mod register;
mod stack;
mod value;

pub mod errors;
pub mod operation;
pub use machine::Machine;

pub type Operations = std::collections::HashMap<&'static str, operation::Operation>;
