use crate::code_ast::*;
use crate::renderer::LanguageRenderer;
use anyhow::{bail, Result};
use askama::Template;

#[derive(Template)]
#[template(path = "python/module_header.txt")]
struct ModuleTemplate<'a> {
    declarations: &'a str,
    doc_comment: &'a str,
}

#[derive(Template)]
#[template(path = "python/struct.txt")]
struct StructTemplate<'a> {
    name: &'a str,
    fields: Vec<FieldContext<'a>>,
    doc_comment: &'a str,
    annotations: &'a Vec<String>,
}

#[derive(Template)]
#[template(path = "python/enum.txt")]
struct EnumTemplate<'a> {
    name: &'a str,
    variants: Vec<VariantContext<'a>>,
    repr: &'a str,
    doc_comment: &'a str,
}

#[derive(Template)]
#[template(path = "python/choice.txt")]
struct ChoiceTemplate<'a> {
    name: &'a str,
    alternatives: Vec<AlternativeContext<'a>>,
    doc_comment: &'a str,
}

#[derive(Template)]
#[template(path = "python/type_alias.txt")]
struct TypeAliasTemplate<'a> {
    name: &'a str,
    target: String,
}

#[derive(Template)]
#[template(path = "python/list_type.txt")]
struct ListTypeTemplate<'a> {
    name: &'a str,
    element_type: String,
    ber: BerContext<'a>,
    doc_comment: &'a str,
}

#[derive(Clone)]
struct FieldContext<'a> {
    name: &'a str,
    ty: String,
    optional: bool,
    default: String,
    ber: BerContext<'a>,
    has_ber: bool,
    order: usize,
}

#[derive(Clone)]
struct ChoiceAltTagContext<'a> {
    tag_class: &'a str,
    tag_number: u32,
    constructed: bool,
}

#[derive(Clone)]
struct BerContext<'a> {
    encoding: &'a str,
    tag_class: &'a str,
    tag_number: u32,
    constructed: bool,
    string_encoding: &'a str,
    referenced_type: &'a str,
    list_element_ber: Vec<BerContext<'a>>,
    tagging_mode: &'a str,
    inherent_tag_class: &'a str,
    inherent_tag_number: u32,
    choice_alternative_tags: Vec<ChoiceAltTagContext<'a>>,
}

#[derive(Clone)]
struct VariantContext<'a> {
    name: &'a str,
    has_value: bool,
    value: i64,
}

#[derive(Clone)]
struct AlternativeContext<'a> {
    name: &'a str,
    ty: String,
    ber: BerContext<'a>,
    has_ber: bool,
}

pub struct PythonRenderer;

impl PythonRenderer {
    pub fn new() -> Self {
        Self
    }

    fn ber_to_context(b: &BerFieldInfo) -> BerContext<'_> {
        BerContext {
            encoding: &b.encoding,
            tag_class: &b.tag_class,
            tag_number: b.tag_number,
            constructed: b.constructed,
            string_encoding: &b.string_encoding,
            referenced_type: b.referenced_type.as_deref().unwrap_or(""),
            list_element_ber: b.list_element_ber.as_ref().map(|inner| vec![Self::ber_to_context(inner)]).unwrap_or_default(),
            tagging_mode: &b.tagging_mode,
            inherent_tag_class: &b.inherent_tag_class,
            inherent_tag_number: b.inherent_tag_number,
            choice_alternative_tags: b.choice_alternative_tags.iter().map(|t| ChoiceAltTagContext {
                tag_class: &t.tag_class,
                tag_number: t.tag_number,
                constructed: t.constructed,
            }).collect(),
        }
    }

    fn value_literal_to_python(v: &ValueLiteral) -> String {
        match v {
            ValueLiteral::Int(n) => n.to_string(),
            ValueLiteral::Bool(true) => "True".to_string(),
            ValueLiteral::Bool(false) => "False".to_string(),
            ValueLiteral::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            ValueLiteral::Bytes(b) => format!("b'{}'", b.iter().map(|byte| format!("\\x{:02x}", byte)).collect::<String>()),
            ValueLiteral::None => "None".to_string(),
            ValueLiteral::Any => "None".to_string(),
        }
    }
}

impl Default for PythonRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageRenderer for PythonRenderer {
    fn language_name(&self) -> &str {
        "python"
    }

    fn render_module(&self, ast: &CodeAstNode) -> Result<String> {
        match ast {
            CodeAstNode::Module {
                name: _,
                imports: _,
                declarations,
                doc_comment,
            } => {
                let decls: Vec<String> = declarations
                    .iter()
                    .map(|d| self.render_declaration(d).unwrap())
                    .collect();
                let decls_str = decls.join("\n\n");
                let tmpl = ModuleTemplate {
                    declarations: &decls_str,
                    doc_comment: doc_comment.as_deref().unwrap_or(""),
                };
                tmpl.render().map_err(|e| anyhow::anyhow!("Failed to render module template: {}", e))
            }
            _ => bail!("Expected Module node"),
        }
    }

