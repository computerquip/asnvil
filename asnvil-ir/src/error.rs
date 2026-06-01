use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
pub enum IrError {
    #[error("Type '{0}' not found")]
    #[diagnostic(code(asnvil::ir::type_not_found))]
    TypeNotFound(String),

    #[error("Circular type reference detected: {0}")]
    #[diagnostic(code(asnvil::ir::circular_reference))]
    CircularReference(String),

    #[error("Unknown module '{0}' in import")]
    #[diagnostic(code(asnvil::ir::unknown_module))]
    UnknownModule(String),

    #[error("Symbol '{0}' not exported by module '{1}'")]
    #[diagnostic(code(asnvil::ir::unexported_symbol))]
    UnexportedSymbol(String, String),

    #[error("Constraint violation: {0}")]
    #[diagnostic(code(asnvil::ir::constraint_violation))]
    ConstraintViolation(String),

    #[error("Value type mismatch: expected {expected}, got {actual}")]
    #[diagnostic(code(asnvil::ir::value_type_mismatch))]
    ValueTypeMismatch { expected: String, actual: String },

    #[error("Invalid tag specification: {0}")]
    #[diagnostic(code(asnvil::ir::invalid_tag))]
    InvalidTag(String),

    #[error("Extension marker error: {0}")]
    #[diagnostic(code(asnvil::ir::extension_error))]
    ExtensionError(String),
}
