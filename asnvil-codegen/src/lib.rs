pub mod code_ast;
pub mod builder;
pub mod renderer;
pub mod python;

#[cfg(test)]
mod tests {
    use asnvil_ir::ir::{
        self, AsnModule, AsnType, SequenceField, TypeAssignment, EnumItem,
        Exports, TagDefault, ChoiceAlternative, Constraints,
    };
    use num_bigint::BigInt;

    use crate::builder::CodeAstBuilder;
    use crate::code_ast::{CodeAstNode, Declaration, TypeRef, BuiltinType, ValueLiteral};
    use crate::python::PythonRenderer;
    use crate::renderer::LanguageRenderer;

    fn make_ir_module(
        name: &str,
        types: Vec<TypeAssignment>,
    ) -> AsnModule {
        AsnModule {
            name: name.to_string(),
            oid: None,
            tag_default: TagDefault::Explicit,
            ext_default: false,
            exports: Exports::All,
            imports: vec![],
            types,
            values: vec![],
            object_classes: vec![],
            objects: vec![],
            object_sets: vec![],
        }
    }

    fn integer_field(name: &str) -> SequenceField {
        SequenceField {
            name: name.to_string(),
            ty: AsnType::Integer { named_numbers: vec![], constraints: ir::Constraints::default() },
            optional: false,
            default: None,
        }
    }

    fn string_field(name: &str) -> SequenceField {
        SequenceField {
            name: name.to_string(),
            ty: AsnType::RestrictedString(ir::CharsetType::UTF8, ir::Constraints::default()),
            optional: false,
            default: None,
        }
    }

    fn bool_field(name: &str) -> SequenceField {
        SequenceField {
            name: name.to_string(),
            ty: AsnType::Boolean,
            optional: false,
            default: None,
        }
    }

    fn default_field(name: &str, default: ValueLiteral) -> SequenceField {
        SequenceField {
            name: name.to_string(),
            ty: AsnType::Integer { named_numbers: vec![], constraints: ir::Constraints::default() },
            optional: true,
            default: Some(ir::AsnValue::Integer(BigInt::from(match default {
                ValueLiteral::Int(n) => n,
                _ => 0,
            }))),
        }
    }

    fn optional_field(name: &str) -> SequenceField {
        SequenceField {
            name: name.to_string(),
            ty: AsnType::Integer { named_numbers: vec![], constraints: ir::Constraints::default() },
            optional: true,
            default: None,
        }
    }

    fn referenced_field(module: Option<&str>, name: &str) -> SequenceField {
        SequenceField {
            name: name.to_string(),
            ty: AsnType::ReferencedType { module: module.map(String::from), name: name.to_string() },
            optional: false,
            default: None,
        }
    }

    // ─── Builder tests ────────────────────────────────────────────────

    #[test]
    fn test_build_sequence() {
        let module = make_ir_module("TestMod", vec![
            TypeAssignment {
                name: "Person".into(),
                ty: AsnType::Sequence {
                    fields: vec![
                        integer_field("id"),
                        string_field("name"),
                        bool_field("active"),
                    ],
                    ext: None,
                },
                parameters: None,
            },
        ]);
        let builder = CodeAstBuilder::new();
        let ast = builder.build_module(&module);

        match ast {
            CodeAstNode::Module { declarations, .. } => {
                assert_eq!(declarations.len(), 1);
                match &declarations[0] {
                    Declaration::Struct { name, fields, .. } => {
                        assert_eq!(name, "Person");
                        assert_eq!(fields.len(), 3);
                        assert_eq!(fields[0].name, "id");
                        assert!(matches!(fields[0].ty, TypeRef::Builtin(BuiltinType::Integer)));
                        assert_eq!(fields[1].name, "name");
                        assert!(matches!(fields[1].ty, TypeRef::Builtin(BuiltinType::String(_))));
                        assert_eq!(fields[2].name, "active");
                        assert!(matches!(fields[2].ty, TypeRef::Builtin(BuiltinType::Boolean)));
                    }
                    _ => panic!("expected Struct declaration"),
                }
            }
            _ => panic!("expected Module node"),
        }
    }

    #[test]
    fn test_build_choice() {
        let module = make_ir_module("TestMod", vec![
            TypeAssignment {
                name: "Status".into(),
                ty: AsnType::Choice {
                    alternatives: vec![
                        ChoiceAlternative { name: "active".into(), ty: AsnType::Boolean },
                        ChoiceAlternative { name: "pending".into(), ty: AsnType::Integer { named_numbers: vec![], constraints: Constraints::default() } },
                    ],
                    ext: None,
                },
                parameters: None,
            },
        ]);
        let builder = CodeAstBuilder::new();
        let ast = builder.build_module(&module);

        match ast {
            CodeAstNode::Module { declarations, .. } => {
                assert_eq!(declarations.len(), 1);
                match &declarations[0] {
                    Declaration::Choice { name, alternatives, .. } => {
                        assert_eq!(name, "Status");
                        assert_eq!(alternatives.len(), 2);
                        assert_eq!(alternatives[0].name, "active");
                        assert_eq!(alternatives[1].name, "pending");
                    }
                    _ => panic!("expected Choice declaration"),
                }
            }
            _ => panic!("expected Module node"),
        }
    }

