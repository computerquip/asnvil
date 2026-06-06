use crate::ast;
use crate::ast::Spanned;
use crate::grammar_trait::*;
use miette::SourceSpan;
use num_bigint::BigInt;
use parol_runtime::Token;
use parol_runtime::Result;
use std::collections::VecDeque;
use std::marker::PhantomData;

/// Create a SourceSpan from a parol Token's location
fn span(token: &Token<'_>) -> SourceSpan {
    SourceSpan::from(token.location.start()..token.location.end())
}

/// Minimal stack helper with error reporting on underflow
struct Stack<T> {
    inner: VecDeque<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Self { inner: VecDeque::new() }
    }
    fn push(&mut self, item: T) {
        self.inner.push_back(item);
    }
    fn try_pop(&mut self) -> Result<T> {
        self.inner.pop_back().ok_or_else(|| parol_runtime::ParolError::UserError(anyhow::anyhow!("Parser stack underflow")))
    }
    fn pop(&mut self) -> Option<T> {
        self.inner.pop_back()
    }
    #[allow(dead_code)]
    fn pop_reversed(&mut self, count: usize) -> Result<Vec<T>> {
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            items.push(self.try_pop()?);
        }
        items.reverse();
        Ok(items)
    }
    fn drain(&mut self) -> Vec<T> {
        self.inner.drain(..).collect()
    }
}

pub struct Grammar<'t> {
    pub parse_tree: Option<parol_runtime::ParseTree>,
    pub result: Option<ast::Module>,
    _marker: PhantomData<&'t ()>,

    // Stacks for bottom-up AST construction
    str_stack: Stack<String>,
    bigint_stack: Stack<BigInt>,
    tag_default_stack: Stack<ast::TagDefault>,
    module_id_stack: Stack<ast::ModuleIdentifier>,
    module_body_stack: Stack<ast::ModuleBody>,
    export_stack: Stack<ast::ExportSymbols>,
    export_is_all: bool,
    import_stack: Stack<ast::Import>,
    assignment_stack: Stack<ast::Assignment>,
    type_stack: Stack<ast::AsnType>,
    value_stack: Stack<ast::AsnValue>,
    named_value_stack: Stack<ast::NamedValue>,
    component_stack: Stack<ast::ComponentType>,
    named_type_stack: Stack<ast::NamedType>,
    enum_item_stack: Stack<ast::EnumItem>,
    named_number_stack: Stack<ast::NamedNumber>,
    named_bit_stack: Stack<ast::NamedBit>,
    oid_stack: Stack<ast::OidComponent>,
    module_oid_stack: Stack<ast::ObjectIdentifier>,
    actual_param_stack: Stack<ast::ActualParameter>,
    constraint_stack: Stack<ast::Constraint>,
    constraint_spec_stack: Stack<ast::ConstraintSpec>,
}

impl<'t> Default for Grammar<'t> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'t> Grammar<'t> {
    pub fn new() -> Self {
        Self {
            parse_tree: None,
            result: None,
            _marker: PhantomData,
            str_stack: Stack::new(),
            bigint_stack: Stack::new(),
            tag_default_stack: Stack::new(),
            module_id_stack: Stack::new(),
            module_body_stack: Stack::new(),
            export_stack: Stack::new(),
            export_is_all: false,
            import_stack: Stack::new(),
            assignment_stack: Stack::new(),
            type_stack: Stack::new(),
            value_stack: Stack::new(),
            named_value_stack: Stack::new(),
            component_stack: Stack::new(),
            named_type_stack: Stack::new(),
            enum_item_stack: Stack::new(),
            named_number_stack: Stack::new(),
            named_bit_stack: Stack::new(),
            oid_stack: Stack::new(),
            module_oid_stack: Stack::new(),
            actual_param_stack: Stack::new(),
            constraint_stack: Stack::new(),
            constraint_spec_stack: Stack::new(),
        }
    }
}

