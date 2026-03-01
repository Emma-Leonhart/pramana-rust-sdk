use thiserror::Error;

/// Errors that can occur in the Pramana SDK.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PramanaError {
    #[error("denominator cannot be zero")]
    ZeroDenominator,

    #[error("division by zero")]
    DivisionByZero,

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("operation requires a real value, but got a complex value")]
    NotReal,

    #[error("operation requires a Gaussian integer, but got a rational")]
    NotGaussianInteger,

    #[error("operation requires an integer, but got a non-integer")]
    NotInteger,

    #[error("overflow: {0}")]
    Overflow(String),

    #[error("cannot reassign a PramanaObject ID once it has been set")]
    IdAlreadyAssigned,
}

pub type PramanaResult<T> = Result<T, PramanaError>;
