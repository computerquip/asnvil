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
    use crate::ast::{AsnType, AsnValue, Assignment, Module};
    use crate::grammar::Grammar;
    use crate::parse;

    fn parse_source(source: &str) -> Module {
        let mut grammar = Grammar::new();
        parse(source, std::path::Path::new("test.asn1"), &mut grammar)
            .expect("parse should succeed");
        grammar.result.expect("grammar should produce a result")
    }

    #[test]
    fn test_value_sequence_in_default() {
        // Uses a SEQUENCE OF with a single-value DEFAULT to avoid
        // value_stack pollution from the broken {ValueItems} collection.
        // The bug: value_item pushes to value_stack but LBraceValueItemsRBrace
        // creates Vec::new() instead of collecting them, leaving orphans on the stack.
        let ast = parse_source(
            r#"
            TestModule DEFINITIONS AUTOMATIC TAGS ::= BEGIN
                Status ::= BOOLEAN
                Thing ::= SEQUENCE {
                    active Status DEFAULT TRUE
                }
            END;
            "#,
        );

        // First verify basic DEFAULT parsing works
        let thing_assignment = ast
            .body
            .assignments
            .iter()
            .find(|a| {
                if let Assignment::Type(ta) = a {
                    ta.name == "Thing"
                } else {
                    false
                }
            })
            .expect("should have Thing assignment");

        let seq = match thing_assignment {
            Assignment::Type(ta) => match &ta.ty {
                AsnType::Sequence { fields, .. } => fields,
                other => panic!("expected Sequence, got {:?}", other),
            },
            _ => unreachable!(),
        };

        let active_field = seq
            .iter()
            .find(|f| f.name == "active")
            .expect("should have active field");

        assert!(
            active_field.default.is_some(),
            "active field should have a DEFAULT value"
        );
    }

    #[test]
    fn test_value_sequence_collects_items() {
        // Tests that { ValueItems } actually collects values from the value_stack.
        // The bug: LBraceValueItemsRBrace creates Vec::new() instead of
        // popping values pushed by value_item callbacks.
        //
        // Using a standalone type that comes AFTER the one with the DEFAULT
        // to detect whether the value_stack pollution breaks downstream parsing.
        let asn1_source = r#"
TestModule DEFINITIONS AUTOMATIC TAGS ::= BEGIN
Colors ::= ENUMERATED { red(0), green(1), blue(2) }
Shape ::= SEQUENCE {
    color Colors DEFAULT { red }
}
Status ::= BOOLEAN
END;
"#;
        let ast = parse_source(asn1_source);

        // Verify that downstream types were parsed correctly
        // (they won't be if value_stack pollution corrupts parsing)
        let _status_assignment = ast
            .body
            .assignments
            .iter()
            .find(|a| {
                if let Assignment::Type(ta) = a {
                    ta.name == "Status"
                } else {
                    false
                }
            })
            .expect("Status assignment should exist — value_stack pollution from {} broke downstream parsing");

        let shape_assignment = ast
            .body
            .assignments
            .iter()
            .find(|a| {
                if let Assignment::Type(ta) = a {
                    ta.name == "Shape"
                } else {
                    false
                }
            })
            .expect("should have Shape assignment");

        let seq = match shape_assignment {
            Assignment::Type(ta) => match &ta.ty {
                AsnType::Sequence { fields, .. } => fields,
                other => panic!("expected Sequence, got {:?}", other),
            },
            _ => unreachable!(),
        };

        let color_field = seq
            .iter()
            .find(|f| f.name == "color")
            .expect("should have color field");

        let default = color_field
            .default
            .as_ref()
            .expect("color field should have a DEFAULT value");

        match default {
            AsnValue::Sequence(items) => {
                assert!(
                    !items.is_empty(),
                    "value sequence should contain parsed items, but got empty Vec — \
                     this is the R1 bug: LBraceValueItemsRBrace ignores value_stack"
                );
            }
            other => panic!("expected AsnValue::Sequence for DEFAULT value, got {:?}", other),
        }
    }

    #[test]
    fn test_import_keyword_symbols() {
        // R2: import_symbol must handle ALL IdentifierOrKeyword variants,
        // not just Identifier. Keywords like INTEGER, SEQUENCE, OCTET, etc.
        // were falling through to `format!("{:?}", ...)` producing debug strings.
        // Note: The grammar only accepts keywords (not arbitrary References)
        // in import positions, so we test with actual ASN.1 keywords.
        let source = r#"
TestModule DEFINITIONS AUTOMATIC TAGS ::= BEGIN
    IMPORTS INTEGER FROM OtherModule;
    MyType ::= BOOLEAN
END;
"#;
        let ast = parse_source(source);
        assert!(!ast.body.imports.is_empty(), "should have IMPORTS");
        let import = &ast.body.imports[0];
        assert_eq!(import.symbols.len(), 1);
        assert_eq!(import.symbols[0], "INTEGER");
        assert_eq!(import.module, "OtherModule");
    }

    #[test]
    fn test_import_multiple_keyword_symbols() {
        // Verify multiple keyword-named symbols are imported correctly.
        let source = r#"
TestModule DEFINITIONS AUTOMATIC TAGS ::= BEGIN
    IMPORTS INTEGER, SEQUENCE, OCTET FROM SpecModule;
    MyType ::= BOOLEAN
END;
"#;
        let ast = parse_source(source);
        assert!(!ast.body.imports.is_empty(), "should have IMPORTS");
        let import = &ast.body.imports[0];
        assert_eq!(import.symbols.len(), 3);
        assert_eq!(import.symbols, &["INTEGER", "SEQUENCE", "OCTET"]);
        assert_eq!(import.module, "SpecModule");
    }
}
