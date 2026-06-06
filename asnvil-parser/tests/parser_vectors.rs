//! Integration tests for the ASN.1 parser using co-located test vectors.
//! 
//! These tests verify that the public `parse` API correctly generates the expected AST
//! for various ASN.1 schemas stored in the workspace `tests/vectors/` directory.

use asnvil_parser::ast::{AsnType, AsnValue, Assignment, ExportSymbols, Module};
use asnvil_parser::grammar::Grammar;
use asnvil_parser::parse;
use std::path::PathBuf;

/// Helper to parse a source string and return the resulting Module.
fn parse_source(source: &str) -> Module {
    let mut grammar = Grammar::new();
    parse(source, std::path::Path::new("test.asn1"), &mut grammar)
        .expect("parse should succeed");
    grammar.result.expect("grammar should produce a result")
}

/// Helper to load an ASN.1 vector file from the workspace tests/vectors/ directory.
fn load_vector(category: &str, name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(format!("../tests/vectors/{}/{}", category, name));
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read vector file: {:?}", path))
}

/// Helper to parse a source and extract a specific type assignment by name.
fn parse_type(source: &str, type_name: &str) -> AsnType {
    let ast = parse_source(source);
    let assignment = ast
        .body
        .assignments
        .iter()
        .find(|a| matches!(a, Assignment::Type(ta) if ta.name == type_name))
        .unwrap_or_else(|| panic!("Type assignment '{}' not found", type_name));

    match assignment {
        Assignment::Type(ta) => ta.ty.clone(),
        _ => unreachable!(),
    }
}

/// Helper to parse a source and extract a specific value assignment by name.
fn parse_value(source: &str, value_name: &str) -> AsnValue {
    let ast = parse_source(source);
    let assignment = ast
        .body
        .assignments
        .iter()
        .find(|a| matches!(a, Assignment::Value(va) if va.name == value_name))
        .unwrap_or_else(|| panic!("Value assignment '{}' not found", value_name));

    match assignment {
        Assignment::Value(va) => va.value.clone(),
        _ => unreachable!(),
    }
}

#[test]
fn test_parser_value_sequences() {
    let source = load_vector("value_sequences", "schema.asn1");
    
    let _status_ty = parse_type(&source, "Status");
    let _next_ty = parse_type(&source, "NextType");

    let thing_ty = parse_type(&source, "Thing");
    match thing_ty {
        AsnType::Sequence { fields, .. } => {
            let active_field = fields.iter().find(|f| f.name == "active")
                .expect("should have active field");
            assert!(active_field.default.is_some(), "active field should have a DEFAULT value");
        }
        other => panic!("expected Sequence, got {:?}", other),
    }

    let shape_ty = parse_type(&source, "Shape");
    match shape_ty {
        AsnType::Sequence { fields, .. } => {
            let color_field = fields.iter().find(|f| f.name == "color")
                .expect("should have color field");
            let default = color_field.default.as_ref()
                .expect("color field should have a DEFAULT value");
            
            match default {
                AsnValue::Sequence(items) => {
                    assert!(!items.is_empty(), "value sequence should contain parsed items");
                }
                other => panic!("expected AsnValue::Sequence, got {:?}", other),
            }
        }
        other => panic!("expected Sequence, got {:?}", other),
    }
}

#[test]
fn test_parser_imports() {
    let source = load_vector("imports", "schema.asn1");
    let ast = parse_source(&source);
    
    assert!(!ast.body.imports.is_empty(), "should have IMPORTS");
    let import = &ast.body.imports[0];
    assert_eq!(import.symbols.len(), 3);
    assert_eq!(import.symbols, &["INTEGER", "SEQUENCE", "OCTET"]);
    assert_eq!(import.module, "SpecModule");

    let exports = ast.body.exports.as_ref().expect("should have EXPORTS");
    match &exports.symbols {
        ExportSymbols::Symbols(symbols) => {
            assert_eq!(symbols.len(), 2);
            assert_eq!(symbols, &["MyType", "AnotherType"]);
        }
        other => panic!("expected Symbols variant, got {:?}", other),
    }

    let _my_type = parse_type(&source, "MyType");
    let _another_type = parse_type(&source, "AnotherType");
}

#[test]
fn test_parser_hex_strings() {
    let source = load_vector("hex_strings", "schema.asn1");
    
    let valid_value = parse_value(&source, "MyOctetValid");
    match valid_value {
        AsnValue::HexString(bytes) => assert_eq!(bytes, &[0xDE, 0xAD, 0xBE, 0xEF]),
        other => panic!("expected HexString, got {:?}", other),
    }

    let odd_value = parse_value(&source, "MyOctetOdd");
    match odd_value {
        AsnValue::HexString(bytes) => assert_eq!(bytes, &[0x0A, 0xBC]),
        other => panic!("expected HexString, got {:?}", other),
    }
}

#[test]
fn test_parser_invalid_hex_string() {
    let source = load_vector("invalid_hex_string", "schema.asn1");
    let mut grammar = Grammar::new();
    let result = parse(
        &source,
        std::path::Path::new("test.asn1"),
        &mut grammar,
    );
    assert!(result.is_err(), "invalid hex string should produce a parse error");
}

#[test]
fn test_parser_reference_imports_exports() {
    let source = load_vector("reference_imports_exports", "schema.asn1");
    let ast = parse_source(&source);
    
    assert!(!ast.body.imports.is_empty(), "should have IMPORTS");
    let import = &ast.body.imports[0];
    assert_eq!(import.symbols.len(), 2);
    assert_eq!(import.symbols, &["Person", "X509Certificate"]);
    assert_eq!(import.module, "OtherModule");

    let exports = ast.body.exports.as_ref().expect("should have EXPORTS");
    match &exports.symbols {
        ExportSymbols::Symbols(symbols) => {
            assert_eq!(symbols.len(), 2);
            assert_eq!(symbols, &["Person", "Certificate"]);
        }
        other => panic!("expected Symbols variant, got {:?}", other),
    }
}

#[test]
fn test_parser_value_items() {
    let source = load_vector("value_items", "schema.asn1");
    let value = parse_value(&source, "MyValue");
    match value {
        AsnValue::Sequence(items) => {
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].name, "foo");
            match &items[0].value {
                AsnValue::Integer(n) => assert_eq!(n.to_string(), "1"),
                other => panic!("expected Integer, got {:?}", other),
            }
            assert_eq!(items[1].name, "bar");
            match &items[1].value {
                AsnValue::Boolean(b) => assert!(*b),
                other => panic!("expected Boolean, got {:?}", other),
            }
        }
        other => panic!("expected Sequence, got {:?}", other),
    }
}

#[test]
fn test_parser_parameterized_types() {
    let source = load_vector("parameterized_types", "schema.asn1");
    let ty = parse_type(&source, "MyRef");
    match ty {
        AsnType::Referenced { name, parameters, .. } => {
            assert_eq!(name, "MyParamType");
            let params = parameters.as_ref().expect("should have parameters");
            assert_eq!(params.len(), 1);
        }
        other => panic!("expected Referenced, got {:?}", other),
    }
}

#[test]
fn test_parser_named_numbers() {
    let source = load_vector("named_numbers", "schema.asn1");
    let next_ty = parse_type(&source, "NextType");
    match next_ty {
        AsnType::Boolean { .. } => {}
        other => panic!("expected Boolean, got {:?}", other),
    }
}
