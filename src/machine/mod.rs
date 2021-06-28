mod errors;
mod function;
mod value;

pub mod machine;
pub mod operation;
pub mod register;
pub mod stack;

type BaseType = std::sync::Arc<dyn std::any::Any + Send + Sync>;
