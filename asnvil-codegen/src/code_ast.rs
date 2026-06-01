use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeRef>,
    pub body: TemplateRef,
}

#[derive(Debug, Clone, Serialize)]
pub enum CodeAstNode {
    Module {
        name: String,
        imports: Vec<Import>,
        declarations: Vec<Declaration>,
        doc_comment: Option<String>,
    },
    Declaration(Box<Declaration>),
}

#[derive(Debug, Clone, Serialize)]
pub struct Import {
    pub module: String,
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum Declaration {
    Struct {
        name: String,
        fields: Vec<Field>,
        doc_comment: Option<String>,
        annotations: Vec<String>,
    },
    Enum {
        name: String,
        variants: Vec<EnumVariant>,
        repr: Option<EnumRepr>,
        doc_comment: Option<String>,
    },
    Choice {
        name: String,
        alternatives: Vec<Field>,
        doc_comment: Option<String>,
    },
    TypeAlias {
        name: String,
        target: TypeRef,
    },
    ListType {
        name: String,
        element_type: TypeRef,
        ber: BerFieldInfo,
        doc_comment: Option<String>,
    },
    Constant {
        name: String,
        ty: TypeRef,
        value: ValueLiteral,
    },
    FunctionDecl(Box<Function>),
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BerFieldInfo {
    pub encoding: String,
    pub tag_class: String,
    pub tag_number: u32,
    pub constructed: bool,
    pub string_encoding: String,
    pub referenced_type: Option<String>,
    pub list_element_ber: Option<Box<BerFieldInfo>>,
    pub tagging_mode: String,
    pub inherent_tag_class: String,
    pub inherent_tag_number: u32,
    pub defined_by: Option<String>,
    pub choice_alternative_tags: Vec<ChoiceAltTag>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChoiceAltTag {
    pub tag_class: String,
    pub tag_number: u32,
    pub constructed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Field {
    pub name: String,
    pub ty: TypeRef,
    pub default: Option<ValueLiteral>,
    pub optional: bool,
    pub doc_comment: Option<String>,
    pub ber: Option<BerFieldInfo>,
    pub order: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<i64>,
    pub doc_comment: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub enum EnumRepr {
    Int,
}

#[derive(Debug, Clone, Serialize)]
pub enum TypeRef {
    Builtin(BuiltinType),
    Named(String),
    Optional(Box<TypeRef>),
    List(Box<TypeRef>),
}

#[derive(Debug, Clone, Serialize)]
pub enum BuiltinType {
    Int { bits: Option<u8>, signed: bool },
    Bool,
    String,
    Bytes,
    Float,
    None,
    Any,
}

#[derive(Debug, Clone, Serialize)]
pub struct Param {
    pub name: String,
    pub ty: TypeRef,
}

#[derive(Debug, Clone, Serialize)]
pub struct TemplateRef {
    pub template_name: String,
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub enum ValueLiteral {
    Int(i64),
    Bool(bool),
    String(String),
    Bytes(Vec<u8>),
    None,
    Any,
}