    fn render_declaration(&self, decl: &Declaration) -> Result<String> {
        match decl {
            Declaration::Struct {
                name,
                fields,
                doc_comment,
                annotations,
            } => {
                let mut rendered_fields: Vec<_> = fields.iter().map(|f| {
                    let ty_str = self.render_type(&f.ty).unwrap_or_else(|_| "Any".to_string());
                    let ber_ctx = f.ber.as_ref().map(Self::ber_to_context).unwrap_or_else(|| BerContext {
                        encoding: "",
                        tag_class: "",
                        tag_number: 0,
                        constructed: false,
                        string_encoding: "",
                        referenced_type: "",
                        list_element_ber: Vec::new(),
                        tagging_mode: "",
                        inherent_tag_class: "",
                        inherent_tag_number: 0,
                        choice_alternative_tags: Vec::new(),
                    });
                    let default_str = f.default.as_ref()
                        .map(Self::value_literal_to_python)
                        .unwrap_or_default();
                    FieldContext {
                        name: &f.name,
                        ty: ty_str,
                        optional: f.optional,
                        default: default_str,
                        ber: ber_ctx,
                        has_ber: f.ber.is_some(),
                        order: f.order,
                    }
                }).collect();
                rendered_fields.sort_by_key(|f| f.order);
                let tmpl = StructTemplate {
                    name,
                    fields: rendered_fields,
                    doc_comment: doc_comment.as_deref().unwrap_or(""),
                    annotations,
                };
                tmpl.render().map_err(|e| anyhow::anyhow!("Failed to render struct template: {}", e))
            }
            Declaration::Enum {
                name,
                variants,
                repr,
                doc_comment,
            } => {
                let rendered_variants: Vec<_> = variants.iter().map(|v| {
                    VariantContext {
                        name: &v.name,
                        has_value: v.value.is_some(),
                        value: v.value.unwrap_or(0),
                    }
                }).collect();
                let repr_str = match repr {
                    Some(crate::code_ast::EnumRepr::Int) => "IntEnum",
                    None => "Enum",
                };
                let tmpl = EnumTemplate {
                    name,
                    variants: rendered_variants,
                    repr: &repr_str,
                    doc_comment: doc_comment.as_deref().unwrap_or(""),
                };
                tmpl.render().map_err(|e| anyhow::anyhow!("Failed to render enum template: {}", e))
            }
            Declaration::Choice {
                name,
                alternatives,
                doc_comment,
            } => {
                let rendered_alts: Vec<_> = alternatives.iter().map(|a| {
                    let ty_str = self.render_type(&a.ty).unwrap_or_else(|_| "Any".to_string());
                    let ber_ctx = a.ber.as_ref().map(Self::ber_to_context).unwrap_or_else(|| BerContext {
                        encoding: "",
                        tag_class: "",
                        tag_number: 0,
                        constructed: false,
                        string_encoding: "",
                        referenced_type: "",
                        list_element_ber: Vec::new(),
                        tagging_mode: "",
                        inherent_tag_class: "",
                        inherent_tag_number: 0,
                        choice_alternative_tags: Vec::new(),
                    });
                    AlternativeContext {
                        name: &a.name,
                        ty: ty_str,
                        ber: ber_ctx,
                        has_ber: a.ber.is_some(),
                    }
                }).collect();
                let tmpl = ChoiceTemplate {
                    name,
                    alternatives: rendered_alts,
                    doc_comment: doc_comment.as_deref().unwrap_or(""),
                };
                tmpl.render().map_err(|e| anyhow::anyhow!("Failed to render choice template: {}", e))
            }
            Declaration::TypeAlias { name, target } => {
                let tmpl = TypeAliasTemplate {
                    name,
                    target: self.render_type(target)?,
                };
                tmpl.render().map_err(|e| anyhow::anyhow!("Failed to render type alias template: {}", e))
            }
            Declaration::ListType { name, element_type, ber, doc_comment } => {
                let ty_str = self.render_type(element_type)?;
                let ber_ctx = Self::ber_to_context(ber);
                let tmpl = ListTypeTemplate {
                    name,
                    element_type: ty_str,
                    ber: ber_ctx,
                    doc_comment: doc_comment.as_deref().unwrap_or(""),
                };
                tmpl.render().map_err(|e| anyhow::anyhow!("Failed to render list type template: {}", e))
            }
            Declaration::Constant { .. } => {
                bail!("Constants not yet supported in Python renderer")
            }
            Declaration::FunctionDecl(func) => {
                self.render_function(func, "function")
            }
        }
    }

    fn render_type(&self, ty: &TypeRef) -> Result<String> {
        match ty {
            TypeRef::Builtin(builtin) => match builtin {
                BuiltinType::Int { .. } => Ok("int".to_string()),
                BuiltinType::Bool => Ok("bool".to_string()),
                BuiltinType::String => Ok("str".to_string()),
                BuiltinType::Bytes => Ok("bytes".to_string()),
                BuiltinType::Float => Ok("float".to_string()),
                BuiltinType::None => Ok("None".to_string()),
                BuiltinType::Any => Ok("Any".to_string()),
            },
            TypeRef::Named(name) => Ok(name.clone()),
            TypeRef::Optional(inner) => {
                let inner_type = self.render_type(inner)?;
                Ok(format!("Optional[{}]", inner_type))
            }
            TypeRef::List(inner) => {
                let inner_type = self.render_type(inner)?;
                Ok(format!("list[{}]", inner_type))
            }
        }
    }

    fn render_function(&self, _func: &Function, _template: &str) -> Result<String> {
        bail!("Function rendering not yet implemented for Askama")
    }

    fn runtime_imports(&self) -> Vec<String> {
        vec![
            "from asn1c_runtime import AsnType, Tag, TagClass, BerEncoder, BerDecoder, DerEncoder, DerDecoder, AsnError".to_string(),
            "from dataclasses import dataclass, field".to_string(),
            "from typing import Optional".to_string(),
        ]
    }
}
