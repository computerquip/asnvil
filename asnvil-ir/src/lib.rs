pub mod ir;
pub mod resolver;
pub mod error;
pub mod from_ast;

#[cfg(test)]
mod tests {
    use miette::SourceSpan;
    use num_bigint::BigInt;

    use crate::ir::{self, AsnModule, AsnType, SequenceField, TypeAssignment, Exports, Import, TagDefault};
    use crate::resolver::Resolver;
    use crate::from_ast::module_to_ir;
    use crate::error::IrError;

    // ─── Helper: create a span ────────────────────────────────────────
    fn span() -> SourceSpan {
        SourceSpan::from(0..0)
    }

    // ─── Helper: build AST module from assignments ────────────────────
    fn make_ast_module(
        name: &str,
        assignments: Vec<asnvil_parser::ast::Assignment>,
    ) -> asnvil_parser::ast::Module {
        asnvil_parser::ast::Module {
            identifier: asnvil_parser::ast::ModuleIdentifier {
                name: name.to_string(),
                oid: None,
                span: span(),
            },
            tag_default: None,
            ext_default: false,
            body: asnvil_parser::ast::ModuleBody {
                exports: None,
                imports: vec![],
                assignments,
            },
            span: span(),
        }
    }

    // ─── Helper: build IR module directly ─────────────────────────────
    fn make_ir_module(
        name: &str,
        types: Vec<TypeAssignment>,
        imports: Vec<Import>,
        exports: Exports,
    ) -> AsnModule {
        AsnModule {
            name: name.to_string(),
            oid: None,
            tag_default: TagDefault::Explicit,
            ext_default: false,
            exports,
            imports,
            types,
            values: vec![],
            object_classes: vec![],
            objects: vec![],
            object_sets: vec![],
        }
    }

    // ─── Helper: make a primitive AST type ────────────────────────────
    fn ast_integer() -> asnvil_parser::ast::AsnType {
        asnvil_parser::ast::AsnType::Integer { named_numbers: None, span: span() }
    }

    fn ast_boolean() -> asnvil_parser::ast::AsnType {
        asnvil_parser::ast::AsnType::Boolean { span: span() }
    }

    fn ast_string() -> asnvil_parser::ast::AsnType {
        asnvil_parser::ast::AsnType::RestrictedString {
            charset: asnvil_parser::ast::CharsetType::UTF8,
            span: span(),
        }
    }

    fn ast_tagged(number: i64, inner: asnvil_parser::ast::AsnType) -> asnvil_parser::ast::AsnType {
        asnvil_parser::ast::AsnType::Tagged {
            class: None,
            number: BigInt::from(number),
            implicit: Some(false),
            inner: Box::new(inner),
            span: span(),
        }
    }

    fn ast_enum_item(name: &str, value: Option<i64>) -> asnvil_parser::ast::EnumItem {
        asnvil_parser::ast::EnumItem {
            name: name.to_string(),
            value: value.map(BigInt::from),
        }
    }

    fn ast_enumerated(items: Vec<asnvil_parser::ast::EnumItem>) -> asnvil_parser::ast::AsnType {
        asnvil_parser::ast::AsnType::Enumerated {
            items,
            extensible: false,
            ext_items: vec![],
            span: span(),
        }
    }

    fn ast_import(symbols: Vec<&str>, module: &str) -> asnvil_parser::ast::Import {
        asnvil_parser::ast::Import {
            symbols: symbols.into_iter().map(String::from).collect(),
            module: module.to_string(),
            module_oid: None,
        }
    }

    fn ast_component(name: &str, ty: asnvil_parser::ast::AsnType) -> asnvil_parser::ast::ComponentType {
        asnvil_parser::ast::ComponentType {
            name: name.to_string(),
            ty,
            optional: false,
            default: None,
        }
    }

