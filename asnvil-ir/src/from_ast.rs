use asnvil_parser::ast;
use num_bigint::BigInt;
use thiserror::Error;

use crate::ir;

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Failed to convert ASN.1 type: {0}")]
    TypeConversion(String),
    #[error("Failed to convert ASN.1 value: {0}")]
    ValueConversion(String),
}

pub fn module_to_ir(ast_mod: &ast::Module) -> Result<ir::AsnModule, ConversionError> {
    let mut types = Vec::new();
    let mut values = Vec::new();

    for assignment in &ast_mod.body.assignments {
        match assignment {
            ast::Assignment::Type(ta) => {
                types.push(ir::TypeAssignment {
                    name: ta.name.clone(),
                    ty: asn_type_to_ir(&ta.ty)?,
                    parameters: ta.parameters.as_ref().map(|p| {
                        p.parameters.iter().map(param_to_ir).collect()
                    }),
                });
            }
            ast::Assignment::Value(va) => {
                values.push(ir::ValueAssignment {
                    name: va.name.clone(),
                    ty: asn_type_to_ir(&va.ty)?,
                    value: asn_value_to_ir(&va.value)?,
                });
            }
            _ => {}
        }
    }

    let exports = match &ast_mod.body.exports {
        Some(e) => match &e.symbols {
            ast::ExportSymbols::All => ir::Exports::All,
            ast::ExportSymbols::Symbols(syms) => ir::Exports::Symbols(syms.clone()),
        },
        None => ir::Exports::None,
    };

    let tag_default = match &ast_mod.tag_default {
        Some(td) => tag_default_to_ir(td),
        None => ir::TagDefault::Explicit,
    };

    let imports: Vec<ir::Import> = ast_mod.body.imports.iter().map(|imp| ir::Import {
        symbols: imp.symbols.clone(),
        module: imp.module.clone(),
        module_oid: imp.module_oid.as_ref().map(oid_to_ir),
    }).collect();

    Ok(ir::AsnModule {
        name: ast_mod.identifier.name.clone(),
        oid: ast_mod.identifier.oid.as_ref().map(oid_to_ir),
        tag_default,
        ext_default: ast_mod.ext_default,
        exports,
        imports,
        types,
        values,
        object_classes: vec![],
        objects: vec![],
        object_sets: vec![],
    })
}

fn tag_default_to_ir(td: &ast::TagDefault) -> ir::TagDefault {
    match td {
        ast::TagDefault::Explicit => ir::TagDefault::Explicit,
        ast::TagDefault::Implicit => ir::TagDefault::Implicit,
        ast::TagDefault::Automatic => ir::TagDefault::Automatic,
    }
}

fn oid_to_ir(ast_oid: &ast::ObjectIdentifier) -> ir::ObjectIdentifier {
    ir::ObjectIdentifier {
        components: ast_oid.components.iter().map(oid_component_to_ir).collect(),
    }
}

fn oid_component_to_ir(c: &ast::OidComponent) -> ir::OidComponent {
    match c {
        ast::OidComponent::Name(n) => ir::OidComponent::Name(n.clone()),
        ast::OidComponent::Number(n) => ir::OidComponent::Number(n.clone()),
    }
}

fn param_to_ir(p: &ast::Parameter) -> ir::Parameter {
    ir::Parameter {
        name: p.name.clone(),
        governor: p.governor.as_ref().map(|g| match g {
            ast::ParameterGovernor::Type(t) => {
                ir::ParameterGovernor::Type(asn_type_to_ir(t).unwrap_or(ir::AsnType::Any))
            }
            ast::ParameterGovernor::ObjectClass(s) => ir::ParameterGovernor::ObjectClass(s.clone()),
            ast::ParameterGovernor::ValueSet(vs) => {
                let vals: Vec<ir::AsnValue> = vs.iter().filter_map(|v| asn_value_to_ir(v).ok()).collect();
                ir::ParameterGovernor::ValueSet(vals)
            }
        }),
    }
}

