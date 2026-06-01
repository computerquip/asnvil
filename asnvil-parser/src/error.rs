use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Diagnostic, Error)]
pub enum ParseError {
    #[error("Parse error: {message}")]
    #[diagnostic(code(asnvil::parse::syntax_error))]
    SyntaxError {
        message: String,
        #[label("error occurred here")]
        span: miette::SourceSpan,
    },
}
