use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
pub enum IrError {
    #[error("Type '{0}' not found")]
    #[diagnostic(code(asn1c::ir::type_not_found))]
    TypeNotFound(String),

    #[error("Circular type reference detected: {0}")]
    #[diagnostic(code(asn1c::ir::circular_reference))]
    CircularReference(String),

    #[error("Unknown module '{0}' in import")]
    #[diagnostic(code(asn1c::ir::unknown_module))]
    UnknownModule(String),

    #[error("Symbol '{0}' not exported by module '{1}'")]
    #[diagnostic(code(asn1c::ir::unexported_symbol))]
    UnexportedSymbol(String, String),

    #[error("Constraint violation: {0}")]
    #[diagnostic(code(asn1c::ir::constraint_violation))]
    ConstraintViolation(String),

    #[error("Value type mismatch: expected {expected}, got {actual}")]
    #[diagnostic(code(asn1c::ir::value_type_mismatch))]
    ValueTypeMismatch { expected: String, actual: String },

    #[error("Invalid tag specification: {0}")]
    #[diagnostic(code(asn1c::ir::invalid_tag))]
    InvalidTag(String),

    #[error("Extension marker error: {0}")]
    #[diagnostic(code(asn1c::ir::extension_error))]
    ExtensionError(String),
}