fn asn_type_to_ir(ast_ty: &ast::AsnType) -> Result<ir::AsnType, ConversionError> {
    let ir_ty = match ast_ty {
        ast::AsnType::Boolean { .. } => ir::AsnType::Boolean,
        ast::AsnType::Integer { named_numbers, .. } => {
            let nums = named_numbers.as_ref().map(|nn| {
                nn.iter().map(|n| (n.name.clone(), n.value.clone())).collect()
            }).unwrap_or_default();
            ir::AsnType::Integer { named_numbers: nums }
        }
        ast::AsnType::Real { .. } => ir::AsnType::Real,
        ast::AsnType::Enumerated { items, extensible, ext_items, .. } => {
            let root = items.iter().map(enum_item_to_ir).collect();
            let ext = if *extensible && !ext_items.is_empty() {
                Some(ext_items.iter().map(enum_item_to_ir).collect())
            } else {
                None
            };
            ir::AsnType::Enumerated { root, ext }
        }
        ast::AsnType::BitString { named_bits, .. } => {
            let bits = named_bits.as_ref().map(|nb| {
                nb.iter().map(|n| (n.name.clone(), n.value.clone())).collect()
            }).unwrap_or_default();
            ir::AsnType::BitString { named_bits: bits }
        }
        ast::AsnType::OctetString { .. } => ir::AsnType::OctetString,
        ast::AsnType::Null { .. } => ir::AsnType::Null,
        ast::AsnType::Sequence { fields, extensible, ext_fields, .. } => {
            let root = fields.iter().map(component_to_ir).collect::<Result<Vec<_>, _>>()?;
            let ext = if *extensible && !ext_fields.is_empty() {
                Some(ext_fields.iter().map(component_to_ir).collect::<Result<Vec<_>, _>>()?)
            } else {
                None
            };
            ir::AsnType::Sequence { fields: root, ext }
        }
        ast::AsnType::Set { fields, extensible, ext_fields, .. } => {
            let root = fields.iter().map(component_to_ir).collect::<Result<Vec<_>, _>>()?;
            let ext = if *extensible && !ext_fields.is_empty() {
                Some(ext_fields.iter().map(component_to_ir).collect::<Result<Vec<_>, _>>()?)
            } else {
                None
            };
            ir::AsnType::Set { fields: root, ext }
        }
        ast::AsnType::Choice { alternatives, extensible, ext_alternatives, .. } => {
            let root = alternatives.iter().map(named_type_to_ir).collect::<Result<Vec<_>, _>>()?;
            let ext = if *extensible && !ext_alternatives.is_empty() {
                Some(ext_alternatives.iter().map(named_type_to_ir).collect::<Result<Vec<_>, _>>()?)
            } else {
                None
            };
            ir::AsnType::Choice { alternatives: root, ext }
        }
        ast::AsnType::SequenceOf { element_type, .. } => {
            ir::AsnType::SequenceOf { element_type: Box::new(asn_type_to_ir(element_type)?) }
        }
        ast::AsnType::SetOf { element_type, .. } => {
            ir::AsnType::SetOf { element_type: Box::new(asn_type_to_ir(element_type)?) }
        }
        ast::AsnType::Tagged { class, number, implicit, inner, .. } => {
            let tc = class.as_ref().map(tag_class_to_ir).unwrap_or(ir::TagClass::ContextSpecific);
            let num = number.try_into().unwrap_or(0);
            let imp = implicit.unwrap_or(false);
            ir::AsnType::Tagged {
                class: tc,
                number: num,
                implicit: imp,
                inner: Box::new(asn_type_to_ir(inner)?),
            }
        }
        ast::AsnType::ObjectIdentifier { .. } => ir::AsnType::ObjectIdentifier,
        ast::AsnType::RelativeOid { .. } => ir::AsnType::RelativeOid,
        ast::AsnType::RestrictedString { charset, .. } => {
            ir::AsnType::RestrictedString(charset_to_ir(charset))
        }
        ast::AsnType::UnrestrictedString { .. } => ir::AsnType::UnrestrictedString,
        ast::AsnType::GeneralizedTime { .. } => ir::AsnType::GeneralizedTime,
        ast::AsnType::UTCTime { .. } => ir::AsnType::UTCTime,
        ast::AsnType::Any { .. } => ir::AsnType::Any,
        ast::AsnType::OpenType { defined_by, .. } => ir::AsnType::OpenType { defined_by: defined_by.clone() },
        ast::AsnType::Constrained { base, constraint, .. } => {
            ir::AsnType::ConstrainedType {
                base: Box::new(asn_type_to_ir(base)?),
                constraints: constraint_to_ir(constraint)?,
            }
        }
        ast::AsnType::Referenced { name, .. } => {
            ir::AsnType::ReferencedType {
                module: None,
                name: name.clone(),
            }
        }
    };
    Ok(ir_ty)
}

