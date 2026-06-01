use crate::code_ast::*;
use crate::code_ast::ChoiceAlternative as CodeChoiceAlt;
use asnvil_ir::ir::*;
use asnvil_ir::ir::ChoiceAlternative as IrChoiceAlt;
use std::collections::HashMap;

fn string_encoding_to_ir(charset: &CharsetType) -> StringEncoding {
    match charset {
        CharsetType::UTF8 => StringEncoding::UTF8,
        CharsetType::Numeric => StringEncoding::Numeric,
        CharsetType::Printable => StringEncoding::Printable,
        CharsetType::IA5 => StringEncoding::IA5,
        CharsetType::Teletex => StringEncoding::Teletex,
        CharsetType::BMP => StringEncoding::BMP,
        CharsetType::Universal => StringEncoding::Universal,
        CharsetType::Videotex | CharsetType::Graphic | CharsetType::Visible | CharsetType::General => StringEncoding::UTF8,
    }
}

pub struct CodeAstBuilder {
    types: HashMap<String, AsnType>,
}

impl Default for CodeAstBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn asn_value_to_literal(v: &AsnValue) -> ValueLiteral {
    match v {
        AsnValue::Boolean(b) => ValueLiteral::Bool(*b),
        AsnValue::Integer(n) => {
            if let Ok(i) = n.try_into() {
                ValueLiteral::Int(i)
            } else {
                ValueLiteral::Any
            }
        }
        AsnValue::CharString(s) => ValueLiteral::String(s.clone()),
        AsnValue::HexString(b) => ValueLiteral::Bytes(b.clone()),
        AsnValue::Null => ValueLiteral::None,
        AsnValue::Enumerated(name) => ValueLiteral::String(name.clone()),
        AsnValue::Referenced(name) => ValueLiteral::String(name.clone()),
        AsnValue::BitString { bytes, .. } => ValueLiteral::Bytes(bytes.clone()),
        _ => ValueLiteral::Any,
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn is_inline_type(ty: &AsnType) -> bool {
    matches!(ty, AsnType::Choice { .. } | AsnType::Sequence { .. } | AsnType::Set { .. })
}

fn tag_class_str(class: &TagClass) -> &str {
    match class {
        TagClass::Universal => "UNIVERSAL",
        TagClass::Application => "APPLICATION",
        TagClass::Private => "PRIVATE",
        TagClass::ContextSpecific => "CONTEXT",
    }
}

fn extract_alt_tag(ty: &AsnType) -> (String, u32, bool) {
    match ty {
        AsnType::Tagged { class, number, implicit, .. } => {
            (tag_class_str(class).to_string(), *number, !implicit)
        }
        AsnType::Boolean => ("UNIVERSAL".to_string(), 1, false),
        AsnType::Integer { .. } => ("UNIVERSAL".to_string(), 2, false),
        AsnType::BitString { .. } => ("UNIVERSAL".to_string(), 3, false),
        AsnType::OctetString => ("UNIVERSAL".to_string(), 4, false),
        AsnType::Null => ("UNIVERSAL".to_string(), 5, false),
        AsnType::ObjectIdentifier => ("UNIVERSAL".to_string(), 6, false),
        AsnType::Enumerated { .. } => ("UNIVERSAL".to_string(), 10, false),
        AsnType::Real => ("UNIVERSAL".to_string(), 9, false),
        AsnType::Sequence { .. } => ("UNIVERSAL".to_string(), 16, true),
        AsnType::Set { .. } => ("UNIVERSAL".to_string(), 17, true),
        AsnType::RestrictedString(charset) => {
            let tag = match charset {
                CharsetType::UTF8 => 12,
                CharsetType::Numeric => 18,
                CharsetType::Printable => 19,
                CharsetType::Teletex => 20,
                CharsetType::Videotex => 21,
                CharsetType::IA5 => 22,
                CharsetType::Graphic => 25,
                CharsetType::Visible => 26,
                CharsetType::General => 27,
                CharsetType::Universal => 28,
                CharsetType::BMP => 30,
            };
            ("UNIVERSAL".to_string(), tag, false)
        }
        AsnType::UnrestrictedString => ("UNIVERSAL".to_string(), 4, false),
        AsnType::GeneralizedTime => ("UNIVERSAL".to_string(), 24, false),
        AsnType::UTCTime => ("UNIVERSAL".to_string(), 23, false),
        AsnType::SequenceOf { .. } => ("UNIVERSAL".to_string(), 16, true),
        AsnType::SetOf { .. } => ("UNIVERSAL".to_string(), 17, true),
        AsnType::ReferencedType { .. } => ("UNIVERSAL".to_string(), 16, true),
        AsnType::Choice { .. } => ("UNIVERSAL".to_string(), 0, true),
        AsnType::ConstrainedType { base, .. } => extract_alt_tag(base),
        _ => ("UNIVERSAL".to_string(), 0, false),
    }
}

fn make_alt_tags(ty: &AsnType) -> Vec<ChoiceAltTag> {
    match ty {
        AsnType::Choice { alternatives, ext } => alternatives
            .iter()
            .chain(ext.iter().flat_map(|e| e.iter()))
            .map(|a| {
                let (tag_class, tag_number, constructed) = extract_alt_tag(&a.ty);
                ChoiceAltTag { tag_class, tag_number, constructed }
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn make_tags_for_type(ty: &AsnType) -> Vec<ChoiceAltTag> {
    if is_inline_type(ty) {
        make_alt_tags(ty)
    } else {
        Vec::new()
    }
}

impl CodeAstBuilder {
    pub fn new() -> Self {
        Self { types: HashMap::new() }
    }

    pub fn with_types(types: HashMap<String, AsnType>) -> Self {
        Self { types }
    }

    fn resolve_type<'a>(&'a self, ty: &'a AsnType) -> &'a AsnType {
        let mut current = ty;
        let mut visited = std::collections::HashSet::new();
        loop {
            if let AsnType::ReferencedType { name, .. } = current {
                if !visited.insert(name.clone()) {
                    return current;
                }
                if let Some(resolved) = self.types.get(name) {
                    current = resolved;
                } else {
                    return current;
                }
            } else {
                return current;
            }
        }
    }

    pub fn build_module(&self, module: &AsnModule) -> CodeAstNode {
        let builder = Self { types: module.types.iter().map(|t| (t.name.clone(), t.ty.clone())).collect() };
        let declarations: Vec<Declaration> = module
            .types
            .iter()
            .flat_map(|t| builder.build_type_assignment(t))
            .collect();

        CodeAstNode::Module {
            name: module.name.clone(),
            imports: vec![],
            declarations,
            doc_comment: Some(format!("Generated from {}.asn1 by asnvil.", module.name)),
        }
    }

    fn inline_type_name(parent: &str, field: &str) -> String {
        format!("{}{}", parent, capitalize(field))
    }

    fn build_inline_decl(&self, ty: &AsnType, name: &str) -> Vec<Declaration> {
        let mut decls = Vec::new();
        match ty {
            AsnType::Choice { alternatives, ext, .. } => {
                for a in alternatives.iter().chain(ext.iter().flat_map(|e| e.iter())) {
                    let resolved = self.resolve_type(&a.ty);
                    if is_inline_type(resolved) {
                        let child_name = Self::inline_type_name(name, &a.name);
                        decls.extend(self.build_inline_decl(resolved, &child_name));
                    }
                }
                let alt_fields: Vec<CodeChoiceAlt> = alternatives
                    .iter()
                    .enumerate()
                    .map(|(_, a)| {
                        let resolved = self.resolve_type(&a.ty);
                        let ty_ref = if is_inline_type(resolved) {
                            TypeRef::Named(Self::inline_type_name(name, &a.name))
                        } else {
                            self.build_type(&a.ty)
                        };
                        CodeChoiceAlt {
                            name: a.name.clone(),
                            ty: ty_ref,
                            ber: Some(self.ber_info_for_type(&a.ty)),
                            encode_stmts: Vec::new(),
                            decode_stmts: Vec::new(),
                            tagging_mode: "inherent".to_string(),
                        }
                    })
                    .chain(ext.iter().flat_map(|ea| ea.iter().enumerate().map(|(_, a)| {
                        let resolved = self.resolve_type(&a.ty);
                        let ty_ref = if is_inline_type(resolved) {
                            TypeRef::Named(Self::inline_type_name(name, &a.name))
                        } else {
                            self.build_type(&a.ty)
                        };
                        CodeChoiceAlt {
                            name: a.name.clone(),
                            ty: ty_ref,
                            ber: Some(self.ber_info_for_type(&a.ty)),
                            encode_stmts: Vec::new(),
                            decode_stmts: Vec::new(),
                            tagging_mode: "inherent".to_string(),
                        }
                    })))
                    .collect();
                decls.push(Declaration::Choice {
                    name: name.to_string(),
                    alternatives: alt_fields,
                    doc_comment: None,
                });
            }
            AsnType::Sequence { fields, ext, .. } => {
                let mut all_fields: Vec<&SequenceField> = fields.iter().collect();
                all_fields.extend(ext.iter().flat_map(|e| e.iter()));
                for f in &all_fields {
                    let resolved = self.resolve_type(&f.ty);
                    if is_inline_type(resolved) {
                        let child_name = Self::inline_type_name(name, &f.name);
                        decls.extend(self.build_inline_decl(resolved, &child_name));
                    }
                }
                let mut ast_fields: Vec<Field> = fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| self.build_seq_field(f, name, i))
                    .chain(ext.iter().flat_map(|ef| ef.iter().enumerate().map(|(i, f)| self.build_seq_field(f, name, i))))
                    .collect();
                ast_fields.sort_by_key(|f| {
                    if f.default.is_some() || f.optional { 1 } else { 0 }
                });
                decls.push(Declaration::Struct {
                    name: name.to_string(),
                    fields: ast_fields,
                    doc_comment: None,
                    annotations: vec!["sequence".to_string()],
                });
            }
            AsnType::Set { fields, ext, .. } => {
                let mut all_fields: Vec<&SequenceField> = fields.iter().collect();
                all_fields.extend(ext.iter().flat_map(|e| e.iter()));
                for f in &all_fields {
                    let resolved = self.resolve_type(&f.ty);
                    if is_inline_type(resolved) {
                        let child_name = Self::inline_type_name(name, &f.name);
                        decls.extend(self.build_inline_decl(resolved, &child_name));
                    }
                }
                let mut ast_fields: Vec<Field> = fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| self.build_seq_field(f, name, i))
                    .chain(ext.iter().flat_map(|ef| ef.iter().enumerate().map(|(i, f)| self.build_seq_field(f, name, i))))
                    .collect();
                ast_fields.sort_by_key(|f| {
                    if f.default.is_some() || f.optional { 1 } else { 0 }
                });
                decls.push(Declaration::Struct {
                    name: name.to_string(),
                    fields: ast_fields,
                    doc_comment: None,
                    annotations: vec!["set".to_string()],
                });
            }
            _ => {
                decls.push(Declaration::TypeAlias {
                    name: name.to_string(),
                    target: self.build_type(ty),
                });
            }
        }
        decls
    }

    fn build_seq_field(&self, f: &SequenceField, parent_name: &str, order: usize) -> Field {
        let resolved = self.resolve_type(&f.ty);
        let ty = if is_inline_type(resolved) {
            let gen_name = Self::inline_type_name(parent_name, &f.name);
            if f.optional || f.default.is_some() {
                TypeRef::Optional(Box::new(TypeRef::Named(gen_name)))
            } else {
                TypeRef::Named(gen_name)
            }
        } else if f.optional || f.default.is_some() {
            TypeRef::Optional(Box::new(self.build_type(&f.ty)))
        } else {
            self.build_type(&f.ty)
        };

        Field {
            name: f.name.clone(),
            ty,
            optional: f.optional || f.default.is_some(),
            default: f.default.as_ref().map(asn_value_to_literal),
            doc_comment: None,
            ber: Some(self.ber_info_for_field(&f.ty, parent_name, &f.name)),
            encode_stmts: Vec::new(),
            decode_stmts: Vec::new(),
            order,
        }
    }

    fn ber_info_for_field(&self, ty: &AsnType, parent_name: &str, field_name: &str) -> BerFieldInfo {
        let resolved = self.resolve_type(ty);
        if is_inline_type(resolved) {
            let gen_name = Self::inline_type_name(parent_name, field_name);
            match resolved {
                AsnType::Choice { .. } => {
                    let alt_tags = make_tags_for_type(resolved);
                    BerFieldInfo {
                        encoding: "choice".to_string(),
                        tag_class: "UNIVERSAL".to_string(),
                        tag_number: 0,
                        constructed: true,
                        string_encoding: String::new(),
                        referenced_type: Some(gen_name),
                        tagging_mode: "inherent".to_string(),
                        inherent_tag_class: "UNIVERSAL".to_string(),
                        inherent_tag_number: 0,
                        list_element_ber: None,
                        defined_by: None,
                        choice_alternative_tags: alt_tags,
                    }
                }
                AsnType::Sequence { .. } => BerFieldInfo {
                    encoding: "constructed".to_string(),
                    tag_class: "UNIVERSAL".to_string(),
                    tag_number: 16,
                    constructed: true,
                    string_encoding: String::new(),
                    referenced_type: Some(gen_name),
                    tagging_mode: "inherent".to_string(),
                    inherent_tag_class: "UNIVERSAL".to_string(),
                    inherent_tag_number: 16,
                    list_element_ber: None,
                    defined_by: None,
                    choice_alternative_tags: Vec::new(),
                },
                AsnType::Set { .. } => BerFieldInfo {
                    encoding: "constructed".to_string(),
                    tag_class: "UNIVERSAL".to_string(),
                    tag_number: 17,
                    constructed: true,
                    string_encoding: String::new(),
                    referenced_type: Some(gen_name),
                    tagging_mode: "inherent".to_string(),
                    inherent_tag_class: "UNIVERSAL".to_string(),
                    inherent_tag_number: 17,
                    list_element_ber: None,
                    defined_by: None,
                    choice_alternative_tags: Vec::new(),
                },
                _ => unreachable!(),
            }
        } else {
            self.ber_info_for_type(ty)
        }
    }

    fn build_type_assignment(&self, assignment: &TypeAssignment) -> Vec<Declaration> {
        match &assignment.ty {
            AsnType::Sequence { fields, ext, .. } => {
                let mut decls = Vec::new();
                let all_fields: Vec<&SequenceField> = fields.iter().chain(ext.iter().flat_map(|e| e.iter())).collect();
                for f in &all_fields {
                    let resolved = self.resolve_type(&f.ty);
                    if is_inline_type(resolved) {
                        let gen_name = Self::inline_type_name(&assignment.name, &f.name);
                        decls.extend(self.build_inline_decl(resolved, &gen_name));
                    }
                }

                let mut ast_fields: Vec<Field> = fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        let mut field = self.build_sequence_field_with_parent(f, &assignment.name);
                        field.order = i;
                        field
                    })
                    .chain(ext.iter().flat_map(|ef| ef.iter().map(|f| self.build_sequence_field_with_parent(f, &assignment.name))))
                    .collect();
                ast_fields.sort_by_key(|f| {
                    if f.default.is_some() || f.optional { 1 } else { 0 }
                });

                decls.push(Declaration::Struct {
                    name: assignment.name.clone(),
                    fields: ast_fields,
                    doc_comment: None,
                    annotations: vec!["sequence".to_string()],
                });
                decls
            }
            AsnType::Set { fields, ext, .. } => {
                let mut decls = Vec::new();
                let all_fields: Vec<&SequenceField> = fields.iter().chain(ext.iter().flat_map(|e| e.iter())).collect();
                for f in &all_fields {
                    let resolved = self.resolve_type(&f.ty);
                    if is_inline_type(resolved) {
                        let gen_name = Self::inline_type_name(&assignment.name, &f.name);
                        decls.extend(self.build_inline_decl(resolved, &gen_name));
                    }
                }

                let mut ast_fields: Vec<Field> = fields
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        let mut field = self.build_sequence_field_with_parent(f, &assignment.name);
                        field.order = i;
                        field
                    })
                    .chain(ext.iter().flat_map(|ef| ef.iter().map(|f| self.build_sequence_field_with_parent(f, &assignment.name))))
                    .collect();
                ast_fields.sort_by_key(|f| {
                    if f.default.is_some() || f.optional { 1 } else { 0 }
                });

                decls.push(Declaration::Struct {
                    name: assignment.name.clone(),
                    fields: ast_fields,
                    doc_comment: None,
                    annotations: vec!["set".to_string()],
                });
                decls
            }
            AsnType::Choice { alternatives, ext, .. } => {
                let mut decls = Vec::new();
                let all_alts: Vec<&IrChoiceAlt> = alternatives.iter().chain(ext.iter().flat_map(|e| e.iter())).collect();
                for a in &all_alts {
                    let resolved = self.resolve_type(&a.ty);
                    if is_inline_type(resolved) {
                        let gen_name = Self::inline_type_name(&assignment.name, &a.name);
                        decls.extend(self.build_inline_decl(resolved, &gen_name));
                    }
                }

                let alt_fields: Vec<CodeChoiceAlt> = alternatives
                    .iter()
                    .enumerate()
                    .map(|(_, a)| CodeChoiceAlt {
                        name: a.name.clone(),
                        ty: self.build_type(&a.ty),
                        ber: Some(self.ber_info_for_type(&a.ty)),
                        encode_stmts: Vec::new(),
                        decode_stmts: Vec::new(),
                        tagging_mode: "inherent".to_string(),
                    })
                    .chain(ext.iter().flat_map(|ea| ea.iter().enumerate().map(|(_, a)| CodeChoiceAlt {
                        name: a.name.clone(),
                        ty: self.build_type(&a.ty),
                        ber: Some(self.ber_info_for_type(&a.ty)),
                        encode_stmts: Vec::new(),
                        decode_stmts: Vec::new(),
                        tagging_mode: "inherent".to_string(),
                    })))
                    .collect();

                decls.push(Declaration::Choice {
                    name: assignment.name.clone(),
                    alternatives: alt_fields,
                    doc_comment: None,
                });
                decls
            }
            AsnType::Enumerated { root, ext, .. } => {
                let mut variants = Vec::new();
                let mut current_value = 0i64;
                for item in root {
                    variants.push(EnumVariant {
                        name: item.name.clone(),
                        value: Some(current_value),
                        doc_comment: None,
                    });
                    current_value += 1;
                }
                if let Some(ext_items) = ext {
                    for item in ext_items {
                        variants.push(EnumVariant {
                            name: item.name.clone(),
                            value: Some(current_value),
                            doc_comment: None,
                        });
                        current_value += 1;
                    }
                }
                vec![Declaration::Enum {
                    name: assignment.name.clone(),
                    variants,
                    repr: Some(EnumRepr::Int),
                    doc_comment: None,
                }]
            }
            AsnType::SequenceOf { element_type } => {
                vec![Declaration::ListType {
                    name: assignment.name.clone(),
                    element_type: self.build_type(element_type),
                    ber: self.ber_info_for_type(&AsnType::SequenceOf { element_type: element_type.clone() }),
                    doc_comment: None,
                }]
            }
            AsnType::SetOf { element_type } => {
                vec![Declaration::ListType {
                    name: assignment.name.clone(),
                    element_type: self.build_type(element_type),
                    ber: self.ber_info_for_type(&AsnType::SetOf { element_type: element_type.clone() }),
                    doc_comment: None,
                }]
            }
            AsnType::BitString { .. } => {
                vec![Declaration::TypeAlias {
                    name: assignment.name.clone(),
                    target: TypeRef::Builtin(BuiltinType::BitString),
                }]
            }
            AsnType::OctetString => {
                vec![Declaration::TypeAlias {
                    name: assignment.name.clone(),
                    target: TypeRef::Builtin(BuiltinType::OctetString),
                }]
            }
            AsnType::ObjectIdentifier => {
                vec![Declaration::TypeAlias {
                    name: assignment.name.clone(),
                    target: TypeRef::Builtin(BuiltinType::ObjectIdentifier),
                }]
            }
            AsnType::Boolean => {
                vec![Declaration::TypeAlias {
                    name: assignment.name.clone(),
                    target: TypeRef::Builtin(BuiltinType::Boolean),
                }]
            }
            AsnType::Integer { .. } => {
                vec![Declaration::TypeAlias {
                    name: assignment.name.clone(),
                    target: TypeRef::Builtin(BuiltinType::Integer),
                }]
            }
            AsnType::RestrictedString(charset) => {
                vec![Declaration::TypeAlias {
                    name: assignment.name.clone(),
                    target: TypeRef::Builtin(BuiltinType::String(string_encoding_to_ir(charset))),
                }]
            }
            AsnType::UnrestrictedString => {
                vec![Declaration::TypeAlias {
                    name: assignment.name.clone(),
                    target: TypeRef::Builtin(BuiltinType::OctetString),
                }]
            }
            AsnType::Null => {
                vec![Declaration::TypeAlias {
                    name: assignment.name.clone(),
                    target: TypeRef::Builtin(BuiltinType::Null),
                }]
            }
            AsnType::GeneralizedTime => {
                vec![Declaration::TypeAlias {
                    name: assignment.name.clone(),
                    target: TypeRef::Builtin(BuiltinType::GeneralizedTime),
                }]
            }
            AsnType::UTCTime => {
                vec![Declaration::TypeAlias {
                    name: assignment.name.clone(),
                    target: TypeRef::Builtin(BuiltinType::UTCTime),
                }]
            }
            AsnType::Tagged { .. } => {
                vec![Declaration::Struct {
                    name: assignment.name.clone(),
                    fields: vec![],
                    doc_comment: None,
                    annotations: vec!["tagged".to_string()],
                }]
            }
            AsnType::ConstrainedType { base, .. } => {
                vec![Declaration::TypeAlias {
                    name: assignment.name.clone(),
                    target: self.build_type(base),
                }]
            }
            AsnType::ReferencedType { name, .. } => {
                vec![Declaration::TypeAlias {
                    name: assignment.name.clone(),
                    target: TypeRef::Named(name.clone()),
                }]
            }
            AsnType::OpenType { .. } => {
                vec![Declaration::TypeAlias {
                    name: assignment.name.clone(),
                    target: TypeRef::Builtin(BuiltinType::Any),
                }]
            }
            AsnType::Any => {
                vec![Declaration::TypeAlias {
                    name: assignment.name.clone(),
                    target: TypeRef::Builtin(BuiltinType::Any),
                }]
            }
            AsnType::Real => {
                vec![Declaration::TypeAlias {
                    name: assignment.name.clone(),
                    target: TypeRef::Builtin(BuiltinType::Real),
                }]
            }
            AsnType::RelativeOid => {
                vec![Declaration::TypeAlias {
                    name: assignment.name.clone(),
                    target: TypeRef::Builtin(BuiltinType::ObjectIdentifier),
                }]
            }
        }
    }

