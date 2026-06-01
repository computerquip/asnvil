use crate::code_ast::{Declaration, Function, TypeRef};
use anyhow::Result;

pub trait LanguageRenderer {
    fn language_name(&self) -> &str;
    fn render_module(&self, ast: &crate::code_ast::CodeAstNode) -> Result<String>;
    fn render_declaration(&self, decl: &Declaration) -> Result<String>;
    fn render_type(&self, ty: &TypeRef) -> Result<String>;
    fn render_function(&self, func: &Function, template: &str) -> Result<String>;
    fn runtime_imports(&self) -> Vec<String>;
}
