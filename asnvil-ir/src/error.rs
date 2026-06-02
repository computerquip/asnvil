use miette::Diagnostic;
use thiserror::Error;

use crate::from_ast::ConversionError;

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

    #[error("Imported symbol '{0}' not found in module '{1}' (imported by '{2}')")]
    #[diagnostic(code(asnvil::ir::imported_symbol_not_found))]
    ImportedSymbolNotFound(String, String, String),

    #[error("AST to IR conversion failed: {0}")]
    #[diagnostic(code(asnvil::ir::conversion_error))]
    ConversionError(String),
}

impl From<ConversionError> for IrError {
    fn from(e: ConversionError) -> Self {
        IrError::ConversionError(e.to_string())
    }
}