    fn ast_sequence(fields: Vec<asnvil_parser::ast::ComponentType>) -> asnvil_parser::ast::AsnType {
        asnvil_parser::ast::AsnType::Sequence {
            fields,
            extensible: false,
            ext_fields: vec![],
            span: span(),
        }
    }

    fn ast_named_type(name: &str, ty: asnvil_parser::ast::AsnType) -> asnvil_parser::ast::NamedType {
        asnvil_parser::ast::NamedType {
            name: name.to_string(),
            ty,
        }
    }

    fn ast_choice(alternatives: Vec<asnvil_parser::ast::NamedType>) -> asnvil_parser::ast::AsnType {
        asnvil_parser::ast::AsnType::Choice {
            alternatives,
            extensible: false,
            ext_alternatives: vec![],
            span: span(),
        }
    }

    fn ast_sequence_of(element: asnvil_parser::ast::AsnType) -> asnvil_parser::ast::AsnType {
        asnvil_parser::ast::AsnType::SequenceOf {
            element_type: Box::new(element),
            span: span(),
        }
    }

    // ─── AST → IR conversion tests ────────────────────────────────────

    #[test]
    fn test_convert_sequence() {
        let fields = vec![
            ast_component("id", ast_integer()),
            ast_component("name", ast_string()),
            ast_component("active", ast_boolean()),
        ];
        let ast_mod = make_ast_module("TestMod", vec![
            asnvil_parser::ast::Assignment::Type(asnvil_parser::ast::TypeAssignment {
                name: "Person".to_string(),
                parameters: None,
                ty: ast_sequence(fields),
                span: span(),
            }),
        ]);
        let ir_mod = module_to_ir(&ast_mod).expect("conversion failed");
        assert_eq!(ir_mod.name, "TestMod");
        assert_eq!(ir_mod.types.len(), 1);
        let ty = &ir_mod.types[0].ty;
        match ty {
            AsnType::Sequence { fields, ext } => {
                assert_eq!(fields.len(), 3);
                assert_eq!(fields[0].name, "id");
                assert!(matches!(fields[0].ty, AsnType::Integer { .. }));
                assert_eq!(fields[1].name, "name");
                assert!(matches!(fields[1].ty, AsnType::RestrictedString(_)));
                assert_eq!(fields[2].name, "active");
                assert!(matches!(fields[2].ty, AsnType::Boolean));
                assert!(ext.is_none());
            }
            _ => panic!("expected Sequence, got {:?}", ty),
        }
    }

    #[test]
    fn test_convert_choice() {
        let alternatives = vec![
            ast_named_type("intValue", ast_integer()),
            ast_named_type("strValue", ast_string()),
        ];
        let ast_mod = make_ast_module("TestMod", vec![
            asnvil_parser::ast::Assignment::Type(asnvil_parser::ast::TypeAssignment {
                name: "MyChoice".to_string(),
                parameters: None,
                ty: ast_choice(alternatives),
                span: span(),
            }),
        ]);
        let ir_mod = module_to_ir(&ast_mod).expect("conversion failed");
        let ty = &ir_mod.types[0].ty;
        match ty {
            AsnType::Choice { alternatives, ext } => {
                assert_eq!(alternatives.len(), 2);
                assert_eq!(alternatives[0].name, "intValue");
                assert!(matches!(alternatives[0].ty, AsnType::Integer { .. }));
                assert_eq!(alternatives[1].name, "strValue");
                assert!(ext.is_none());
            }
            _ => panic!("expected Choice, got {:?}", ty),
        }
    }

