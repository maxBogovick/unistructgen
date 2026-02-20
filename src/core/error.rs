use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Type inference error: {0}")]
    TypeInference(String),

    #[error("Invalid field name: {0}")]
    InvalidFieldName(String),

    #[error("Invalid type reference: {0}")]
    InvalidTypeRef(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;