impl<'t> GrammarTrait<'t> for Grammar<'t> {
    fn identifier(&mut self, arg: &Identifier<'t>) -> Result<()> {
        self.str_stack.push(arg.identifier.text().to_string());
        Ok(())
    }

    fn reference(&mut self, arg: &Reference<'t>) -> Result<()> {
        self.str_stack.push(arg.reference.text().to_string());
        Ok(())
    }

    fn number_literal(&mut self, arg: &NumberLiteral<'t>) -> Result<()> {
        let text = arg.number_literal.text();
        let value: BigInt = text.parse().map_err(|e| anyhow::anyhow!("Invalid number: {text}: {e}"))?;
        self.bigint_stack.push(value);
        Ok(())
    }

    fn binary_string(&mut self, arg: &BinaryString<'t>) -> Result<()> {
        let text = arg.binary_string.text();
        let bits: Vec<u8> = text[1..text.len() - 1]
            .chars()
            .map(|c| if c == '1' { 1 } else { 0 })
            .collect();
        self.value_stack.push(ast::AsnValue::BitString(bits));
        Ok(())
    }

    fn hex_string(&mut self, arg: &HexString<'t>) -> Result<()> {
        let text = arg.hex_string.text();
        let hex = &text[1..text.len() - 2];
        let hex_data = if hex.len() % 2 == 0 {
            hex.to_string()
        } else {
            format!("0{}", hex)
        };
        let mut bytes = Vec::with_capacity(hex_data.len() / 2);
        for i in (0..hex_data.len()).step_by(2) {
            let byte = u8::from_str_radix(&hex_data[i..i + 2], 16)
                .map_err(|e| parol_runtime::ParolError::UserError(anyhow::anyhow!(
                    "Invalid hex string '{}': {}",
                    hex_data,
                    e
                )))?;
            bytes.push(byte);
        }
        self.value_stack.push(ast::AsnValue::HexString(bytes));
        Ok(())
    }

    fn char_string(&mut self, arg: &CharString<'t>) -> Result<()> {
        let text = arg.char_string.text();
        let inner = &text[1..text.len() - 1];
        self.value_stack.push(ast::AsnValue::CharString(inner.to_string()));
        Ok(())
    }

    fn tag_default(&mut self, arg: &TagDefault) -> Result<()> {
        let td = match arg {
            TagDefault::EXPLICITTAGS(_) => ast::TagDefault::Explicit,
            TagDefault::IMPLICITTAGS(_) => ast::TagDefault::Implicit,
            TagDefault::AUTOMATICTAGS(_) => ast::TagDefault::Automatic,
        };
        self.tag_default_stack.push(td);
        Ok(())
    }



    fn definitive_obj_id_component(&mut self, arg: &DefinitiveObjIdComponent<'t>) -> Result<()> {
        match arg {
            DefinitiveObjIdComponent::Identifier(inner) => {
                self.oid_stack.push(ast::OidComponent::Name(inner.identifier.identifier.text().to_string()));
            }
            DefinitiveObjIdComponent::NumberLiteral(inner) => {
                let text = inner.number_literal.number_literal.text();
                let num: BigInt = text.parse().map_err(|e| anyhow::anyhow!("Invalid OID number: {text}: {e}"))?;
                self.oid_stack.push(ast::OidComponent::Number(num));
            }
        }
        Ok(())
    }

    fn definitive_obj_id_components(&mut self, arg: &DefinitiveObjIdComponents<'t>) -> Result<()> {
        let mut components: Vec<ast::OidComponent> = Vec::new();
        let count = 1 + arg.definitive_obj_id_components_list.len();
        for _ in 0..count {
            components.push(self.oid_stack.try_pop()?);
        }
        components.reverse();

        // Push complete OID to module_oid_stack for later consumption
        self.module_oid_stack.push(ast::ObjectIdentifier { components });
        Ok(())
    }

    fn definitive_identifier(&mut self, _arg: &DefinitiveIdentifier<'t>) -> Result<()> {
        Ok(())
    }

    fn module_identifier(&mut self, arg: &ModuleIdentifier<'t>) -> Result<()> {
        let name = self.str_stack.try_pop()?;
        let oid = if arg.module_identifier_opt.is_some() {
            let oid = self.module_oid_stack.try_pop()?;
            Some(oid)
        } else {
            None
        };
        self.module_id_stack.push(ast::ModuleIdentifier {
            name,
            oid,
            span: span(&arg.reference.reference),
        });
        Ok(())
    }

    fn export_symbol(&mut self, arg: &ExportSymbol<'t>) -> Result<()> {
        let name = match &*arg.identifier_or_keyword {
            IdentifierOrKeyword::ALL(_) => {
                self.export_is_all = true;
                return Ok(());
            }
            IdentifierOrKeyword::Reference(inner) => {
                self.export_is_all = false;
                let _raw_ref = self.str_stack.try_pop()?;
                inner.reference.reference.text().to_string()
            }
            IdentifierOrKeyword::Identifier(inner) => {
                self.export_is_all = false;
                inner.identifier.identifier.text().to_string()
            }
            IdentifierOrKeyword::ABSENT(inner) => { self.export_is_all = false; inner.a_b_s_e_n_t.text().to_string() }
            IdentifierOrKeyword::AnyType(inner) => { self.export_is_all = false; inner.any_type.text().to_string() }
            IdentifierOrKeyword::APPLICATION(inner) => { self.export_is_all = false; inner.a_p_p_l_i_c_a_t_i_o_n.text().to_string() }
            IdentifierOrKeyword::AUTOMATIC(inner) => { self.export_is_all = false; inner.a_u_t_o_m_a_t_i_c.text().to_string() }
            IdentifierOrKeyword::BEGIN(inner) => { self.export_is_all = false; inner.b_e_g_i_n.text().to_string() }
            IdentifierOrKeyword::BIT(inner) => { self.export_is_all = false; inner.b_i_t.text().to_string() }
            IdentifierOrKeyword::BMPString(inner) => { self.export_is_all = false; inner.b_m_p_string.text().to_string() }
            IdentifierOrKeyword::BooleanType(inner) => { self.export_is_all = false; inner.boolean_type.text().to_string() }
            IdentifierOrKeyword::BY(inner) => { self.export_is_all = false; inner.b_y.text().to_string() }
            IdentifierOrKeyword::CHARACTER(inner) => { self.export_is_all = false; inner.c_h_a_r_a_c_t_e_r.text().to_string() }
            IdentifierOrKeyword::CHOICE(inner) => { self.export_is_all = false; inner.c_h_o_i_c_e.text().to_string() }
            IdentifierOrKeyword::COMPONENT(inner) => { self.export_is_all = false; inner.c_o_m_p_o_n_e_n_t.text().to_string() }
            IdentifierOrKeyword::COMPONENTS(inner) => { self.export_is_all = false; inner.c_o_m_p_o_n_e_n_t_s.text().to_string() }
            IdentifierOrKeyword::CONSTRAINED(inner) => { self.export_is_all = false; inner.c_o_n_s_t_r_a_i_n_e_d.text().to_string() }
            IdentifierOrKeyword::CONTAINING(inner) => { self.export_is_all = false; inner.c_o_n_t_a_i_n_i_n_g.text().to_string() }
            IdentifierOrKeyword::DEFAULT(inner) => { self.export_is_all = false; inner.d_e_f_a_u_l_t.text().to_string() }
            IdentifierOrKeyword::DEFINED(inner) => { self.export_is_all = false; inner.d_e_f_i_n_e_d.text().to_string() }
            IdentifierOrKeyword::DEFINITIONS(inner) => { self.export_is_all = false; inner.d_e_f_i_n_i_t_i_o_n_s.text().to_string() }
            IdentifierOrKeyword::EMBEDDED(inner) => { self.export_is_all = false; inner.e_m_b_e_d_d_e_d.text().to_string() }
            IdentifierOrKeyword::END(inner) => { self.export_is_all = false; inner.e_n_d.text().to_string() }
            IdentifierOrKeyword::ENUMERATED(inner) => { self.export_is_all = false; inner.e_n_u_m_e_r_a_t_e_d.text().to_string() }
            IdentifierOrKeyword::EXCEPT(inner) => { self.export_is_all = false; inner.e_x_c_e_p_t.text().to_string() }
            IdentifierOrKeyword::EXPLICIT(inner) => { self.export_is_all = false; inner.e_x_p_l_i_c_i_t.text().to_string() }
            IdentifierOrKeyword::EXTENSIBILITY(inner) => { self.export_is_all = false; inner.e_x_t_e_n_s_i_b_i_l_i_t_y.text().to_string() }
            IdentifierOrKeyword::EXTERNAL(inner) => { self.export_is_all = false; inner.e_x_t_e_r_n_a_l.text().to_string() }
            IdentifierOrKeyword::FALSE(inner) => { self.export_is_all = false; inner.f_a_l_s_e.text().to_string() }
            IdentifierOrKeyword::FROM(inner) => { self.export_is_all = false; inner.f_r_o_m.text().to_string() }
            IdentifierOrKeyword::GeneralizedTimeType(inner) => { self.export_is_all = false; inner.generalized_time_type.text().to_string() }
            IdentifierOrKeyword::GraphicString(inner) => { self.export_is_all = false; inner.graphic_string.text().to_string() }
            IdentifierOrKeyword::IA5String(inner) => { self.export_is_all = false; inner.i_a5_string.text().to_string() }
            IdentifierOrKeyword::IDENTIFIER(inner) => { self.export_is_all = false; inner.i_d_e_n_t_i_f_i_e_r.text().to_string() }
            IdentifierOrKeyword::IMPLICIT(inner) => { self.export_is_all = false; inner.i_m_p_l_i_c_i_t.text().to_string() }
            IdentifierOrKeyword::IMPLIED(inner) => { self.export_is_all = false; inner.i_m_p_l_i_e_d.text().to_string() }
            IdentifierOrKeyword::IMPORTS(inner) => { self.export_is_all = false; inner.i_m_p_o_r_t_s.text().to_string() }
            IdentifierOrKeyword::INCLUDES(inner) => { self.export_is_all = false; inner.i_n_c_l_u_d_e_s.text().to_string() }
            IdentifierOrKeyword::INSTANCE(inner) => { self.export_is_all = false; inner.i_n_s_t_a_n_c_e.text().to_string() }
            IdentifierOrKeyword::INTEGER(inner) => { self.export_is_all = false; inner.i_n_t_e_g_e_r.text().to_string() }
            IdentifierOrKeyword::INTERSECTION(inner) => { self.export_is_all = false; inner.i_n_t_e_r_s_e_c_t_i_o_n.text().to_string() }
            IdentifierOrKeyword::ISO646String(inner) => { self.export_is_all = false; inner.i_s_o646_string.text().to_string() }
            IdentifierOrKeyword::MAX(inner) => { self.export_is_all = false; inner.m_a_x.text().to_string() }
            IdentifierOrKeyword::MIN(inner) => { self.export_is_all = false; inner.m_i_n.text().to_string() }
            IdentifierOrKeyword::MINUSMinusINFINITY(inner) => { self.export_is_all = false; inner.m_i_n_u_s_minus_i_n_f_i_n_i_t_y.text().to_string() }
            IdentifierOrKeyword::NullType(inner) => { self.export_is_all = false; inner.null_type.text().to_string() }
            IdentifierOrKeyword::NumericString(inner) => { self.export_is_all = false; inner.numeric_string.text().to_string() }
            IdentifierOrKeyword::OBJECT(inner) => { self.export_is_all = false; inner.o_b_j_e_c_t.text().to_string() }
            IdentifierOrKeyword::OCTET(inner) => { self.export_is_all = false; inner.o_c_t_e_t.text().to_string() }
            IdentifierOrKeyword::OF(inner) => { self.export_is_all = false; inner.o_f.text().to_string() }
            IdentifierOrKeyword::OPTIONAL(inner) => { self.export_is_all = false; inner.o_p_t_i_o_n_a_l.text().to_string() }
            IdentifierOrKeyword::PATTERN(inner) => { self.export_is_all = false; inner.p_a_t_t_e_r_n.text().to_string() }
            IdentifierOrKeyword::PDV(inner) => { self.export_is_all = false; inner.p_d_v.text().to_string() }
            IdentifierOrKeyword::PLUSMinusINFINITY(inner) => { self.export_is_all = false; inner.p_l_u_s_minus_i_n_f_i_n_i_t_y.text().to_string() }
            IdentifierOrKeyword::PRESENT(inner) => { self.export_is_all = false; inner.p_r_e_s_e_n_t.text().to_string() }
            IdentifierOrKeyword::PRIVATE(inner) => { self.export_is_all = false; inner.p_r_i_v_a_t_e.text().to_string() }
            IdentifierOrKeyword::PrintableString(inner) => { self.export_is_all = false; inner.printable_string.text().to_string() }
            IdentifierOrKeyword::RealType(inner) => { self.export_is_all = false; inner.real_type.text().to_string() }
            IdentifierOrKeyword::RelativeOidType(inner) => { self.export_is_all = false; inner.relative_oid_type.text().to_string() }
            IdentifierOrKeyword::SEQUENCE(inner) => { self.export_is_all = false; inner.s_e_q_u_e_n_c_e.text().to_string() }
            IdentifierOrKeyword::SET(inner) => { self.export_is_all = false; inner.s_e_t.text().to_string() }
            IdentifierOrKeyword::SIZE(inner) => { self.export_is_all = false; inner.s_i_z_e.text().to_string() }
            IdentifierOrKeyword::STRING(inner) => { self.export_is_all = false; inner.s_t_r_i_n_g.text().to_string() }
            IdentifierOrKeyword::SYNTAX(inner) => { self.export_is_all = false; inner.s_y_n_t_a_x.text().to_string() }
            IdentifierOrKeyword::T61String(inner) => { self.export_is_all = false; inner.t61_string.text().to_string() }
            IdentifierOrKeyword::TeletexString(inner) => { self.export_is_all = false; inner.teletex_string.text().to_string() }
            IdentifierOrKeyword::TRUE(inner) => { self.export_is_all = false; inner.t_r_u_e.text().to_string() }
            IdentifierOrKeyword::TYPE(inner) => { self.export_is_all = false; inner.t_y_p_e.text().to_string() }
            IdentifierOrKeyword::UNION(inner) => { self.export_is_all = false; inner.u_n_i_o_n.text().to_string() }
            IdentifierOrKeyword::UNIQUE(inner) => { self.export_is_all = false; inner.u_n_i_q_u_e.text().to_string() }
            IdentifierOrKeyword::UNIVERSAL(inner) => { self.export_is_all = false; inner.u_n_i_v_e_r_s_a_l.text().to_string() }
            IdentifierOrKeyword::UniversalString(inner) => { self.export_is_all = false; inner.universal_string.text().to_string() }
            IdentifierOrKeyword::UNRESTRICTED(inner) => { self.export_is_all = false; inner.u_n_r_e_s_t_r_i_c_t_e_d.text().to_string() }
            IdentifierOrKeyword::UTCTimeType(inner) => { self.export_is_all = false; inner.u_t_c_time_type.text().to_string() }
            IdentifierOrKeyword::UTF8String(inner) => { self.export_is_all = false; inner.u_t_f8_string.text().to_string() }
            IdentifierOrKeyword::VideotexString(inner) => { self.export_is_all = false; inner.videotex_string.text().to_string() }
            IdentifierOrKeyword::VisibleString(inner) => { self.export_is_all = false; inner.visible_string.text().to_string() }
            IdentifierOrKeyword::WITH(inner) => { self.export_is_all = false; inner.w_i_t_h.text().to_string() }
        };
        self.str_stack.push(name);
        Ok(())
    }

    fn export_list(&mut self, arg: &ExportList<'t>) -> Result<()> {
        let mut symbols: Vec<String> = Vec::new();
        for _ in &arg.export_list_list {
            symbols.push(self.str_stack.try_pop()?);
        }
        symbols.push(self.str_stack.try_pop()?);
        symbols.reverse();

        if self.export_is_all {
            self.export_stack.push(ast::ExportSymbols::All);
        } else {
            self.export_stack.push(ast::ExportSymbols::Symbols(symbols));
        }
        self.export_is_all = false;
        Ok(())
    }

    fn exports(&mut self, _arg: &Exports<'t>) -> Result<()> {
        Ok(())
    }

    fn import_symbol(&mut self, arg: &ImportSymbol<'t>) -> Result<()> {
        let name = match &*arg.identifier_or_keyword {
            IdentifierOrKeyword::Reference(inner) => {
                // The reference() callback already pushed the raw name, so pop it first
                let _raw_ref = self.str_stack.try_pop()?;
                inner.reference.reference.text().to_string()
            }
            IdentifierOrKeyword::Identifier(inner) => inner.identifier.identifier.text().to_string(),
            IdentifierOrKeyword::ALL(inner) => inner.a_l_l.text().to_string(),
            IdentifierOrKeyword::ABSENT(inner) => inner.a_b_s_e_n_t.text().to_string(),
            IdentifierOrKeyword::AnyType(inner) => inner.any_type.text().to_string(),
            IdentifierOrKeyword::APPLICATION(inner) => inner.a_p_p_l_i_c_a_t_i_o_n.text().to_string(),
            IdentifierOrKeyword::AUTOMATIC(inner) => inner.a_u_t_o_m_a_t_i_c.text().to_string(),
            IdentifierOrKeyword::BEGIN(inner) => inner.b_e_g_i_n.text().to_string(),
            IdentifierOrKeyword::BIT(inner) => inner.b_i_t.text().to_string(),
            IdentifierOrKeyword::BMPString(inner) => inner.b_m_p_string.text().to_string(),
            IdentifierOrKeyword::BooleanType(inner) => inner.boolean_type.text().to_string(),
            IdentifierOrKeyword::BY(inner) => inner.b_y.text().to_string(),
            IdentifierOrKeyword::CHARACTER(inner) => inner.c_h_a_r_a_c_t_e_r.text().to_string(),
            IdentifierOrKeyword::CHOICE(inner) => inner.c_h_o_i_c_e.text().to_string(),
            IdentifierOrKeyword::COMPONENT(inner) => inner.c_o_m_p_o_n_e_n_t.text().to_string(),
            IdentifierOrKeyword::COMPONENTS(inner) => inner.c_o_m_p_o_n_e_n_t_s.text().to_string(),
            IdentifierOrKeyword::CONSTRAINED(inner) => inner.c_o_n_s_t_r_a_i_n_e_d.text().to_string(),
            IdentifierOrKeyword::CONTAINING(inner) => inner.c_o_n_t_a_i_n_i_n_g.text().to_string(),
            IdentifierOrKeyword::DEFAULT(inner) => inner.d_e_f_a_u_l_t.text().to_string(),
            IdentifierOrKeyword::DEFINED(inner) => inner.d_e_f_i_n_e_d.text().to_string(),
            IdentifierOrKeyword::DEFINITIONS(inner) => inner.d_e_f_i_n_i_t_i_o_n_s.text().to_string(),
            IdentifierOrKeyword::EMBEDDED(inner) => inner.e_m_b_e_d_d_e_d.text().to_string(),
            IdentifierOrKeyword::END(inner) => inner.e_n_d.text().to_string(),
            IdentifierOrKeyword::ENUMERATED(inner) => inner.e_n_u_m_e_r_a_t_e_d.text().to_string(),
            IdentifierOrKeyword::EXCEPT(inner) => inner.e_x_c_e_p_t.text().to_string(),
            IdentifierOrKeyword::EXPLICIT(inner) => inner.e_x_p_l_i_c_i_t.text().to_string(),
            IdentifierOrKeyword::EXTENSIBILITY(inner) => inner.e_x_t_e_n_s_i_b_i_l_i_t_y.text().to_string(),
            IdentifierOrKeyword::EXTERNAL(inner) => inner.e_x_t_e_r_n_a_l.text().to_string(),
            IdentifierOrKeyword::FALSE(inner) => inner.f_a_l_s_e.text().to_string(),
            IdentifierOrKeyword::FROM(inner) => inner.f_r_o_m.text().to_string(),
            IdentifierOrKeyword::GeneralizedTimeType(inner) => inner.generalized_time_type.text().to_string(),
            IdentifierOrKeyword::GraphicString(inner) => inner.graphic_string.text().to_string(),
            IdentifierOrKeyword::IA5String(inner) => inner.i_a5_string.text().to_string(),
            IdentifierOrKeyword::IDENTIFIER(inner) => inner.i_d_e_n_t_i_f_i_e_r.text().to_string(),
            IdentifierOrKeyword::IMPLICIT(inner) => inner.i_m_p_l_i_c_i_t.text().to_string(),
            IdentifierOrKeyword::IMPLIED(inner) => inner.i_m_p_l_i_e_d.text().to_string(),
            IdentifierOrKeyword::IMPORTS(inner) => inner.i_m_p_o_r_t_s.text().to_string(),
            IdentifierOrKeyword::INCLUDES(inner) => inner.i_n_c_l_u_d_e_s.text().to_string(),
            IdentifierOrKeyword::INSTANCE(inner) => inner.i_n_s_t_a_n_c_e.text().to_string(),
            IdentifierOrKeyword::INTEGER(inner) => inner.i_n_t_e_g_e_r.text().to_string(),
            IdentifierOrKeyword::INTERSECTION(inner) => inner.i_n_t_e_r_s_e_c_t_i_o_n.text().to_string(),
            IdentifierOrKeyword::ISO646String(inner) => inner.i_s_o646_string.text().to_string(),
            IdentifierOrKeyword::MAX(inner) => inner.m_a_x.text().to_string(),
            IdentifierOrKeyword::MIN(inner) => inner.m_i_n.text().to_string(),
            IdentifierOrKeyword::MINUSMinusINFINITY(inner) => inner.m_i_n_u_s_minus_i_n_f_i_n_i_t_y.text().to_string(),
            IdentifierOrKeyword::NullType(inner) => inner.null_type.text().to_string(),
            IdentifierOrKeyword::NumericString(inner) => inner.numeric_string.text().to_string(),
            IdentifierOrKeyword::OBJECT(inner) => inner.o_b_j_e_c_t.text().to_string(),
            IdentifierOrKeyword::OCTET(inner) => inner.o_c_t_e_t.text().to_string(),
            IdentifierOrKeyword::OF(inner) => inner.o_f.text().to_string(),
            IdentifierOrKeyword::OPTIONAL(inner) => inner.o_p_t_i_o_n_a_l.text().to_string(),
            IdentifierOrKeyword::PATTERN(inner) => inner.p_a_t_t_e_r_n.text().to_string(),
            IdentifierOrKeyword::PDV(inner) => inner.p_d_v.text().to_string(),
            IdentifierOrKeyword::PLUSMinusINFINITY(inner) => inner.p_l_u_s_minus_i_n_f_i_n_i_t_y.text().to_string(),
            IdentifierOrKeyword::PRESENT(inner) => inner.p_r_e_s_e_n_t.text().to_string(),
            IdentifierOrKeyword::PRIVATE(inner) => inner.p_r_i_v_a_t_e.text().to_string(),
            IdentifierOrKeyword::PrintableString(inner) => inner.printable_string.text().to_string(),
            IdentifierOrKeyword::RealType(inner) => inner.real_type.text().to_string(),
            IdentifierOrKeyword::RelativeOidType(inner) => inner.relative_oid_type.text().to_string(),
            IdentifierOrKeyword::SEQUENCE(inner) => inner.s_e_q_u_e_n_c_e.text().to_string(),
            IdentifierOrKeyword::SET(inner) => inner.s_e_t.text().to_string(),
            IdentifierOrKeyword::SIZE(inner) => inner.s_i_z_e.text().to_string(),
            IdentifierOrKeyword::STRING(inner) => inner.s_t_r_i_n_g.text().to_string(),
            IdentifierOrKeyword::SYNTAX(inner) => inner.s_y_n_t_a_x.text().to_string(),
            IdentifierOrKeyword::T61String(inner) => inner.t61_string.text().to_string(),
            IdentifierOrKeyword::TeletexString(inner) => inner.teletex_string.text().to_string(),
            IdentifierOrKeyword::TRUE(inner) => inner.t_r_u_e.text().to_string(),
            IdentifierOrKeyword::TYPE(inner) => inner.t_y_p_e.text().to_string(),
            IdentifierOrKeyword::UNION(inner) => inner.u_n_i_o_n.text().to_string(),
            IdentifierOrKeyword::UNIQUE(inner) => inner.u_n_i_q_u_e.text().to_string(),
            IdentifierOrKeyword::UNIVERSAL(inner) => inner.u_n_i_v_e_r_s_a_l.text().to_string(),
            IdentifierOrKeyword::UniversalString(inner) => inner.universal_string.text().to_string(),
            IdentifierOrKeyword::UNRESTRICTED(inner) => inner.u_n_r_e_s_t_r_i_c_t_e_d.text().to_string(),
            IdentifierOrKeyword::UTCTimeType(inner) => inner.u_t_c_time_type.text().to_string(),
            IdentifierOrKeyword::UTF8String(inner) => inner.u_t_f8_string.text().to_string(),
            IdentifierOrKeyword::VideotexString(inner) => inner.videotex_string.text().to_string(),
            IdentifierOrKeyword::VisibleString(inner) => inner.visible_string.text().to_string(),
            IdentifierOrKeyword::WITH(inner) => inner.w_i_t_h.text().to_string(),
        };
        self.str_stack.push(format!("__import_sym__:{}", name));
        Ok(())
    }

    fn symbols_imported(&mut self, arg: &SymbolsImported<'t>) -> Result<()> {
        let mut symbols: Vec<String> = Vec::new();
        for _ in &arg.symbols_imported_list {
            let s = self.str_stack.try_pop()?;
            let name = s.strip_prefix("__import_sym__:").unwrap_or(&s).to_string();
            symbols.push(name);
        }
        let s = self.str_stack.try_pop()?;
        let name = s.strip_prefix("__import_sym__:").unwrap_or(&s).to_string();
        symbols.push(name);
        symbols.reverse();

        // Store as a single marker for import_item
        let marker = symbols.join(",");
        self.str_stack.push(format!("__import_list__:{}", marker));
        Ok(())
    }

    fn module_reference(&mut self, arg: &ModuleReference<'t>) -> Result<()> {
        self.str_stack.push(format!("__import_mod__:{}", arg.reference.reference.text()));
        Ok(())
    }

    fn import_item(&mut self, arg: &ImportItem<'t>) -> Result<()> {
        // Pop: module_reference first, then the raw Reference pushed by reference() callback,
        // then the symbols_imported list
        let module_str = self.str_stack.try_pop()?;
        let module = module_str.strip_prefix("__import_mod__:").unwrap_or(&module_str).to_string();

        // The reference() callback also fires for ModuleReference: Reference,
        // leaving a raw reference name on the stack that needs to be consumed
        let _raw_ref = self.str_stack.try_pop()?;

        let symbols_str = self.str_stack.try_pop()?;
        let symbols: Vec<String> = if let Some(rest) = symbols_str.strip_prefix("__import_list__:") {
            if rest.is_empty() {
                Vec::new()
            } else {
                rest.split(',').map(|s| s.to_string()).collect()
            }
        } else {
            Vec::new()
        };

        let module_oid = if arg.import_item_opt.is_some() {
            self.module_oid_stack.pop()
        } else {
            None
        };

        self.import_stack.push(ast::Import {
            symbols,
            module,
            module_oid,
        });
        Ok(())
    }

    fn import_list(&mut self, _arg: &ImportList<'t>) -> Result<()> {
        Ok(())
    }

    fn imports(&mut self, _arg: &Imports<'t>) -> Result<()> {
        Ok(())
    }

    fn boolean_type(&mut self, arg: &BooleanType<'t>) -> Result<()> {
        self.type_stack.push(ast::AsnType::Boolean { span: span(&arg.boolean_type) });
        Ok(())
    }

    fn integer_type(&mut self, arg: &IntegerType<'t>) -> Result<()> {
        let s = span(&arg.i_n_t_e_g_e_r);
        let named_numbers = if arg.integer_type_opt.is_some() {
            let mut nums = Vec::new();
            while let Some(nn) = self.named_number_stack.pop() {
                nums.push(nn);
            }
            nums.reverse();
            Some(nums)
        } else {
            None
        };
        let constraint = arg.integer_type_opt0.as_ref().and_then(|_| self.constraint_stack.pop());
        self.type_stack.push(ast::AsnType::Integer { named_numbers, constraint, span: s });
        Ok(())
    }

    fn real_type(&mut self, arg: &RealType<'t>) -> Result<()> {
        self.type_stack.push(ast::AsnType::Real { span: span(&arg.real_type) });
        Ok(())
    }

    fn enumerated_type(&mut self, arg: &EnumeratedType<'t>) -> Result<()> {
        let s = span(&arg.e_n_u_m_e_r_a_t_e_d);
        let (items, ext_items, extensible) = if let Some(ref opt) = arg.enumerated_type_opt {
            let root_items: Vec<ast::EnumItem> = Vec::new();

            if let Some(ref ext_opt) = opt.enumerated_type_opt0 {
                let ext_count = 1 + ext_opt.ext_additions.ext_additions_list.len();
                let _total = root_items.len();

                let mut all_items = Vec::new();
                while let Some(item) = self.enum_item_stack.pop() {
                    all_items.push(item);
                }

                if ext_count <= all_items.len() {
                    let ext = all_items.split_off(all_items.len() - ext_count);
                    (all_items, ext, true)
                } else {
                    (all_items, Vec::new(), false)
                }
            } else {
                let mut root = Vec::new();
                while let Some(item) = self.enum_item_stack.pop() {
                    root.push(item);
                }
                root.reverse();
                (root, Vec::new(), false)
            }
        } else {
            (Vec::new(), Vec::new(), false)
        };

        self.type_stack.push(ast::AsnType::Enumerated {
            items,
            extensible,
            ext_items,
            span: s,
        });
        Ok(())
    }

    fn bit_string_type(&mut self, arg: &BitStringType<'t>) -> Result<()> {
        let s = span(&arg.s_t_r_i_n_g);
        let named_bits = if arg.bit_string_type_opt.is_some() {
            let mut bits = Vec::new();
            while let Some(nb) = self.named_bit_stack.pop() {
                bits.push(nb);
            }
            bits.reverse();
            Some(bits)
        } else {
            None
        };
        self.type_stack.push(ast::AsnType::BitString { named_bits, constraint: arg.bit_string_type_opt0.as_ref().and_then(|_| self.constraint_stack.pop()), span: s });
        Ok(())
    }

    fn octet_string_type(&mut self, arg: &OctetStringType<'t>) -> Result<()> {
        let constraint = arg.octet_string_type_opt.as_ref().and_then(|_| self.constraint_stack.pop());
        self.type_stack.push(ast::AsnType::OctetString { constraint, span: span(&arg.s_t_r_i_n_g) });
        Ok(())
    }

    fn null_type(&mut self, arg: &NullType<'t>) -> Result<()> {
        self.type_stack.push(ast::AsnType::Null { span: span(&arg.null_type) });
        Ok(())
    }

    fn sequence_type(&mut self, arg: &SequenceType<'t>) -> Result<()> {
        let s = span(&arg.s_e_q_u_e_n_c_e);
        let (fields, ext_fields, extensible) = if let Some(ref opt) = arg.sequence_type_opt {
            let mut root = Vec::new();
            while let Some(comp) = self.component_stack.pop() {
                root.push(comp);
            }
            root.reverse();

            if let Some(ref ext_opt) = opt.sequence_type_opt0 {
                let ext_count = 1 + ext_opt.ext_add_sequence.ext_add_sequence_list.len();
                if ext_count <= root.len() {
                    let ext = root.split_off(root.len() - ext_count);
                    (root, ext, true)
                } else {
                    (root, Vec::new(), false)
                }
            } else {
                (root, Vec::new(), false)
            }
        } else {
            (Vec::new(), Vec::new(), false)
        };

        self.type_stack.push(ast::AsnType::Sequence {
            fields,
            extensible,
            ext_fields,
            span: s,
        });
        Ok(())
    }

    fn set_type(&mut self, arg: &SetType<'t>) -> Result<()> {
        let s = span(&arg.s_e_t);
        let (fields, ext_fields, extensible) = if let Some(ref opt) = arg.set_type_opt {
            let mut root = Vec::new();
            while let Some(comp) = self.component_stack.pop() {
                root.push(comp);
            }
            root.reverse();

            if let Some(ref ext_opt) = opt.set_type_opt0 {
                let ext_count = 1 + ext_opt.ext_add_set.ext_add_set_list.len();
                if ext_count <= root.len() {
                    let ext = root.split_off(root.len() - ext_count);
                    (root, ext, true)
                } else {
                    (root, Vec::new(), false)
                }
            } else {
                (root, Vec::new(), false)
            }
        } else {
            (Vec::new(), Vec::new(), false)
        };

        self.type_stack.push(ast::AsnType::Set {
            fields,
            extensible,
            ext_fields,
            span: s,
        });
        Ok(())
    }

    fn choice_type(&mut self, arg: &ChoiceType<'t>) -> Result<()> {
        let s = span(&arg.c_h_o_i_c_e);
        let (alts, ext_alts, extensible) = if let Some(ref opt) = arg.choice_type_opt {
            let mut root = Vec::new();
            while let Some(nt) = self.named_type_stack.pop() {
                root.push(nt);
            }
            root.reverse();

            if let Some(ref ext_opt) = opt.choice_type_opt0 {
                let ext_count = 1 + ext_opt.ext_add_alternatives.ext_add_alternatives_list.len();
                if ext_count <= root.len() {
                    let ext = root.split_off(root.len() - ext_count);
                    (root, ext, true)
                } else {
                    (root, Vec::new(), false)
                }
            } else {
                (root, Vec::new(), false)
            }
        } else {
            (Vec::new(), Vec::new(), false)
        };

        self.type_stack.push(ast::AsnType::Choice {
            alternatives: alts,
            extensible,
            ext_alternatives: ext_alts,
            span: s,
        });
        Ok(())
    }

    fn sequence_of_type(&mut self, _arg: &SequenceOfType<'t>) -> Result<()> {
        let element_type = self.type_stack.try_pop()?;
        let s = element_type.span();
        self.type_stack.push(ast::AsnType::SequenceOf {
            element_type: Box::new(element_type),
            span: s,
        });
        Ok(())
    }

    fn set_of_type(&mut self, _arg: &SetOfType<'t>) -> Result<()> {
        let element_type = self.type_stack.try_pop()?;
        let s = element_type.span();
        self.type_stack.push(ast::AsnType::SetOf {
            element_type: Box::new(element_type),
            span: s,
        });
        Ok(())
    }

    fn object_identifier_type(&mut self, arg: &ObjectIdentifierType<'t>) -> Result<()> {
        self.type_stack.push(ast::AsnType::ObjectIdentifier { span: span(&arg.i_d_e_n_t_i_f_i_e_r) });
        Ok(())
    }

    fn relative_oid_type(&mut self, arg: &RelativeOidType<'t>) -> Result<()> {
        self.type_stack.push(ast::AsnType::RelativeOid { span: span(&arg.relative_oid_type) });
        Ok(())
    }

    fn generalized_time_type(&mut self, arg: &GeneralizedTimeType<'t>) -> Result<()> {
        self.type_stack.push(ast::AsnType::GeneralizedTime { span: span(&arg.generalized_time_type) });
        Ok(())
    }

    fn u_t_c_time_type(&mut self, arg: &UTCTimeType<'t>) -> Result<()> {
        self.type_stack.push(ast::AsnType::UTCTime { span: span(&arg.u_t_c_time_type) });
        Ok(())
    }

    fn any_type(&mut self, arg: &AnyType<'t>) -> Result<()> {
        self.type_stack.push(ast::AsnType::Any { span: span(&arg.any_type) });
        Ok(())
    }

    fn open_type(&mut self, arg: &OpenType) -> Result<()> {
        let defined_by = arg.identifier.identifier.text().to_string();
        self.str_stack.pop();
        self.type_stack.push(ast::AsnType::OpenType { defined_by: Some(defined_by), span: span(&arg.identifier.identifier) });
        Ok(())
    }

    fn restricted_string_type(&mut self, arg: &RestrictedStringType<'t>) -> Result<()> {
        let base = &*arg.restricted_string_base;
        let (charset, s) = match base {
            RestrictedStringBase::UTF8String(inner) => (ast::CharsetType::UTF8, span(&inner.u_t_f8_string)),
            RestrictedStringBase::NumericString(inner) => (ast::CharsetType::Numeric, span(&inner.numeric_string)),
            RestrictedStringBase::PrintableString(inner) => (ast::CharsetType::Printable, span(&inner.printable_string)),
            RestrictedStringBase::TeletexString(inner) => (ast::CharsetType::Teletex, span(&inner.teletex_string)),
            RestrictedStringBase::T61String(inner) => (ast::CharsetType::Teletex, span(&inner.t61_string)),
            RestrictedStringBase::VideotexString(inner) => (ast::CharsetType::Videotex, span(&inner.videotex_string)),
            RestrictedStringBase::IA5String(inner) => (ast::CharsetType::IA5, span(&inner.i_a5_string)),
            RestrictedStringBase::GraphicString(inner) => (ast::CharsetType::Graphic, span(&inner.graphic_string)),
            RestrictedStringBase::VisibleString(inner) => (ast::CharsetType::Visible, span(&inner.visible_string)),
            RestrictedStringBase::ISO646String(inner) => (ast::CharsetType::General, span(&inner.i_s_o646_string)),
            RestrictedStringBase::GeneralString(inner) => (ast::CharsetType::General, span(&inner.general_string)),
            RestrictedStringBase::UniversalString(inner) => (ast::CharsetType::Universal, span(&inner.universal_string)),
            RestrictedStringBase::BMPString(inner) => (ast::CharsetType::BMP, span(&inner.b_m_p_string)),
        };
        let constraint = arg.restricted_string_type_opt.as_ref().and_then(|_| self.constraint_stack.pop());
        self.type_stack.push(ast::AsnType::RestrictedString { charset, constraint, span: s });
        Ok(())
    }

    fn unrestricted_string_type(&mut self, arg: &UnrestrictedStringType<'t>) -> Result<()> {
        let constraint = arg.unrestricted_string_type_opt.as_ref().and_then(|_| self.constraint_stack.pop());
        self.type_stack.push(ast::AsnType::UnrestrictedString { constraint, span: SourceSpan::from(0..0) });
        Ok(())
    }

    fn tag_class(&mut self, arg: &TagClass<'t>) -> Result<()> {
        let tc = match arg {
            TagClass::UNIVERSAL(_) => ast::TagClass::Universal,
            TagClass::APPLICATION(_) => ast::TagClass::Application,
            TagClass::PRIVATE(_) => ast::TagClass::Private,
        };
        // Store tag class info for tagged_type
        self.str_stack.push(format!("__tag_class__:{:?}", tc));
        Ok(())
    }

    fn tagged_type(&mut self, arg: &TaggedType<'t>) -> Result<()> {
        let inner = self.type_stack.try_pop()?;
        let number = self.bigint_stack.try_pop()?;

        let class = if let Some(ref _opt) = arg.tagged_type_opt {
            let s = self.str_stack.try_pop()?;
            s.strip_prefix("__tag_class__:")
                .map(|tc| match tc {
                    "Universal" => ast::TagClass::Universal,
                    "Application" => ast::TagClass::Application,
                    "Private" => ast::TagClass::Private,
                    _ => ast::TagClass::Universal,
                })
        } else {
            None
        };

        let implicit = if let Some(ref opt) = arg.tagged_type_opt0 {
            match &*opt.tagged_type_opt0_group {
                TaggedTypeOpt0Group::IMPLICIT(_) => Some(true),
                TaggedTypeOpt0Group::EXPLICIT(_) => Some(false),
            }
        } else {
            None
        };

        self.type_stack.push(ast::AsnType::Tagged {
            class,
            number,
            implicit,
            inner: Box::new(inner),
            span: span(&arg.number_literal.number_literal),
        });
        Ok(())
    }

    fn actual_parameter(&mut self, _arg: &ActualParameter<'t>) -> Result<()> {
        // Type is already pushed to type_stack by the Type callback
        let ty = self.type_stack.try_pop()?;
        self.actual_param_stack.push(ast::ActualParameter::Type(ty));
        Ok(())
    }

    fn referenced_type(&mut self, arg: &ReferencedType<'t>) -> Result<()> {
        let parameters = if let Some(opt) = &arg.referenced_type_opt {
            let mut params = Vec::new();
            let al = &opt.actual_parameter_list;
            let count = 1 + al.actual_parameter_list_list.len();
            for _ in 0..count {
                params.push(self.actual_param_stack.try_pop()?);
            }
            params.reverse();
            Some(params)
        } else {
            None
        };
        let name = self.str_stack.try_pop()?;
        self.type_stack.push(ast::AsnType::Referenced {
            name,
            parameters,
            span: span(&arg.reference.reference),
        });
        Ok(())
    }

    fn named_number(&mut self, arg: &NamedNumber<'t>) -> Result<()> {
        let name = self.str_stack.try_pop()?;
        let value = match &*arg.named_number_value {
            NamedNumberValue::NumberLiteral(inner) => {
                let text = inner.number_literal.number_literal.text();
                text.parse().map_err(|e| anyhow::anyhow!("Invalid number: {text}: {e}"))?
            }
            NamedNumberValue::Reference(_) => {
                let _ref_name = self.str_stack.try_pop()?;
                BigInt::from(0)
            }
        };
        self.named_number_stack.push(ast::NamedNumber { name, value });
        Ok(())
    }

    fn named_bit(&mut self, arg: &NamedBit<'t>) -> Result<()> {
        let name = self.str_stack.try_pop()?;
        let value = match &*arg.named_bit_value {
            NamedBitValue::NumberLiteral(inner) => {
                let text = inner.number_literal.number_literal.text();
                text.parse().map_err(|e| anyhow::anyhow!("Invalid number: {text}: {e}"))?
            }
            NamedBitValue::Reference(_) => {
                let _ref_name = self.str_stack.try_pop()?;
                BigInt::from(0)
            }
        };
        self.named_bit_stack.push(ast::NamedBit { name, value });
        Ok(())
    }

    fn enum_item(&mut self, arg: &EnumItem<'t>) -> Result<()> {
        let name = self.str_stack.try_pop()?;
        let value = if let Some(ref opt) = arg.enum_item_opt {
            let text = opt.number_literal.number_literal.text();
            Some(text.parse().map_err(|e| anyhow::anyhow!("Invalid enum value: {text}: {e}"))?)
        } else {
            None
        };
        self.enum_item_stack.push(ast::EnumItem { name, value });
        Ok(())
    }

    fn component_type(&mut self, arg: &ComponentType<'t>) -> Result<()> {
        let (optional, default) = match &*arg.component_type_rest {
            ComponentTypeRest::DEFAULTValue(_inner) => (false, Some(self.value_stack.try_pop()?)),
            ComponentTypeRest::OPTIONAL(_) => (true, None),
            ComponentTypeRest::ComponentTypeRestEmpty(_) => (false, None),
        };

        let ty = self.type_stack.try_pop()?;
        let name = self.str_stack.try_pop()?;

        self.component_stack.push(ast::ComponentType {
            name,
            ty,
            optional,
            default,
        });
        Ok(())
    }

    fn named_type(&mut self, _arg: &NamedType<'t>) -> Result<()> {
        let ty = self.type_stack.try_pop()?;
        let name = self.str_stack.try_pop()?;
        self.named_type_stack.push(ast::NamedType { name, ty });
        Ok(())
    }

    fn value(&mut self, arg: &Value<'t>) -> Result<()> {
        let val = match arg {
            Value::TRUE(_) => ast::AsnValue::Boolean(true),
            Value::FALSE(_) => ast::AsnValue::Boolean(false),
            Value::NumberLiteral(inner) => {
                let text = inner.number_literal.number_literal.text();
                let num: BigInt = text.parse().map_err(|e| anyhow::anyhow!("Invalid value: {text}: {e}"))?;
                ast::AsnValue::Integer(num)
            }
            Value::BinaryString(_) => self.value_stack.try_pop()?,
            Value::HexString(_) => self.value_stack.try_pop()?,
            Value::CharString(inner) => {
                let text = inner.char_string.char_string.text();
                ast::AsnValue::CharString(text[1..text.len() - 1].to_string())
            }
            Value::LBraceValueItemsRBrace(_inner) => {
                let items: Vec<ast::NamedValue> = self.named_value_stack.drain();
                ast::AsnValue::Sequence(items)
            }
        Value::Identifier(inner) => {
            let _ = self.str_stack.try_pop()?;
            ast::AsnValue::Referenced(inner.identifier.identifier.text().to_string())
        }
        Value::NullType(_) => ast::AsnValue::Null,
        Value::Reference(inner) => {
            let _ = self.str_stack.try_pop()?;
            ast::AsnValue::Referenced(inner.reference.reference.text().to_string())
        }
        };
        self.value_stack.push(val);
        Ok(())
    }

    fn value_item(&mut self, arg: &ValueItem<'t>) -> Result<()> {
        let named = match arg {
            ValueItem::IdentifierValueItemSuffix0(inner) => {
                let name = self.str_stack.try_pop()?;
                match &*inner.value_item_suffix0 {
                    ValueItemSuffix0::ColonValue(_colon_val) => {
                        let val = self.value_stack.try_pop()?;
                        ast::NamedValue { name, value: val }
                    }
                    ValueItemSuffix0::ValueItemSuffix0Empty(_) => {
                        ast::NamedValue {
                            name: String::new(),
                            value: ast::AsnValue::Referenced(name),
                        }
                    }
                }
            }
            ValueItem::NumberLiteral(inner) => {
                let text = inner.number_literal.number_literal.text();
                let num: BigInt = text.parse().map_err(|e| anyhow::anyhow!("Invalid value: {text}: {e}"))?;
                ast::NamedValue {
                    name: String::new(),
                    value: ast::AsnValue::Integer(num),
                }
            }
            ValueItem::ReferenceValueItemSuffix(inner) => {
                let name = self.str_stack.try_pop()?;
                match &*inner.value_item_suffix {
                    ValueItemSuffix::ColonValue(_colon_val) => {
                        let val = self.value_stack.try_pop()?;
                        ast::NamedValue { name, value: val }
                    }
                    ValueItemSuffix::ValueItemSuffixEmpty(_) => {
                        ast::NamedValue {
                            name: String::new(),
                            value: ast::AsnValue::Referenced(name),
                        }
                    }
                }
            }
        };
        self.named_value_stack.push(named);
        Ok(())
    }

    fn type_assignment(&mut self, arg: &TypeAssignment<'t>) -> Result<()> {
        let ty = self.type_stack.try_pop()?;
        let name = self.str_stack.try_pop()?;
        self.assignment_stack.push(ast::Assignment::Type(ast::TypeAssignment {
            name,
            parameters: None,
            ty,
            span: span(&arg.reference.reference),
        }));
        Ok(())
    }

    fn value_assignment(&mut self, arg: &ValueAssignment<'t>) -> Result<()> {
        let value = self.value_stack.try_pop()?;
        let ty = self.type_stack.try_pop()?;
        let name = self.str_stack.try_pop()?;
        self.assignment_stack.push(ast::Assignment::Value(ast::ValueAssignment {
            name,
            ty,
            value,
            span: span(&arg.reference.reference),
        }));
        Ok(())
    }

    fn assignment(&mut self, _arg: &Assignment<'t>) -> Result<()> {
        Ok(())
    }

    fn assignment_list(&mut self, _arg: &AssignmentList<'t>) -> Result<()> {
        Ok(())
    }

    fn module_body(&mut self, arg: &ModuleBody<'t>) -> Result<()> {
        let exports = if arg.module_body_opt.is_some() {
            self.export_stack.pop().map(|e| ast::Exports { symbols: e })
        } else {
            None
        };

        let imports: Vec<ast::Import> = if arg.module_body_opt0.is_some() {
            let mut list = Vec::new();
            while let Some(imp) = self.import_stack.pop() {
                list.push(imp);
            }
            list.reverse();
            list
        } else {
            Vec::new()
        };

        let assignments: Vec<ast::Assignment> = if arg.module_body_opt1.is_some() {
            let mut list = Vec::new();
            while let Some(a) = self.assignment_stack.pop() {
                list.push(a);
            }
            list.reverse();
            list
        } else {
            Vec::new()
        };

        self.module_body_stack.push(ast::ModuleBody {
            exports,
            imports,
            assignments,
        });
        Ok(())
    }

    fn module(&mut self, arg: &Module<'t>) -> Result<()> {
        let module_id = self.module_id_stack.try_pop()?;
        let body = self.module_body_stack.try_pop()?;
        let module_span = module_id.span;

        let tag_default = if arg.module_opt.is_some() {
            self.tag_default_stack.pop()
        } else {
            None
        };

        let ext_default = arg.module_opt0.is_some();

        self.result = Some(ast::Module {
            identifier: module_id,
            tag_default,
            ext_default,
            body,
            span: module_span,
        });
        Ok(())
    }

    // === Constraint parsing callbacks ===

    fn constraint(&mut self, arg: &Constraint<'t>) -> Result<()> {
        let spec = self.constraint_spec_stack.try_pop()?;
        let s = span(&arg.l_paren);
        self.constraint_stack.push(ast::Constraint { spec: Box::new(spec), span: s });
        Ok(())
    }

    fn constraint_spec(&mut self, arg: &ConstraintSpec<'t>) -> Result<()> {
        match arg {
            ConstraintSpec::DotDotValue(_inner) => {
                let val = self.value_stack.try_pop()?;
                self.constraint_spec_stack.push(ast::ConstraintSpec::Single(Box::new(
                    ast::SingleElementConstraint::ValueRange(ast::ValueRange {
                        min: ast::RangeValue::Min,
                        max: ast::RangeValue::Value(val),
                    }),
                )));
            }
            ConstraintSpec::ValueSingleValueOrRange(inner) => {
                let second_val = self.value_stack.try_pop()?;
                match &*inner.single_value_or_range {
                    SingleValueOrRange::DotDotSingleValueOrRangeOpt(_opt) => {
                        let first_val = self.value_stack.try_pop()?;
                        self.constraint_spec_stack.push(ast::ConstraintSpec::Single(Box::new(
                            ast::SingleElementConstraint::ValueRange(ast::ValueRange {
                                min: ast::RangeValue::Value(first_val),
                                max: ast::RangeValue::Value(second_val),
                            }),
                        )));
                    }
                    SingleValueOrRange::SingleValueOrRangeEmpty(_) => {
                        self.constraint_spec_stack.push(ast::ConstraintSpec::Single(Box::new(
                            ast::SingleElementConstraint::ValueRange(ast::ValueRange {
                                min: ast::RangeValue::Value(second_val),
                                max: ast::RangeValue::Max,
                            }),
                        )));
                    }
                }
            }
            ConstraintSpec::SizeConstraint(_inner) => {
                let inner_spec = self.constraint_spec_stack.try_pop()?;
                self.constraint_spec_stack.push(ast::ConstraintSpec::Single(Box::new(
                    ast::SingleElementConstraint::Size(Box::new(ast::Constraint {
                        spec: Box::new(inner_spec),
                        span: SourceSpan::from(0..0),
                    })),
                )));
            }
            ConstraintSpec::PermittedAlphabet(_inner) => {
                let inner_spec = self.constraint_spec_stack.try_pop()?;
                self.constraint_spec_stack.push(ast::ConstraintSpec::Single(Box::new(
                    ast::SingleElementConstraint::PermittedAlphabet(Box::new(ast::Constraint {
                        spec: Box::new(inner_spec),
                        span: SourceSpan::from(0..0),
                    })),
                )));
            }
        }
        Ok(())
    }
}