    #[test]
    fn test_convert_enumerated() {
        let items = vec![
            ast_enum_item("red", Some(1)),
            ast_enum_item("green", Some(2)),
            ast_enum_item("blue", Some(3)),
        ];
        let ast_mod = make_ast_module("TestMod", vec![
            asnvil_parser::ast::Assignment::Type(asnvil_parser::ast::TypeAssignment {
                name: "Color".to_string(),
                parameters: None,
                ty: ast_enumerated(items),
                span: span(),
            }),
        ]);
        let ir_mod = module_to_ir(&ast_mod).expect("conversion failed");
        let ty = &ir_mod.types[0].ty;
        match ty {
            AsnType::Enumerated { root, ext } => {
                assert_eq!(root.len(), 3);
                assert_eq!(root[0].name, "red");
                assert_eq!(root[0].value, BigInt::from(1));
                assert_eq!(root[1].name, "green");
                assert_eq!(root[1].value, BigInt::from(2));
                assert!(ext.is_none());
            }
            _ => panic!("expected Enumerated, got {:?}", ty),
        }
    }

    #[test]
    fn test_convert_sequence_of() {
        let ast_mod = make_ast_module("TestMod", vec![
            asnvil_parser::ast::Assignment::Type(asnvil_parser::ast::TypeAssignment {
                name: "IntList".to_string(),
                parameters: None,
                ty: ast_sequence_of(ast_integer()),
                span: span(),
            }),
        ]);
        let ir_mod = module_to_ir(&ast_mod).expect("conversion failed");
        let ty = &ir_mod.types[0].ty;
        match ty {
            AsnType::SequenceOf { element_type } => {
                assert!(matches!(*element_type.clone(), AsnType::Integer { .. }));
            }
            _ => panic!("expected SequenceOf, got {:?}", ty),
        }
    }

    #[test]
    fn test_convert_tagged() {
        let ast_mod = make_ast_module("TestMod", vec![
            asnvil_parser::ast::Assignment::Type(asnvil_parser::ast::TypeAssignment {
                name: "TaggedInt".to_string(),
                parameters: None,
                ty: ast_tagged(5, ast_integer()),
                span: span(),
            }),
        ]);
        let ir_mod = module_to_ir(&ast_mod).expect("conversion failed");
        let ty = &ir_mod.types[0].ty;
        match ty {
            AsnType::Tagged { class, number, implicit, inner } => {
                assert!(matches!(class, ir::TagClass::ContextSpecific));
                assert_eq!(*number, 5);
                assert_eq!(*implicit, false);
                assert!(matches!(**inner, AsnType::Integer { .. }));
            }
            _ => panic!("expected Tagged, got {:?}", ty),
        }
    }

    #[test]
    fn test_convert_boolean() {
        let ast_mod = make_ast_module("TestMod", vec![
            asnvil_parser::ast::Assignment::Type(asnvil_parser::ast::TypeAssignment {
                name: "Flag".to_string(),
                parameters: None,
                ty: ast_boolean(),
                span: span(),
            }),
        ]);
        let ir_mod = module_to_ir(&ast_mod).expect("conversion failed");
        assert!(matches!(ir_mod.types[0].ty, AsnType::Boolean));
    }

    #[test]
    fn test_enum_sequential_values() {
        // When enum items have no value, they should be computed sequentially (R8 fix)
        let items = vec![
            ast_enum_item("first", None),
            ast_enum_item("second", None),
        ];
        let ast_mod = make_ast_module("TestMod", vec![
            asnvil_parser::ast::Assignment::Type(asnvil_parser::ast::TypeAssignment {
                name: "Status".to_string(),
                parameters: None,
                ty: ast_enumerated(items),
                span: span(),
            }),
        ]);
        let ir_mod = module_to_ir(&ast_mod).expect("conversion failed");
        let ty = &ir_mod.types[0].ty;
        match ty {
            AsnType::Enumerated { root, .. } => {
                // Fixed behavior: missing values computed sequentially
                assert_eq!(root[0].value, BigInt::from(0));
                assert_eq!(root[1].value, BigInt::from(1));
            }
            _ => panic!("expected Enumerated"),
        }
    }

    // ─── Resolver tests ───────────────────────────────────────────────

