use num_bigint::BigInt;

#[derive(Debug, Clone)]
pub struct AsnModule {
    pub name: String,
    pub oid: Option<ObjectIdentifier>,
    pub tag_default: TagDefault,
    pub ext_default: bool,
    pub exports: Exports,
    pub imports: Vec<Import>,
    pub types: Vec<TypeAssignment>,
    pub values: Vec<ValueAssignment>,
    pub object_classes: Vec<ObjectClassAssignment>,
    pub objects: Vec<ObjectAssignment>,
    pub object_sets: Vec<ObjectSetAssignment>,
}

#[derive(Debug, Clone)]
pub enum TagDefault {
    Explicit,
    Implicit,
    Automatic,
}

#[derive(Debug, Clone)]
pub enum Exports {
    All,
    Symbols(Vec<String>),
    None,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub symbols: Vec<String>,
    pub module: String,
    pub module_oid: Option<ObjectIdentifier>,
}

#[derive(Debug, Clone)]
pub struct TypeAssignment {
    pub name: String,
    pub ty: AsnType,
    pub parameters: Option<Vec<Parameter>>,
}

#[derive(Debug, Clone)]
pub struct ValueAssignment {
    pub name: String,
    pub ty: AsnType,
    pub value: AsnValue,
}

#[derive(Debug, Clone)]
pub struct ObjectClassAssignment {
    pub name: String,
    pub fields: Vec<ObjectClassField>,
}

#[derive(Debug, Clone)]
pub struct ObjectAssignment {
    pub name: String,
    pub class: String,
    pub fields: Vec<ObjectFieldValue>,
}

#[derive(Debug, Clone)]
pub struct ObjectSetAssignment {
    pub name: String,
    pub class: String,
    pub members: Vec<ObjectSetMember>,
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
pub enum AsnType {
    Boolean,
    Integer { named_numbers: Vec<(String, BigInt)>, constraints: Constraints },
    Real,
    Enumerated { root: Vec<EnumItem>, ext: Option<Vec<EnumItem>> },
    BitString { named_bits: Vec<(String, BigInt)>, constraints: Constraints },
    OctetString { constraints: Constraints },
    Null,
    ObjectIdentifier,
    RelativeOid,
    Sequence { fields: Vec<SequenceField>, ext: Option<Vec<SequenceField>> },
    Set { fields: Vec<SequenceField>, ext: Option<Vec<SequenceField>> },
    SequenceOf { element_type: Box<AsnType> },
    SetOf { element_type: Box<AsnType> },
    Choice { alternatives: Vec<ChoiceAlternative>, ext: Option<Vec<ChoiceAlternative>> },
    Tagged { class: TagClass, number: u32, implicit: bool, inner: Box<AsnType> },
    RestrictedString(CharsetType, Constraints),
    UnrestrictedString { constraints: Constraints },
    GeneralizedTime,
    UTCTime,
    ReferencedType { module: Option<String>, name: String },
    ConstrainedType { base: Box<AsnType>, constraints: Constraints },
    OpenType { defined_by: Option<String> },
    Any,
}

#[derive(Debug, Clone)]
pub enum TagClass {
    Universal,
    Application,
    Private,
    ContextSpecific,
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
pub struct SequenceField {
    pub name: String,
    pub ty: AsnType,
    pub optional: bool,
    pub default: Option<AsnValue>,
}

#[derive(Debug, Clone)]
pub struct ChoiceAlternative {
    pub name: String,
    pub ty: AsnType,
}

#[derive(Debug, Clone)]
pub struct EnumItem {
    pub name: String,
    pub value: BigInt,
}

#[derive(Debug, Clone, Default)]
pub struct Constraints {
    pub subtypes: Vec<SubtypeConstraint>,
}

#[derive(Debug, Clone)]
pub enum SubtypeConstraint {
    SingleValue(AsnValue),
    ValueRange { min: ConstraintValue, max: ConstraintValue },
    Size(Box<Constraints>),
    PermittedAlphabet(Box<Constraints>),
    ContainedSubtype(AsnType),
}

#[derive(Debug, Clone)]
pub enum ConstraintValue {
    Min,
    Max,
    Value(AsnValue),
}

#[derive(Debug, Clone)]
pub enum AsnValue {
    Boolean(bool),
    Integer(BigInt),
    BitString { unused_bits: u8, bytes: Vec<u8> },
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