    #[test]
    fn test_build_enumerated() {
        let module = make_ir_module("TestMod", vec![
            TypeAssignment {
                name: "Color".into(),
                ty: AsnType::Enumerated {
                    root: vec![
                        EnumItem { name: "red".into(), value: BigInt::from(1) },
                        EnumItem { name: "green".into(), value: BigInt::from(2) },
                        EnumItem { name: "blue".into(), value: BigInt::from(3) },
                    ],
                    ext: None,
                },
                parameters: None,
            },
        ]);
        let builder = CodeAstBuilder::new();
        let ast = builder.build_module(&module);

        match ast {
            CodeAstNode::Module { declarations, .. } => {
                assert_eq!(declarations.len(), 1);
                match &declarations[0] {
                    Declaration::Enum { name, variants, .. } => {
                        assert_eq!(name, "Color");
                        assert_eq!(variants.len(), 3);
                        assert_eq!(variants[0].name, "red");
                        assert_eq!(variants[0].value, Some(0));
                        assert_eq!(variants[1].name, "green");
                        assert_eq!(variants[1].value, Some(1));
                    }
                    _ => panic!("expected Enum declaration"),
                }
            }
            _ => panic!("expected Module node"),
        }
    }

    #[test]
    fn test_build_sequence_of() {
        let module = make_ir_module("TestMod", vec![
            TypeAssignment {
                name: "People".into(),
                ty: AsnType::SequenceOf {
                    element_type: Box::new(AsnType::ReferencedType { module: None, name: "Person".into() }),
                },
                parameters: None,
            },
        ]);
        let builder = CodeAstBuilder::new();
        let ast = builder.build_module(&module);

        match ast {
            CodeAstNode::Module { declarations, .. } => {
                assert_eq!(declarations.len(), 1);
                match &declarations[0] {
                    Declaration::ListType { name, element_type, .. } => {
                        assert_eq!(name, "People");
                        assert!(matches!(element_type, TypeRef::Named(n) if n == "Person"));
                    }
                    _ => panic!("expected ListType declaration"),
                }
            }
            _ => panic!("expected Module node"),
        }
    }

    #[test]
    fn test_build_with_defaults() {
        let module = make_ir_module("TestMod", vec![
            TypeAssignment {
                name: "Config".into(),
                ty: AsnType::Sequence {
                    fields: vec![
                        integer_field("id"),
                        default_field("timeout", ValueLiteral::Int(30)),
                        optional_field("retry"),
                        default_field("max_conn", ValueLiteral::Int(10)),
                    ],
                    ext: None,
                },
                parameters: None,
            },
        ]);
        let builder = CodeAstBuilder::new();
        let ast = builder.build_module(&module);

        match ast {
            CodeAstNode::Module { declarations, .. } => {
                assert_eq!(declarations.len(), 1);
                match &declarations[0] {
                    Declaration::Struct { fields, .. } => {
                        // Non-default fields should come first
                        assert_eq!(fields[0].name, "id");
                        assert!(fields[0].default.is_none());
                        assert!(!fields[0].optional);
                        // Default/optional fields should come last
                        assert!(fields.iter().skip(1).all(|f| f.default.is_some() || f.optional));
                    }
                    _ => panic!("expected Struct declaration"),
                }
            }
            _ => panic!("expected Module node"),
        }
    }

    #[test]
    fn test_build_with_referenced_type() {
        let module = make_ir_module("TestMod", vec![
            TypeAssignment {
                name: "Person".into(),
                ty: AsnType::Sequence {
                    fields: vec![string_field("name"), integer_field("age")],
                    ext: None,
                },
                parameters: None,
            },
            TypeAssignment {
                name: "Employee".into(),
                ty: AsnType::Sequence {
                    fields: vec![
                        string_field("id"),
                        referenced_field(None, "Person"),
                    ],
                    ext: None,
                },
                parameters: None,
            },
        ]);
        let builder = CodeAstBuilder::new();
        let ast = builder.build_module(&module);

        match ast {
            CodeAstNode::Module { declarations, .. } => {
                assert!(declarations.len() >= 2);
                let person = declarations.iter().find(|d| matches!(d, Declaration::Struct { name, .. } if name == "Person"));
                let employee = declarations.iter().find(|d| matches!(d, Declaration::Struct { name, .. } if name == "Employee"));
                assert!(person.is_some());
                assert!(employee.is_some());
            }
            _ => panic!("expected Module node"),
        }
    }

    // ─── Renderer tests ───────────────────────────────────────────────