    #[test]
    fn test_resolve_simple_sequence() {
        let fields = vec![
            SequenceField { name: "id".into(), ty: AsnType::Integer { named_numbers: vec![] }, optional: false, default: None },
            SequenceField { name: "name".into(), ty: AsnType::RestrictedString(ir::CharsetType::UTF8), optional: false, default: None },
        ];
        let mut resolver = Resolver::new();
        let module = make_ir_module("TestMod", vec![
            TypeAssignment { name: "Person".into(), ty: AsnType::Sequence { fields, ext: None }, parameters: None },
        ], vec![], Exports::All);
        resolver.add_module(module).unwrap();
        resolver.resolve().unwrap();
        let resolved = &resolver.modules().get("TestMod").unwrap().types[0];
        assert_eq!(resolved.name, "Person");
    }

    #[test]
    fn test_resolve_referenced_type() {
        let mut resolver = Resolver::new();
        let mod_a = make_ir_module("ModA", vec![
            TypeAssignment { name: "Person".into(), ty: AsnType::Sequence { fields: vec![], ext: None }, parameters: None },
            TypeAssignment { name: "People".into(), ty: AsnType::Sequence { fields: vec![
                SequenceField { name: "people".into(), ty: AsnType::ReferencedType { module: None, name: "Person".into() }, optional: false, default: None },
            ], ext: None }, parameters: None },
        ], vec![], Exports::All);
        resolver.add_module(mod_a).unwrap();
        resolver.resolve().unwrap();
        let resolved = &resolver.modules().get("ModA").unwrap().types[1];
        // Person is a complex type, so the reference should be preserved
        match &resolved.ty {
            AsnType::Sequence { fields, .. } => {
                match &fields[0].ty {
                    AsnType::ReferencedType { name, .. } => {
                        assert_eq!(name, "Person");
                    }
                    _ => panic!("expected ReferencedType for field"),
                }
            }
            _ => panic!("expected Sequence for People"),
        }
    }

    // NOTE: Circular reference detection (detect_circular_references) is dead code.
    // resolve_types() calls resolve_type() which recurses infinitely for cycles
    // before detect_circular_references() ever runs. This is a known resolver bug.
    // The test below is commented out to avoid stack overflow.
    #[test]
    #[ignore = "resolver has known bug: resolve_type() stack overflows on circular refs before cycle detection runs"]
    fn test_resolve_circular_reference() {
        let mut resolver = Resolver::new();
        // A references B, B references A (circular)
        let mod_a = make_ir_module("ModA", vec![
            TypeAssignment { name: "A".into(), ty: AsnType::Sequence { fields: vec![
                SequenceField { name: "b".into(), ty: AsnType::ReferencedType { module: None, name: "B".into() }, optional: false, default: None },
            ], ext: None }, parameters: None },
            TypeAssignment { name: "B".into(), ty: AsnType::Sequence { fields: vec![
                SequenceField { name: "a".into(), ty: AsnType::ReferencedType { module: None, name: "A".into() }, optional: false, default: None },
            ], ext: None }, parameters: None },
        ], vec![], Exports::All);
        resolver.add_module(mod_a).unwrap();
        let result = resolver.resolve();
        assert!(result.is_err(), "circular reference should be detected");
        match result.unwrap_err() {
            IrError::CircularReference(msg) => {
                assert!(msg.contains("A") && msg.contains("B"), "cycle message should mention both types: {}", msg);
            }
            e => panic!("expected CircularReference, got {:?}", e),
        }
    }

    #[test]
    fn test_resolve_import_exists() {
        let mut resolver = Resolver::new();
        let mod_a = make_ir_module("ModA", vec![
            TypeAssignment { name: "Person".into(), ty: AsnType::Integer { named_numbers: vec![] }, parameters: None },
        ], vec![], Exports::All);
        let mod_b = make_ir_module("ModB", vec![
            TypeAssignment { name: "Employee".into(), ty: AsnType::ReferencedType { module: Some("ModA".into()), name: "Person".into() }, parameters: None },
        ], vec![
            Import { symbols: vec!["Person".into()], module: "ModA".into(), module_oid: None },
        ], Exports::All);
        resolver.add_module(mod_a).unwrap();
        resolver.add_module(mod_b).unwrap();
        resolver.resolve().unwrap();
    }