fn enum_item_to_ir(item: &ast::EnumItem) -> ir::EnumItem {
    ir::EnumItem {
        name: item.name.clone(),
        value: item.value.clone().unwrap_or(BigInt::from(0)),
    }
}

fn component_to_ir(comp: &ast::ComponentType) -> Result<ir::SequenceField, ConversionError> {
    Ok(ir::SequenceField {
        name: comp.name.clone(),
        ty: asn_type_to_ir(&comp.ty)?,
        optional: comp.optional,
        default: comp.default.as_ref().map(asn_value_to_ir).transpose()?,
    })
}

fn named_type_to_ir(nt: &ast::NamedType) -> Result<ir::ChoiceAlternative, ConversionError> {
    Ok(ir::ChoiceAlternative {
        name: nt.name.clone(),
        ty: asn_type_to_ir(&nt.ty)?,
    })
}

fn tag_class_to_ir(tc: &ast::TagClass) -> ir::TagClass {
    match tc {
        ast::TagClass::Universal => ir::TagClass::Universal,
        ast::TagClass::Application => ir::TagClass::Application,
        ast::TagClass::Private => ir::TagClass::Private,
    }
}

fn charset_to_ir(cs: &ast::CharsetType) -> ir::CharsetType {
    match cs {
        ast::CharsetType::UTF8 => ir::CharsetType::UTF8,
        ast::CharsetType::Numeric => ir::CharsetType::Numeric,
        ast::CharsetType::Printable => ir::CharsetType::Printable,
        ast::CharsetType::Teletex => ir::CharsetType::Teletex,
        ast::CharsetType::Videotex => ir::CharsetType::Videotex,
        ast::CharsetType::IA5 => ir::CharsetType::IA5,
        ast::CharsetType::Graphic => ir::CharsetType::Graphic,
        ast::CharsetType::Visible => ir::CharsetType::Visible,
        ast::CharsetType::General => ir::CharsetType::General,
        ast::CharsetType::Universal => ir::CharsetType::Universal,
        ast::CharsetType::BMP => ir::CharsetType::BMP,
    }
}

fn constraint_to_ir(_c: &ast::Constraint) -> Result<ir::Constraints, ConversionError> {
    // Constraints not yet fully parsed from grammar, return empty
    Ok(ir::Constraints { subtypes: vec![] })
}

fn asn_value_to_ir(val: &ast::AsnValue) -> Result<ir::AsnValue, ConversionError> {
    let ir_val = match val {
        ast::AsnValue::Boolean(b) => ir::AsnValue::Boolean(*b),
        ast::AsnValue::Integer(n) => ir::AsnValue::Integer(n.clone()),
        ast::AsnValue::BitString(bytes) => ir::AsnValue::BitString {
            unused_bits: 0,
            bytes: bytes.clone(),
        },
        ast::AsnValue::HexString(bytes) => ir::AsnValue::HexString(bytes.clone()),
        ast::AsnValue::CharString(s) => ir::AsnValue::CharString(s.clone()),
        ast::AsnValue::Null => ir::AsnValue::Null,
        ast::AsnValue::Sequence(items) => {
            let named: Vec<ir::NamedValue> = items.iter().map(|nv| {
                asn_value_to_ir(&nv.value).map(|v| ir::NamedValue {
                    name: nv.name.clone(),
                    value: v,
                })
            }).collect::<Result<Vec<_>, ConversionError>>()?;
            ir::AsnValue::Sequence(named)
        }
        ast::AsnValue::Choice { name, value } => ir::AsnValue::Choice {
            name: name.clone(),
            value: Box::new(asn_value_to_ir(value)?),
        },
        ast::AsnValue::Enumerated(s) => ir::AsnValue::Enumerated(s.clone()),
        ast::AsnValue::ObjectIdentifier(oid) => ir::AsnValue::ObjectIdentifier(oid_to_ir(oid)),
        ast::AsnValue::Referenced(s) => ir::AsnValue::Referenced(s.clone()),
    };
    Ok(ir_val)
}
