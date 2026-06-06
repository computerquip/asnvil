#![allow(clippy::enum_variant_names)]
#![allow(clippy::large_enum_variant)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::too_many_arguments)]

mod grammar_trait {
    include!(concat!(env!("OUT_DIR"), "/grammar_trait.rs"));
}

mod asn1_parser {
    include!(concat!(env!("OUT_DIR"), "/parser.rs"));
}

pub mod grammar;
pub use asn1_parser::parse;
pub use grammar_trait::GrammarTrait;

pub mod ast;
pub mod error;

#[cfg(test)]
mod tests {
    use crate::ast::{AsnType, AsnValue, Assignment, ExportSymbols, Module};
    use crate::grammar::Grammar;
    use crate::parse;
    use std::path::PathBuf;

    fn parse_source(source: &str) -> Module {
        let mut grammar = Grammar::new();
        parse(source, std::path::Path::new("test.asn1"), &mut grammar)
            .expect("parse should succeed");
        grammar.result.expect("grammar should produce a result")
    }

    /// Helper to load a parser vector file from the tests/vectors/parser directory.
    fn load_vector(name: &str) -> String {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../tests/vectors/parser");
        path.push(name);
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
    fn test_value_sequence_in_default() {
        let source = r#"
TestModule DEFINITIONS AUTOMATIC TAGS ::= BEGIN
    Status ::= BOOLEAN
    Thing ::= SEQUENCE {
        active Status DEFAULT TRUE
    }
END;
"#;
        let ty = parse_type(source, "Thing");
        match ty {
            AsnType::Sequence { fields, .. } => {
                let active_field = fields.iter().find(|f| f.name == "active")
                    .expect("should have active field");
                assert!(active_field.default.is_some(), "active field should have a DEFAULT value");
            }
            other => panic!("expected Sequence, got {:?}", other),
        }
    }

    #[test]
    fn test_value_sequence_collects_items() {
        // Tests that { ValueItems } actually collects values from the value_stack.
        // The bug: LBraceValueItemsRBrace creates Vec::new() instead of
        // popping values pushed by value_item callbacks.
        // Using a standalone type that comes AFTER the one with the DEFAULT
        // to detect whether the value_stack pollution breaks downstream parsing.
        let source = r#"
TestModule DEFINITIONS AUTOMATIC TAGS ::= BEGIN
    Colors ::= ENUMERATED { red(0), green(1), blue(2) }
    Shape ::= SEQUENCE {
        color Colors DEFAULT { red }
    }
    Status ::= BOOLEAN
END;
"#;
        // Verify downstream types parse correctly (would fail if stack polluted)
        let _status_ty = parse_type(source, "Status");
        
        let shape_ty = parse_type(source, "Shape");
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
    fn test_parser_vector_imports() {
        // Tests module imports, exports, and type assignments from a file.
        let source = load_vector("01_imports.asn1");
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
    fn test_parser_vector_hex_strings() {
        let source = load_vector("05_hex_strings.asn1");
        
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
    fn test_parser_vector_invalid_hex_string() {
        // R5: Invalid hex digits should produce a parse error, not silently become 0.
        let source = load_vector("06_invalid_hex_string.asn1");
        let mut grammar = crate::grammar::Grammar::new();
        let result = parse(
            &source,
            std::path::Path::new("test.asn1"),
            &mut grammar,
        );
        assert!(result.is_err(), "invalid hex string should produce a parse error");
    }

    #[test]
    fn test_import_reference_type_name() {
        // R41: Import symbols must accept Reference (uppercase type names like Person).
        // Before: only Identifier (lowercase) and keywords were accepted.
        let source = r#"
TestModule DEFINITIONS AUTOMATIC TAGS ::= BEGIN
    IMPORTS Person, X509Certificate FROM OtherModule;
    MyType ::= BOOLEAN
END;
"#;
        let ast = parse_source(source);
        assert!(!ast.body.imports.is_empty(), "should have IMPORTS");
        let import = &ast.body.imports[0];
        assert_eq!(import.symbols.len(), 2);
        assert_eq!(import.symbols, &["Person", "X509Certificate"]);
        assert_eq!(import.module, "OtherModule");
    }

    #[test]
    fn test_export_reference_type_name() {
        // R41: Export symbols must accept Reference (uppercase type names).
        let source = r#"
TestModule DEFINITIONS AUTOMATIC TAGS ::= BEGIN
    EXPORTS Person, Certificate;
    Person ::= SEQUENCE { name IA5String }
    Certificate ::= OCTET STRING
END;
"#;
        let ast = parse_source(source);
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
    fn test_parser_vector_value_items() {
        // Tests lowercase identifier with colon in sequence value.
        let source = load_vector("02_value_items.asn1");
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
    fn test_parser_vector_parameterized_types() {
        // Tests referenced types with parameters.
        let source = load_vector("04_parameterized_types.asn1");
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
    fn test_parser_vector_named_numbers() {
        // Verifies that named_number properly pops the Reference from str_stack,
        // preventing stack pollution that would corrupt downstream parsing.
        let source = load_vector("03_named_numbers.asn1");
        // If stack pollution occurs, parsing NextType will fail or be corrupted
        let next_ty = parse_type(&source, "NextType");
        match next_ty {
            AsnType::Boolean { .. } => {}
            other => panic!("expected Boolean, got {:?}", other),
        }
    }
}
