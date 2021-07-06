mod function;
mod machine;
mod operation;
mod register;
mod stack;
mod value;

pub mod errors;
pub use machine::Machine;

type BaseType = std::sync::Arc<dyn std::any::Any + Send + Sync>;
pub type Operations = std::collections::HashMap<&'static str, operation::Operation>;
