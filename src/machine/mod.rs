mod function;
mod machine;
mod operation;
mod register;
mod stack;
mod value;

pub mod errors;
pub use machine::Machine;

pub type Operations = std::collections::HashMap<&'static str, operation::Operation>;
