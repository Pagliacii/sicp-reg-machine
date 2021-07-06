use std::fmt;

use thiserror::Error;

/// Machine errors
#[derive(Debug, Error, PartialEq)]
pub enum MachineError {
    #[error(transparent)]
    OperationError(#[from] OperationError),
    #[error(transparent)]
    TypeError(#[from] TypeError),
    #[error(transparent)]
    RegisterError(#[from] RegisterError),
    #[error("Failed to convert a vector to a tuple.")]
    ToTupleError,
    #[error("Unknown label: {0}")]
    UnknownLabel(String),
    #[error("Unknown instruction: {0}")]
    UnknownInstruction(String),
    #[error("Unrecognized instructions.")]
    UnrecognizedInsts,
    #[error("No more instructions to be executed.")]
    NoMoreInsts,
    #[error("Unable to assemble the controller text, caused by\n\t{0}")]
    UnableAssemble(String),
    #[error("Stack error: {0}.")]
    StackError(String),
}

pub type MResult<T> = std::result::Result<T, MachineError>;

/// Ref: https://docs.rs/oso/0.13.0/src/oso/errors.rs.html
#[derive(Debug, Error, PartialEq)]
pub struct TypeError {
    pub got: Option<String>,
    pub expected: String,
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref got) = self.got {
            writeln!(f, "TypeError: expected {}, got {}", self.expected, got)
        } else {
            writeln!(f, "TypeError: expected {}", self.expected)
        }
    }
}

impl TypeError {
    /// Create a type error with expected type `expected`.
    pub fn expected<T: Into<String>>(expected: T) -> Self {
        Self {
            got: None,
            expected: expected.into(),
        }
    }

    /// Set `got` on self.
    pub fn got<T: Into<String>>(mut self, got: T) -> Self {
        self.got.replace(got.into());
        self
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum OperationError {
    #[error("Operation {0} not found")]
    NotFound(String),
    #[error("Operation {0} call failure")]
    CallFailure(String),
    #[error("Operation {op_name} call with an invalid type {type_name} of argument {arg_name}")]
    InvalidArgType {
        op_name: String,
        arg_name: String,
        type_name: String,
    },
}

#[derive(Debug, Error, PartialEq)]
pub enum RegisterError {
    #[error("Unknown register: {0}")]
    LookupFailure(String),
    #[error("Multiply defined register: {0}")]
    AllocateFailure(String),
    #[error("Unmatched content type in register {reg_name}, expected {type_name}")]
    UnmatchedContentType { reg_name: String, type_name: String },
}
