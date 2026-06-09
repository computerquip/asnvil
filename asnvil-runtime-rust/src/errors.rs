use thiserror::Error;
use crate::types::Tag;

#[derive(Debug, Error)]
pub enum AsnError {
    #[error("Unexpected tag: expected {expected:?}, got {actual:?}")]
    UnexpectedTag { expected: Tag, actual: Tag },

    #[error("Invalid length encoding")]
    InvalidLength,

    #[error("Invalid integer encoding")]
    InvalidIntegerEncoding,

    #[error("Truncated input data")]
    TruncatedInput,

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Unknown choice variant")]
    UnknownChoiceVariant,

    #[error("Non-minimal length encoding (DER-specific)")]
    NonMinimalLength,

    #[error("Indefinite length not allowed in DER")]
    IndefiniteLengthNotAllowed,

    #[error("SET not in canonical order (DER-specific)")]
    SetNotCanonical,

    #[error("Invalid tag encoding")]
    InvalidTag,
}