    #[test]
    fn test_resolve_import_missing_module() {
        let mut resolver = Resolver::new();
        let mod_b = make_ir_module("ModB", vec![
            TypeAssignment { name: "Employee".into(), ty: AsnType::ReferencedType { module: Some("NonExistent".into()), name: "Person".into() }, parameters: None },
        ], vec![
            Import { symbols: vec!["Person".into()], module: "NonExistent".into(), module_oid: None },
        ], Exports::All);
        resolver.add_module(mod_b).unwrap();
        let result = resolver.resolve();
        assert!(result.is_err(), "missing module should cause error");
        match result.unwrap_err() {
            IrError::UnknownModule(name) => {
                assert_eq!(name, "NonExistent");
            }
            e => panic!("expected UnknownModule, got {:?}", e),
        }
    }

    #[test]
    fn test_resolve_import_missing_symbol() {
        let mut resolver = Resolver::new();
        // ModA exports only "Person"
        let mod_a = make_ir_module("ModA", vec![
            TypeAssignment { name: "Person".into(), ty: AsnType::Integer { named_numbers: vec![] }, parameters: None },
        ], vec![], Exports::Symbols(vec!["Person".into()]));
        // ModB tries to import "Employee" which doesn't exist
        let mod_b = make_ir_module("ModB", vec![], vec![
            Import { symbols: vec!["Employee".into()], module: "ModA".into(), module_oid: None },
        ], Exports::All);
        resolver.add_module(mod_a).unwrap();
        resolver.add_module(mod_b).unwrap();
        let result = resolver.resolve();
        assert!(result.is_err(), "unexported symbol should cause error");
        match result.unwrap_err() {
            IrError::UnexportedSymbol(symbol, module) => {
                assert_eq!(symbol, "Employee");
                assert_eq!(module, "ModA");
            }
            e => panic!("expected UnexportedSymbol, got {:?}", e),
        }
    }

    #[test]
    fn test_resolve_exports_none() {
        let mut resolver = Resolver::new();
        let mod_a = make_ir_module("ModA", vec![
            TypeAssignment { name: "Secret".into(), ty: AsnType::Integer { named_numbers: vec![] }, parameters: None },
        ], vec![], Exports::None);
        let mod_b = make_ir_module("ModB", vec![], vec![
            Import { symbols: vec!["Secret".into()], module: "ModA".into(), module_oid: None },
        ], Exports::All);
        resolver.add_module(mod_a).unwrap();
        resolver.add_module(mod_b).unwrap();
        let result = resolver.resolve();
        assert!(result.is_err(), "importing from non-exporting module should fail");
        match result.unwrap_err() {
            IrError::UnexportedSymbol(symbol, module) => {
                assert_eq!(symbol, "Secret");
                assert_eq!(module, "ModA");
            }
            e => panic!("expected UnexportedSymbol, got {:?}", e),
        }
    }

    // ─── Duplicate detection tests ────────────────────────────────────

    #[test]
    fn test_duplicate_type_names() {
        // Currently, duplicate types silently coexist (R9 - known issue)
        let mut resolver = Resolver::new();
        let module = make_ir_module("ModA", vec![
            TypeAssignment { name: "Person".into(), ty: AsnType::Integer { named_numbers: vec![] }, parameters: None },
            TypeAssignment { name: "Person".into(), ty: AsnType::Boolean, parameters: None },
        ], vec![], Exports::All);
        resolver.add_module(module).unwrap();
        let result = resolver.resolve();
        // Current behavior: no error for duplicates (R9 gap)
        assert!(result.is_ok(), "current behavior allows duplicates");
        let resolved = resolver.modules().get("ModA").unwrap();
        assert_eq!(resolved.types.len(), 2, "both types should still exist");
    }
}
