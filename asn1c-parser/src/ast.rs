use miette::SourceSpan;
use num_bigint::BigInt;

pub trait Spanned {
    fn span(&self) -> SourceSpan;
}

#[derive(Debug, Clone)]
pub struct Module {
    pub identifier: ModuleIdentifier,
    pub tag_default: Option<TagDefault>,
    pub ext_default: bool,
    pub body: ModuleBody,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ModuleIdentifier {
    pub name: String,
    pub oid: Option<ObjectIdentifier>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum TagDefault {
    Explicit,
    Implicit,
    Automatic,
}

#[derive(Debug, Clone)]
pub struct ModuleBody {
    pub exports: Option<Exports>,
    pub imports: Vec<Import>,
    pub assignments: Vec<Assignment>,
}

#[derive(Debug, Clone)]
pub struct Exports {
    pub symbols: ExportSymbols,
}

#[derive(Debug, Clone)]
pub enum ExportSymbols {
    All,
    Symbols(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct Import {
    pub symbols: Vec<String>,
    pub module: String,
    pub module_oid: Option<ObjectIdentifier>,
}

#[derive(Debug, Clone)]
pub enum Assignment {
    Type(TypeAssignment),
    Value(ValueAssignment),
    ValueSetType(ValueSetTypeAssignment),
    ObjectClass(ObjectClassAssignment),
    Object(ObjectAssignment),
    ObjectSet(ObjectSetAssignment),
    ParameterizedType(ParameterizedTypeAssignment),
    ParameterizedValue(ParameterizedValueAssignment),
}

#[derive(Debug, Clone)]
pub struct TypeAssignment {
    pub name: String,
    pub parameters: Option<ParameterList>,
    pub ty: AsnType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ValueAssignment {
    pub name: String,
    pub ty: AsnType,
    pub value: AsnValue,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ValueSetTypeAssignment {
    pub name: String,
    pub value_set: Vec<AsnValue>,
    pub ty: AsnType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ObjectClassAssignment {
    pub name: String,
    pub fields: Vec<ObjectClassField>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ObjectAssignment {
    pub name: String,
    pub class: String,
    pub fields: Vec<ObjectFieldValue>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ObjectSetAssignment {
    pub name: String,
    pub class: String,
    pub members: Vec<ObjectSetMember>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ParameterizedTypeAssignment {
    pub name: String,
    pub parameters: ParameterList,
    pub ty: AsnType,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ParameterizedValueAssignment {
    pub name: String,
    pub ty: AsnType,
    pub parameters: ParameterList,
    pub value: AsnValue,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct ParameterList {
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub governor: Option<ParameterGovernor>,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum ParameterGovernor {
    Type(AsnType),
    ObjectClass(String),
    ValueSet(Vec<AsnValue>),
}

#[derive(Debug, Clone)]
pub struct ObjectClassField {
    pub name: String,
    pub ty: Option<AsnType>,
    pub default: Option<AsnValue>,
}

#[derive(Debug, Clone)]
pub struct ObjectFieldValue {
    pub field: String,
    pub value: AsnValue,
}

#[derive(Debug, Clone)]
pub enum ObjectSetMember {
    Reference(String),
    Nested(Vec<ObjectSetMember>),
    Except(Box<ObjectSetMember>),
}

#[derive(Debug, Clone)]
pub enum AsnType {
    Boolean { span: SourceSpan },
    Integer { named_numbers: Option<Vec<NamedNumber>>, span: SourceSpan },
    Real { span: SourceSpan },
    Enumerated { items: Vec<EnumItem>, extensible: bool, ext_items: Vec<EnumItem>, span: SourceSpan },
    BitString { named_bits: Option<Vec<NamedBit>>, span: SourceSpan },
    OctetString { span: SourceSpan },
    Null { span: SourceSpan },
    Sequence { fields: Vec<ComponentType>, extensible: bool, ext_fields: Vec<ComponentType>, span: SourceSpan },
    Set { fields: Vec<ComponentType>, extensible: bool, ext_fields: Vec<ComponentType>, span: SourceSpan },
    Choice { alternatives: Vec<NamedType>, extensible: bool, ext_alternatives: Vec<NamedType>, span: SourceSpan },
    SequenceOf { element_type: Box<AsnType>, span: SourceSpan },
    SetOf { element_type: Box<AsnType>, span: SourceSpan },
    Tagged { class: Option<TagClass>, number: BigInt, implicit: Option<bool>, inner: Box<AsnType>, span: SourceSpan },
    ObjectIdentifier { span: SourceSpan },
    RelativeOid { span: SourceSpan },
    RestrictedString { charset: CharsetType, span: SourceSpan },
    UnrestrictedString { span: SourceSpan },
    GeneralizedTime { span: SourceSpan },
    UTCTime { span: SourceSpan },
    Any { span: SourceSpan },
    OpenType { defined_by: Option<String>, span: SourceSpan },
    Constrained { base: Box<AsnType>, constraint: Constraint, span: SourceSpan },
    Referenced { name: String, parameters: Option<Vec<ActualParameter>>, span: SourceSpan },
}

impl Spanned for AsnType {
    fn span(&self) -> SourceSpan {
        match self {
            AsnType::Boolean { span } => *span,
            AsnType::Integer { span, .. } => *span,
            AsnType::Real { span } => *span,
            AsnType::Enumerated { span, .. } => *span,
            AsnType::BitString { span, .. } => *span,
            AsnType::OctetString { span } => *span,
            AsnType::Null { span } => *span,
            AsnType::Sequence { span, .. } => *span,
            AsnType::Set { span, .. } => *span,
            AsnType::Choice { span, .. } => *span,
            AsnType::SequenceOf { span, .. } => *span,
            AsnType::SetOf { span, .. } => *span,
            AsnType::Tagged { span, .. } => *span,
            AsnType::ObjectIdentifier { span } => *span,
            AsnType::RelativeOid { span } => *span,
            AsnType::RestrictedString { span, .. } => *span,
            AsnType::UnrestrictedString { span } => *span,
            AsnType::GeneralizedTime { span } => *span,
            AsnType::UTCTime { span } => *span,
            AsnType::Any { span } => *span,
            AsnType::OpenType { span, .. } => *span,
            AsnType::Constrained { span, .. } => *span,
            AsnType::Referenced { span, .. } => *span,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TagClass {
    Universal,
    Application,
    Private,
}

#[derive(Debug, Clone)]
pub enum CharsetType {
    UTF8,
    Numeric,
    Printable,
    Teletex,
    Videotex,
    IA5,
    Graphic,
    Visible,
    General,
    Universal,
    BMP,
}

#[derive(Debug, Clone)]
pub struct NamedNumber {
    pub name: String,
    pub value: BigInt,
}

#[derive(Debug, Clone)]
pub struct NamedBit {
    pub name: String,
    pub value: BigInt,
}

#[derive(Debug, Clone)]
pub struct EnumItem {
    pub name: String,
    pub value: Option<BigInt>,
}

#[derive(Debug, Clone)]
pub struct ComponentType {
    pub name: String,
    pub ty: AsnType,
    pub optional: bool,
    pub default: Option<AsnValue>,
}

#[derive(Debug, Clone)]
pub struct NamedType {
    pub name: String,
    pub ty: AsnType,
}

#[derive(Debug, Clone)]
pub struct Constraint {
    pub spec: Box<ConstraintSpec>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum ConstraintSpec {
    Single(Box<SingleElementConstraint>),
    Union(Vec<ElementSet>),
}

#[derive(Debug, Clone)]
pub enum SingleElementConstraint {
    Value(AsnValue),
    ValueRange(ValueRange),
    Size(Box<Constraint>),
    PermittedAlphabet(Box<Constraint>),
    ContainedSubtype(AsnType),
}

#[derive(Debug, Clone)]
pub struct ValueRange {
    pub min: RangeValue,
    pub max: RangeValue,
}

#[derive(Debug, Clone)]
pub enum RangeValue {
    Min,
    Max,
    Value(AsnValue),
}

#[derive(Debug, Clone)]
pub struct ElementSet {
    pub intersections: Vec<ElementIntersection>,
}

#[derive(Debug, Clone)]
pub struct ElementIntersection {
    pub all: bool,
    pub except: bool,
    pub spec: Box<ConstraintSpec>,
}

#[derive(Debug, Clone)]
pub enum AsnValue {
    Boolean(bool),
    Integer(BigInt),
    BitString(Vec<u8>),
    HexString(Vec<u8>),
    CharString(String),
    Null,
    Sequence(Vec<NamedValue>),
    Choice { name: String, value: Box<AsnValue> },
    Enumerated(String),
    ObjectIdentifier(ObjectIdentifier),
    Referenced(String),
}

#[derive(Debug, Clone)]
pub struct NamedValue {
    pub name: String,
    pub value: AsnValue,
}

#[derive(Debug, Clone)]
pub struct ObjectIdentifier {
    pub components: Vec<OidComponent>,
}

#[derive(Debug, Clone)]
pub enum OidComponent {
    Name(String),
    Number(BigInt),
}

#[derive(Debug, Clone)]
pub enum ActualParameter {
    Type(AsnType),
    Value(AsnValue),
    Object(String),
    ValueSet(Vec<AsnValue>),
}