    fn build_sequence_field_with_parent(&self, f: &SequenceField, parent_name: &str) -> Field {
        let resolved = self.resolve_type(&f.ty);
        let ty = if is_inline_type(resolved) {
            let gen_name = Self::inline_type_name(parent_name, &f.name);
            if f.optional || f.default.is_some() {
                TypeRef::Optional(Box::new(TypeRef::Named(gen_name)))
            } else {
                TypeRef::Named(gen_name)
            }
        } else if f.optional || f.default.is_some() {
            TypeRef::Optional(Box::new(self.build_type(&f.ty)))
        } else {
            self.build_type(&f.ty)
        };

        Field {
            name: f.name.clone(),
            ty,
            optional: f.optional || f.default.is_some(),
            default: f.default.as_ref().map(asn_value_to_literal),
            doc_comment: None,
            ber: Some(self.ber_info_for_field(&f.ty, parent_name, &f.name)),
            encode_stmts: Vec::new(),
            decode_stmts: Vec::new(),
            order: 0,
        }
    }

    fn ber_info_for_type(&self, ty: &AsnType) -> BerFieldInfo {
        let resolved = self.resolve_type(ty);
        let original_name = if let AsnType::ReferencedType { name, .. } = ty {
            Some(name.clone())
        } else {
            None
        };
        match resolved {
            AsnType::Boolean => BerFieldInfo {
                encoding: "boolean".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 1,
                constructed: false,
                string_encoding: String::new(),
                referenced_type: None,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 1,
                list_element_ber: None,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
            AsnType::Integer { .. } => BerFieldInfo {
                encoding: "integer".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 2,
                constructed: false,
                string_encoding: String::new(),
                referenced_type: None,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 2,
                list_element_ber: None,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
            AsnType::Real => BerFieldInfo {
                encoding: "real".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 9,
                constructed: false,
                string_encoding: String::new(),
                referenced_type: None,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 9,
                list_element_ber: None,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
            AsnType::Enumerated { .. } => BerFieldInfo {
                encoding: "enumerated".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 10,
                constructed: false,
                string_encoding: String::new(),
                referenced_type: None,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 10,
                list_element_ber: None,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
            AsnType::BitString { .. } => BerFieldInfo {
                encoding: "bit_string".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 3,
                constructed: false,
                string_encoding: String::new(),
                referenced_type: None,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 3,
                list_element_ber: None,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
            AsnType::OctetString => BerFieldInfo {
                encoding: "bytes".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 4,
                constructed: false,
                string_encoding: String::new(),
                referenced_type: None,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 4,
                list_element_ber: None,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
            AsnType::Null => BerFieldInfo {
                encoding: "null".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 5,
                constructed: false,
                string_encoding: String::new(),
                referenced_type: None,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 5,
                list_element_ber: None,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
            AsnType::ObjectIdentifier => BerFieldInfo {
                encoding: "oid".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 6,
                constructed: false,
                string_encoding: String::new(),
                referenced_type: None,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 6,
                list_element_ber: None,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
            AsnType::Sequence { .. } => BerFieldInfo {
                encoding: "constructed".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 16,
                constructed: true,
                string_encoding: String::new(),
                referenced_type: original_name,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 16,
                list_element_ber: None,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
            AsnType::Set { .. } => BerFieldInfo {
                encoding: "constructed".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 17,
                constructed: true,
                string_encoding: String::new(),
                referenced_type: original_name,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 17,
                list_element_ber: None,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
            AsnType::SequenceOf { element_type } => BerFieldInfo {
                encoding: "list".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 16,
                constructed: true,
                string_encoding: String::new(),
                referenced_type: original_name,
                list_element_ber: Some(Box::new(self.ber_info_for_type(element_type))),
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 16,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
            AsnType::SetOf { element_type } => BerFieldInfo {
                encoding: "list".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 17,
                constructed: true,
                string_encoding: String::new(),
                referenced_type: original_name,
                list_element_ber: Some(Box::new(self.ber_info_for_type(element_type))),
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 17,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
            AsnType::Choice { .. } => BerFieldInfo {
                encoding: "choice".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 0,
                constructed: true,
                string_encoding: String::new(),
                referenced_type: original_name,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 0,
                list_element_ber: None,
                defined_by: None,
                choice_alternative_tags: make_alt_tags(resolved),
            },
            AsnType::RestrictedString(charset) => {
                let (tag_number, string_encoding) = match charset {
                    CharsetType::UTF8 => (12, "utf-8".to_string()),
                    CharsetType::Numeric => (18, "utf-8".to_string()),
                    CharsetType::Printable => (19, "utf-8".to_string()),
                    CharsetType::Teletex => (20, "utf-8".to_string()),
                    CharsetType::Videotex => (21, "utf-8".to_string()),
                    CharsetType::IA5 => (22, "utf-8".to_string()),
                    CharsetType::Graphic => (25, "utf-8".to_string()),
                    CharsetType::Visible => (26, "utf-8".to_string()),
                    CharsetType::General => (27, "utf-8".to_string()),
                    CharsetType::Universal => (28, "utf-8".to_string()),
                    CharsetType::BMP => (30, "ucs-2".to_string()),
                };
                BerFieldInfo {
                    encoding: "string".to_string(),
                    tag_class: "UNIVERSAL".to_string(),
                    tag_number,
                    constructed: false,
                    string_encoding,
                    referenced_type: None,
                    tagging_mode: "inherent".to_string(),
                    inherent_tag_class: "UNIVERSAL".to_string(),
                    inherent_tag_number: tag_number,
                    list_element_ber: None,
                    defined_by: None,
                    choice_alternative_tags: Vec::new(),
                }
            }
            AsnType::UnrestrictedString => BerFieldInfo {
                encoding: "bytes".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 4,
                constructed: false,
                string_encoding: String::new(),
                referenced_type: None,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 4,
                list_element_ber: None,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
            AsnType::GeneralizedTime => BerFieldInfo {
                encoding: "time".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 24,
                constructed: false,
                string_encoding: String::new(),
                referenced_type: None,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 24,
                list_element_ber: None,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
            AsnType::UTCTime => BerFieldInfo {
                encoding: "time".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 23,
                constructed: false,
                string_encoding: String::new(),
                referenced_type: None,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 23,
                list_element_ber: None,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
            AsnType::ReferencedType { name, .. } => {
                BerFieldInfo {
                    encoding: "constructed".to_string(),
                    tag_class: "UNIVERSAL".to_string(),
                    tag_number: 16,
                    constructed: true,
                    string_encoding: String::new(),
                    referenced_type: Some(name.clone()),
                    tagging_mode: "inherent".to_string(),
                    inherent_tag_class: "UNIVERSAL".to_string(),
                    inherent_tag_number: 16,
                    list_element_ber: None,
                    defined_by: None,
                    choice_alternative_tags: Vec::new(),
                }
            }
            AsnType::ConstrainedType { base, .. } => self.ber_info_for_type(base),
            AsnType::Tagged { class, number, implicit, inner } => {
                let tag_class = tag_class_str(class).to_string();
                let inner_info = self.ber_info_for_type(inner);
                let tagging_mode = if *implicit { "implicit".to_string() } else { "explicit".to_string() };
                BerFieldInfo {
                    tag_class,
                    tag_number: *number,
                    constructed: true,
                    tagging_mode,
                    inherent_tag_class: inner_info.tag_class.clone(),
                    inherent_tag_number: inner_info.tag_number,
                    ..inner_info
                }
            }
            AsnType::RelativeOid => BerFieldInfo {
                encoding: "oid".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 13,
                constructed: false,
                string_encoding: String::new(),
                referenced_type: None,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 13,
                list_element_ber: None,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
            AsnType::OpenType { defined_by } => BerFieldInfo {
                encoding: "any".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 0,
                constructed: false,
                string_encoding: String::new(),
                referenced_type: None,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 0,
                list_element_ber: None,
                defined_by: defined_by.clone(),
                choice_alternative_tags: Vec::new(),
            },
            AsnType::Any => BerFieldInfo {
                encoding: "any".to_string(),
                tag_class: "UNIVERSAL".to_string(),
                tag_number: 0,
                constructed: false,
                string_encoding: String::new(),
                referenced_type: None,
                tagging_mode: "inherent".to_string(),
                inherent_tag_class: "UNIVERSAL".to_string(),
                inherent_tag_number: 0,
                list_element_ber: None,
                defined_by: None,
                choice_alternative_tags: Vec::new(),
            },
        }
    }

    fn build_type(&self, ty: &AsnType) -> TypeRef {
        match ty {
            AsnType::Boolean => TypeRef::Builtin(BuiltinType::Boolean),
            AsnType::Integer { .. } => TypeRef::Builtin(BuiltinType::Integer),
            AsnType::OctetString => TypeRef::Builtin(BuiltinType::OctetString),
            AsnType::Null => TypeRef::Builtin(BuiltinType::Null),
            AsnType::RestrictedString(charset) => {
                TypeRef::Builtin(BuiltinType::String(string_encoding_to_ir(charset)))
            }
            AsnType::UnrestrictedString => {
                TypeRef::Builtin(BuiltinType::OctetString)
            }
            AsnType::SequenceOf { element_type } => {
                TypeRef::List(Box::new(self.build_type(element_type)))
            }
            AsnType::SetOf { element_type } => {
                TypeRef::List(Box::new(self.build_type(element_type)))
            }
            AsnType::BitString { .. } => TypeRef::Builtin(BuiltinType::BitString),
            AsnType::ObjectIdentifier => TypeRef::Builtin(BuiltinType::ObjectIdentifier),
            AsnType::Enumerated { .. } => TypeRef::Builtin(BuiltinType::Integer),
            AsnType::ReferencedType { name, .. } => TypeRef::Named(name.clone()),
            AsnType::GeneralizedTime => TypeRef::Builtin(BuiltinType::GeneralizedTime),
            AsnType::UTCTime => TypeRef::Builtin(BuiltinType::UTCTime),
            AsnType::Real => TypeRef::Builtin(BuiltinType::Real),
            AsnType::ConstrainedType { base, .. } => self.build_type(base),
            AsnType::Tagged { inner, .. } => self.build_type(inner),
            AsnType::OpenType { .. } => TypeRef::Builtin(BuiltinType::Any),
            AsnType::Any => TypeRef::Builtin(BuiltinType::Any),
            _ => TypeRef::Builtin(BuiltinType::Any),
        }
    }
}