    #[test]
    fn test_render_struct_has_encode_decode() {
        let module = make_ir_module("TestMod", vec![
            TypeAssignment {
                name: "Person".into(),
                ty: AsnType::Sequence {
                    fields: vec![string_field("name"), integer_field("age")],
                    ext: None,
                },
                parameters: None,
            },
        ]);
        let builder = CodeAstBuilder::new();
        let ast = builder.build_module(&module);
        let renderer = PythonRenderer::new();
        let output = renderer.render_module(&ast).expect("render failed");

        assert!(output.contains("class Person"), "should define Person class");
        assert!(output.contains("encode_ber"), "should have encode_ber method");
        assert!(output.contains("encode_der"), "should have encode_der method");
        assert!(output.contains("decode_der"), "should have decode_der method");
    }

    #[test]
    fn test_render_choice_has_tag_checking() {
        let module = make_ir_module("TestMod", vec![
            TypeAssignment {
                name: "Status".into(),
                ty: AsnType::Choice {
                    alternatives: vec![
                        ChoiceAlternative { name: "active".into(), ty: AsnType::Boolean },
                        ChoiceAlternative { name: "pending".into(), ty: AsnType::Integer { named_numbers: vec![], constraints: Constraints::default() } },
                    ],
                    ext: None,
                },
                parameters: None,
            },
        ]);
        let builder = CodeAstBuilder::new();
        let ast = builder.build_module(&module);
        let renderer = PythonRenderer::new();
        let output = renderer.render_module(&ast).expect("render failed");

        assert!(output.contains("class Status"), "should define Status class");
        assert!(output.contains("decode_der"), "CHOICE should have decode_der");
    }

    #[test]
    fn test_render_list_type() {
        let module = make_ir_module("TestMod", vec![
            TypeAssignment {
                name: "People".into(),
                ty: AsnType::SequenceOf {
                    element_type: Box::new(AsnType::ReferencedType { module: None, name: "Person".into() }),
                },
                parameters: None,
            },
        ]);
        let builder = CodeAstBuilder::new();
        let ast = builder.build_module(&module);
        let renderer = PythonRenderer::new();
        let output = renderer.render_module(&ast).expect("render failed");

        assert!(output.contains("class People"), "should define People class");
        assert!(output.contains("encode_ber"), "list type should have encode_ber");
        assert!(output.contains("decode_der"), "list type should have decode_der");
    }

    #[test]
    fn test_render_valid_python() {
        let module = make_ir_module("TestMod", vec![
            TypeAssignment {
                name: "Person".into(),
                ty: AsnType::Sequence {
                    fields: vec![
                        string_field("name"),
                        integer_field("age"),
                        bool_field("active"),
                    ],
                    ext: None,
                },
                parameters: None,
            },
            TypeAssignment {
                name: "Status".into(),
                ty: AsnType::Choice {
                    alternatives: vec![
                        ChoiceAlternative { name: "active".into(), ty: AsnType::Boolean },
                        ChoiceAlternative { name: "code".into(), ty: AsnType::Integer { named_numbers: vec![], constraints: Constraints::default() } },
                    ],
                    ext: None,
                },
                parameters: None,
            },
        ]);
        let builder = CodeAstBuilder::new();
        let ast = builder.build_module(&module);
        let renderer = PythonRenderer::new();
        let output = renderer.render_module(&ast).expect("render failed");

        let result = std::process::Command::new("python3")
            .arg("-c")
            .arg(format!("compile({}, '<string>', 'exec')", serde_json::to_string(&output).unwrap()))
            .output()
            .expect("failed to run python3");

        assert!(
            result.status.success(),
            "generated Python should be syntactically valid.\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr),
        );
    }

    #[test]
    fn test_render_enumerated() {
        let module = make_ir_module("TestMod", vec![
            TypeAssignment {
                name: "Color".into(),
                ty: AsnType::Enumerated {
                    root: vec![
                        EnumItem { name: "red".into(), value: BigInt::from(1) },
                        EnumItem { name: "green".into(), value: BigInt::from(2) },
                    ],
                    ext: None,
                },
                parameters: None,
            },
        ]);
        let builder = CodeAstBuilder::new();
        let ast = builder.build_module(&module);
        let renderer = PythonRenderer::new();
        let output = renderer.render_module(&ast).expect("render failed");

        assert!(output.contains("class Color"), "should define Color class");
        assert!(output.contains("IntEnum"), "enumerated should use IntEnum");
        assert!(output.contains("red"), "should have red variant");
        assert!(output.contains("green"), "should have green variant");
    }

    #[test]
    fn test_render_type_alias() {
        let module = make_ir_module("TestMod", vec![
            TypeAssignment {
                name: "MyInt".into(),
                ty: AsnType::Integer { named_numbers: vec![], constraints: Constraints::default() },
                parameters: None,
            },
        ]);
        let builder = CodeAstBuilder::new();
        let ast = builder.build_module(&module);
        let renderer = PythonRenderer::new();
        let output = renderer.render_module(&ast).expect("render failed");

        assert!(output.contains("MyInt"), "should define MyInt type alias");
    }
}
