use std::fmt;

use thiserror::Error;

/// Operations errors
#[derive(Debug, Error)]
pub enum MachineError {
    #[error(transparent)]
    OperationError(#[from] OperationError),
    #[error(transparent)]
    TypeError(#[from] TypeError),
    #[error(transparent)]
    RegisterError(#[from] RegisterError),
    #[error("Failed to convert a vector to a tuple.")]
    ToTupleError,
}

pub type Result<T> = std::result::Result<T, MachineError>;

impl PartialEq for MachineError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::OperationError(op1), Self::OperationError(op2)) => op1 == op2,
            (Self::TypeError(t1), Self::TypeError(t2)) => t1 == t2,
            (Self::RegisterError(r1), Self::RegisterError(r2)) => r1 == r2,
            (Self::ToTupleError, Self::ToTupleError) => true,
            _ => false,
        }
    }
}

/// Ref: https://docs.rs/oso/0.13.0/src/oso/errors.rs.html
#[derive(Debug, Error)]
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

impl PartialEq for TypeError {
    fn eq(&self, other: &Self) -> bool {
        self.expected == other.expected && self.got == other.got
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

#[derive(Debug, Error)]
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

impl PartialEq for OperationError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::NotFound(op1), Self::NotFound(op2)) => op1 == op2,
            (Self::CallFailure(op1), Self::CallFailure(op2)) => op1 == op2,
            (
                Self::InvalidArgType {
                    op_name: op1,
                    arg_name: a1,
                    type_name: t1,
                },
                Self::InvalidArgType {
                    op_name: op2,
                    arg_name: a2,
                    type_name: t2,
                },
            ) => op1 == op2 && a1 == a2 && t1 == t2,
            _ => false,
        }
    }
}

#[derive(Debug, Error)]
pub enum RegisterError {
    #[error("Unknown register: {0}")]
    LookupFailure(String),
    #[error("Multiply defined register: {0}")]
    AllocateFailure(String),
}

impl PartialEq for RegisterError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::LookupFailure(r1), Self::LookupFailure(r2)) => r1 == r2,
            (Self::AllocateFailure(r1), Self::AllocateFailure(r2)) => r1 == r2,
            _ => false,
        }
    }
}
