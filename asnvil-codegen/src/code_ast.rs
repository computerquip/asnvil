use serde::Serialize;

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
        alternatives: Vec<ChoiceAlternative>,
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
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BerFieldInfo {
    pub encoding: EncodingType,
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
    pub constraints: Vec<ConstraintValidation>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConstraintValidation {
    pub field: String,
    pub kind: ConstraintKind,
}

#[derive(Debug, Clone, Serialize)]
pub enum ConstraintKind {
    IntegerRange { min: Option<i64>, max: Option<i64> },
    SizeRange { min: Option<usize>, max: Option<usize> },
    SingleValue { value: ValueLiteral },
}

#[derive(Debug, Clone, Serialize)]
pub struct ChoiceAltTag {
    pub tag_class: String,
    pub tag_number: u32,
    pub constructed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Tag {
    pub class: String,
    pub number: u32,
    pub constructed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub enum EncodeStmt {
    WriteInteger { name: String, tag: Tag, value: String },
    WriteEnumerated { name: String, tag: Tag, value: String },
    WriteBoolean { name: String, tag: Tag, value: String },
    WriteString { name: String, tag: Tag, value: String, encoding: String },
    WriteBytes { name: String, tag: Tag, value: String, tlv_method: String },
    WriteBitString { name: String, tag: Tag, value: String },
    WriteOid { name: String, tag: Tag, value: String },
    WriteNull { name: String, tag: Tag },
    WriteReal { name: String, tag: Tag, value: String },
    WriteTime { name: String, tag: Tag, value: String },
    WriteAny { name: String, value: String },
    WriteReferenced { name: String, tag: Tag, inner_type: String, encode_method: String, value: String },
    WriteChoice { name: String, tag: Tag, inner_type: String, encode_method: String, value: String },
    WriteList { name: String, tag: Tag, value: String, element_info: ListElementEncodeInfo },
    WrapExplicit { outer_tag: Tag, inner_stmt: Box<EncodeStmt> },
    WrapImplicit { outer_tag: Tag, inner_stmt: Box<EncodeStmt>, tag_number: u32 },
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ListElementEncodeInfo {
    pub encoding: EncodingType,
    pub tag_number: u32,
    pub string_encoding: String,
    pub referenced_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum DecodeStmt {
    ReadInteger { name: String },
    ReadEnumerated { name: String },
    ReadBoolean { name: String },
    ReadString { name: String, encoding: String },
    ReadBytes { name: String },
    ReadBitString { name: String },
    ReadOid { name: String },
    ReadNull { name: String },
    ReadReal { name: String },
    ReadTime { name: String },
    ReadAny { name: String, reconstruct_tlv: bool },
    ReadReferenced { name: String, inner_type: String, decode_method: String, reconstruct_tlv: bool },
    ReadChoice { name: String, inner_type: String, decode_method: String, reconstruct_tlv: bool },
    ReadList { name: String, element_info: ListElementDecodeInfo },
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ListElementDecodeInfo {
    pub encoding: EncodingType,
    pub string_encoding: String,
    pub referenced_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Field {
    pub name: String,
    pub ty: TypeRef,
    pub default: Option<ValueLiteral>,
    pub optional: bool,
    pub doc_comment: Option<String>,
    pub ber: Option<BerFieldInfo>,
    pub encode_stmts: Vec<EncodeStmt>,
    pub decode_stmts: Vec<DecodeStmt>,
    pub order: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChoiceAlternative {
    pub name: String,
    pub ty: TypeRef,
    pub ber: Option<BerFieldInfo>,
    pub encode_stmts: Vec<EncodeStmt>,
    pub decode_stmts: Vec<DecodeStmt>,
    pub tagging_mode: String,
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
pub enum StringEncoding {
    UTF8,
    Numeric,
    Printable,
    IA5,
    Teletex,
    BMP,
    Universal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub enum EncodingType {
    #[default]
    Integer,
    Boolean,
    String,
    Bytes,
    BitString,
    Oid,
    Null,
    Real,
    Time,
    Enumerated,
    Constructed,
    Choice,
    List,
    Referenced,
    Any,
}

impl EncodingType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EncodingType::Integer => "integer",
            EncodingType::Boolean => "boolean",
            EncodingType::String => "string",
            EncodingType::Bytes => "bytes",
            EncodingType::BitString => "bit_string",
            EncodingType::Oid => "oid",
            EncodingType::Null => "null",
            EncodingType::Real => "real",
            EncodingType::Time => "time",
            EncodingType::Enumerated => "enumerated",
            EncodingType::Constructed => "constructed",
            EncodingType::Choice => "choice",
            EncodingType::List => "list",
            EncodingType::Referenced => "referenced",
            EncodingType::Any => "any",
        }
    }
}

impl std::fmt::Display for EncodingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum BuiltinType {
    Integer,
    Boolean,
    String(StringEncoding),
    OctetString,
    BitString,
    ObjectIdentifier,
    Null,
    Real,
    GeneralizedTime,
    UTCTime,
    Any,
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
