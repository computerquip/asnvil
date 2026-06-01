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

pub struct Grammar<'t> {
    pub parse_tree: Option<parol_runtime::ParseTree>,
    pub result: Option<ast::Module>,
    _marker: PhantomData<&'t ()>,

    // Stacks for bottom-up AST construction
    str_stack: VecDeque<String>,
    bigint_stack: VecDeque<BigInt>,
    tag_default_stack: VecDeque<ast::TagDefault>,
    module_id_stack: VecDeque<ast::ModuleIdentifier>,
    module_body_stack: VecDeque<ast::ModuleBody>,
    export_stack: VecDeque<ast::ExportSymbols>,
    import_stack: VecDeque<ast::Import>,
    assignment_stack: VecDeque<ast::Assignment>,
    type_stack: VecDeque<ast::AsnType>,
    value_stack: VecDeque<ast::AsnValue>,
    named_value_stack: VecDeque<ast::NamedValue>,
    component_stack: VecDeque<ast::ComponentType>,
    named_type_stack: VecDeque<ast::NamedType>,
    enum_item_stack: VecDeque<ast::EnumItem>,
    named_number_stack: VecDeque<ast::NamedNumber>,
    named_bit_stack: VecDeque<ast::NamedBit>,
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
            str_stack: VecDeque::new(),
            bigint_stack: VecDeque::new(),
            tag_default_stack: VecDeque::new(),
            module_id_stack: VecDeque::new(),
            module_body_stack: VecDeque::new(),
            export_stack: VecDeque::new(),
            import_stack: VecDeque::new(),
            assignment_stack: VecDeque::new(),
            type_stack: VecDeque::new(),
            value_stack: VecDeque::new(),
            named_value_stack: VecDeque::new(),
            component_stack: VecDeque::new(),
            named_type_stack: VecDeque::new(),
            enum_item_stack: VecDeque::new(),
            named_number_stack: VecDeque::new(),
            named_bit_stack: VecDeque::new(),
        }
    }
}

impl<'t> GrammarTrait<'t> for Grammar<'t> {
    fn identifier(&mut self, arg: &Identifier<'t>) -> Result<()> {
        self.str_stack.push_back(arg.identifier.text().to_string());
        Ok(())
    }

    fn reference(&mut self, arg: &Reference<'t>) -> Result<()> {
        self.str_stack.push_back(arg.reference.text().to_string());
        Ok(())
    }

    fn number_literal(&mut self, arg: &NumberLiteral<'t>) -> Result<()> {
        let text = arg.number_literal.text();
        let value: BigInt = text.parse().map_err(|e| anyhow::anyhow!("Invalid number: {text}: {e}"))?;
        self.bigint_stack.push_back(value);
        Ok(())
    }

    fn binary_string(&mut self, arg: &BinaryString<'t>) -> Result<()> {
        let text = arg.binary_string.text();
        let bits: Vec<u8> = text[1..text.len() - 1]
            .chars()
            .map(|c| if c == '1' { 1 } else { 0 })
            .collect();
        self.value_stack.push_back(ast::AsnValue::BitString(bits));
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
        self.value_stack.push_back(ast::AsnValue::HexString(bytes));
        Ok(())
    }

    fn char_string(&mut self, arg: &CharString<'t>) -> Result<()> {
        let text = arg.char_string.text();
        let inner = &text[1..text.len() - 1];
        self.value_stack.push_back(ast::AsnValue::CharString(inner.to_string()));
        Ok(())
    }

    fn tag_default(&mut self, arg: &TagDefault) -> Result<()> {
        let td = match arg {
            TagDefault::EXPLICITTAGS(_) => ast::TagDefault::Explicit,
            TagDefault::IMPLICITTAGS(_) => ast::TagDefault::Implicit,
            TagDefault::AUTOMATICTAGS(_) => ast::TagDefault::Automatic,
        };
        self.tag_default_stack.push_back(td);
        Ok(())
    }

    fn extension_default(&mut self, _arg: &ExtensionDefault) -> Result<()> {
        // Just a marker - we don't need to push, module checks module_opt0.is_some()
        Ok(())
    }

    fn definitive_obj_id_component(&mut self, arg: &DefinitiveObjIdComponent<'t>) -> Result<()> {
        match arg {
            DefinitiveObjIdComponent::Identifier(inner) => {
                self.str_stack.push_back(format!("__oid_name__:{}", inner.identifier.identifier.text()));
            }
            DefinitiveObjIdComponent::NumberLiteral(inner) => {
                let text = inner.number_literal.number_literal.text();
                self.str_stack.push_back(format!("__oid_num__:{}", text));
            }
        }
        Ok(())
    }

    fn definitive_obj_id_components(&mut self, arg: &DefinitiveObjIdComponents<'t>) -> Result<()> {
        let mut components: Vec<ast::OidComponent> = Vec::new();
        let count = 1 + arg.definitive_obj_id_components_list.len();
        for _ in 0..count {
            let s = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
            if let Some(rest) = s.strip_prefix("__oid_name__:") {
                components.push(ast::OidComponent::Name(rest.to_string()));
            } else if let Some(rest) = s.strip_prefix("__oid_num__:") {
                let num: BigInt = rest.parse().map_err(|e| anyhow::anyhow!("Invalid OID number: {rest}: {e}"))?;
                components.push(ast::OidComponent::Number(num));
            }
        }
        components.reverse();

        // Store OID components as a marker for definitive_identifier
        let marker = components.iter().map(|c| match c {
            ast::OidComponent::Name(n) => format!("__oid_name__:{}", n),
            ast::OidComponent::Number(n) => format!("__oid_num__:{}", n),
        }).collect::<Vec<_>>().join(",");
        self.str_stack.push_back(format!("__oid__:{}", marker));
        Ok(())
    }

    fn definitive_identifier(&mut self, _arg: &DefinitiveIdentifier<'t>) -> Result<()> {
        // OID already on str_stack from definitive_obj_id_components
        Ok(())
    }

    fn module_identifier(&mut self, arg: &ModuleIdentifier<'t>) -> Result<()> {
        let name = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
        let oid = if arg.module_identifier_opt.is_some() {
            let s = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
            if let Some(rest) = s.strip_prefix("__oid__:") {
                let components: Vec<ast::OidComponent> = rest.split(',')
                    .filter(|s| !s.is_empty())
                    .map(|c| {
                        if let Some(name) = c.strip_prefix("__oid_name__:") {
                            ast::OidComponent::Name(name.to_string())
                        } else if let Some(num) = c.strip_prefix("__oid_num__:") {
                            let n: BigInt = num.parse().unwrap_or_default();
                            ast::OidComponent::Number(n)
                        } else {
                            ast::OidComponent::Name(c.to_string())
                        }
                    })
                    .collect();
                Some(ast::ObjectIdentifier { components })
            } else {
                None
            }
        } else {
            None
        };
        self.module_id_stack.push_back(ast::ModuleIdentifier {
            name,
            oid,
            span: span(&arg.reference.reference),
        });
        Ok(())
    }

    fn export_symbol(&mut self, arg: &ExportSymbol<'t>) -> Result<()> {
        let name = match &*arg.identifier_or_keyword {
            IdentifierOrKeyword::Reference(inner) => {
                // The reference() callback already pushed the raw name, so pop it first
                let _raw_ref = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
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
        self.str_stack.push_back(name);
        Ok(())
    }

    fn export_list(&mut self, arg: &ExportList<'t>) -> Result<()> {
        let mut symbols: Vec<String> = Vec::new();
        for _ in &arg.export_list_list {
            symbols.push(self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?);
        }
        symbols.push(self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?);
        symbols.reverse();

        if symbols.len() == 1 && symbols[0] == "ALL" {
            self.export_stack.push_back(ast::ExportSymbols::All);
        } else {
            self.export_stack.push_back(ast::ExportSymbols::Symbols(symbols));
        }
        Ok(())
    }

    fn exports(&mut self, _arg: &Exports<'t>) -> Result<()> {
        Ok(())
    }

    fn import_symbol(&mut self, arg: &ImportSymbol<'t>) -> Result<()> {
        let name = match &*arg.identifier_or_keyword {
            IdentifierOrKeyword::Reference(inner) => {
                // The reference() callback already pushed the raw name, so pop it first
                let _raw_ref = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
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
        self.str_stack.push_back(format!("__import_sym__:{}", name));
        Ok(())
    }

    fn symbols_imported(&mut self, arg: &SymbolsImported<'t>) -> Result<()> {
        let mut symbols: Vec<String> = Vec::new();
        for _ in &arg.symbols_imported_list {
            let s = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
            let name = s.strip_prefix("__import_sym__:").unwrap_or(&s).to_string();
            symbols.push(name);
        }
        let s = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
        let name = s.strip_prefix("__import_sym__:").unwrap_or(&s).to_string();
        symbols.push(name);
        symbols.reverse();

        // Store as a single marker for import_item
        let marker = symbols.join(",");
        self.str_stack.push_back(format!("__import_list__:{}", marker));
        Ok(())
    }

    fn module_reference(&mut self, arg: &ModuleReference<'t>) -> Result<()> {
        self.str_stack.push_back(format!("__import_mod__:{}", arg.reference.reference.text()));
        Ok(())
    }

    fn import_item(&mut self, arg: &ImportItem<'t>) -> Result<()> {
        // Pop: module_reference first, then the raw Reference pushed by reference() callback,
        // then the symbols_imported list
        let module_str = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
        let module = module_str.strip_prefix("__import_mod__:").unwrap_or(&module_str).to_string();

        // The reference() callback also fires for ModuleReference: Reference,
        // leaving a raw reference name on the stack that needs to be consumed
        let _raw_ref = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;

        let symbols_str = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
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
            // Check if there's an OID marker on the stack
            if let Some(s) = self.str_stack.back() {
                if s.starts_with("__oid__:") {
                    let s = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
                    let rest = s.strip_prefix("__oid__:").unwrap();
                    let components: Vec<ast::OidComponent> = rest.split(',')
                        .filter(|s| !s.is_empty())
                        .map(|c| {
                            if let Some(name) = c.strip_prefix("__oid_name__:") {
                                ast::OidComponent::Name(name.to_string())
                            } else if let Some(num) = c.strip_prefix("__oid_num__:") {
                                let n: BigInt = num.parse().unwrap_or_default();
                                ast::OidComponent::Number(n)
                            } else {
                                ast::OidComponent::Name(c.to_string())
                            }
                        })
                        .collect();
                    Some(ast::ObjectIdentifier { components })
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        self.import_stack.push_back(ast::Import {
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
        self.type_stack.push_back(ast::AsnType::Boolean { span: span(&arg.boolean_type) });
        Ok(())
    }

    fn integer_type(&mut self, arg: &IntegerType<'t>) -> Result<()> {
        let s = span(&arg.i_n_t_e_g_e_r);
        let named_numbers = if arg.integer_type_opt.is_some() {
            let mut nums = Vec::new();
            while let Some(nn) = self.named_number_stack.pop_back() {
                nums.push(nn);
            }
            nums.reverse();
            Some(nums)
        } else {
            None
        };
        self.type_stack.push_back(ast::AsnType::Integer { named_numbers, span: s });
        Ok(())
    }

    fn real_type(&mut self, arg: &RealType<'t>) -> Result<()> {
        self.type_stack.push_back(ast::AsnType::Real { span: span(&arg.real_type) });
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
                while let Some(item) = self.enum_item_stack.pop_back() {
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
                while let Some(item) = self.enum_item_stack.pop_back() {
                    root.push(item);
                }
                root.reverse();
                (root, Vec::new(), false)
            }
        } else {
            (Vec::new(), Vec::new(), false)
        };

        self.type_stack.push_back(ast::AsnType::Enumerated {
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
            while let Some(nb) = self.named_bit_stack.pop_back() {
                bits.push(nb);
            }
            bits.reverse();
            Some(bits)
        } else {
            None
        };
        self.type_stack.push_back(ast::AsnType::BitString { named_bits, span: s });
        Ok(())
    }

    fn octet_string_type(&mut self, arg: &OctetStringType<'t>) -> Result<()> {
        self.type_stack.push_back(ast::AsnType::OctetString { span: span(&arg.s_t_r_i_n_g) });
        Ok(())
    }

    fn null_type(&mut self, arg: &NullType<'t>) -> Result<()> {
        self.type_stack.push_back(ast::AsnType::Null { span: span(&arg.null_type) });
        Ok(())
    }

    fn sequence_type(&mut self, arg: &SequenceType<'t>) -> Result<()> {
        let s = span(&arg.s_e_q_u_e_n_c_e);
        let (fields, ext_fields, extensible) = if let Some(ref opt) = arg.sequence_type_opt {
            let mut root = Vec::new();
            while let Some(comp) = self.component_stack.pop_back() {
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

        self.type_stack.push_back(ast::AsnType::Sequence {
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
            while let Some(comp) = self.component_stack.pop_back() {
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

        self.type_stack.push_back(ast::AsnType::Set {
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
            while let Some(nt) = self.named_type_stack.pop_back() {
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

        self.type_stack.push_back(ast::AsnType::Choice {
            alternatives: alts,
            extensible,
            ext_alternatives: ext_alts,
            span: s,
        });
        Ok(())
    }

    fn sequence_of_type(&mut self, _arg: &SequenceOfType<'t>) -> Result<()> {
        let element_type = self.type_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected AsnType on type_stack"))?;
        let s = element_type.span();
        self.type_stack.push_back(ast::AsnType::SequenceOf {
            element_type: Box::new(element_type),
            span: s,
        });
        Ok(())
    }

    fn set_of_type(&mut self, _arg: &SetOfType<'t>) -> Result<()> {
        let element_type = self.type_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected AsnType on type_stack"))?;
        let s = element_type.span();
        self.type_stack.push_back(ast::AsnType::SetOf {
            element_type: Box::new(element_type),
            span: s,
        });
        Ok(())
    }

    fn object_identifier_type(&mut self, arg: &ObjectIdentifierType<'t>) -> Result<()> {
        self.type_stack.push_back(ast::AsnType::ObjectIdentifier { span: span(&arg.i_d_e_n_t_i_f_i_e_r) });
        Ok(())
    }

    fn relative_oid_type(&mut self, arg: &RelativeOidType<'t>) -> Result<()> {
        self.type_stack.push_back(ast::AsnType::RelativeOid { span: span(&arg.relative_oid_type) });
        Ok(())
    }

    fn generalized_time_type(&mut self, arg: &GeneralizedTimeType<'t>) -> Result<()> {
        self.type_stack.push_back(ast::AsnType::GeneralizedTime { span: span(&arg.generalized_time_type) });
        Ok(())
    }

    fn u_t_c_time_type(&mut self, arg: &UTCTimeType<'t>) -> Result<()> {
        self.type_stack.push_back(ast::AsnType::UTCTime { span: span(&arg.u_t_c_time_type) });
        Ok(())
    }

    fn any_type(&mut self, arg: &AnyType<'t>) -> Result<()> {
        self.type_stack.push_back(ast::AsnType::Any { span: span(&arg.any_type) });
        Ok(())
    }

    fn open_type(&mut self, arg: &OpenType) -> Result<()> {
        let defined_by = arg.identifier.identifier.text().to_string();
        self.str_stack.pop_back();
        self.type_stack.push_back(ast::AsnType::OpenType { defined_by: Some(defined_by), span: span(&arg.identifier.identifier) });
        Ok(())
    }

    fn restricted_string_type(&mut self, arg: &RestrictedStringType<'t>) -> Result<()> {
        let (charset, s) = match arg {
            RestrictedStringType::UTF8String(inner) => (ast::CharsetType::UTF8, span(&inner.u_t_f8_string)),
            RestrictedStringType::NumericString(inner) => (ast::CharsetType::Numeric, span(&inner.numeric_string)),
            RestrictedStringType::PrintableString(inner) => (ast::CharsetType::Printable, span(&inner.printable_string)),
            RestrictedStringType::TeletexString(inner) => (ast::CharsetType::Teletex, span(&inner.teletex_string)),
            RestrictedStringType::T61String(inner) => (ast::CharsetType::Teletex, span(&inner.t61_string)),
            RestrictedStringType::VideotexString(inner) => (ast::CharsetType::Videotex, span(&inner.videotex_string)),
            RestrictedStringType::IA5String(inner) => (ast::CharsetType::IA5, span(&inner.i_a5_string)),
            RestrictedStringType::GraphicString(inner) => (ast::CharsetType::Graphic, span(&inner.graphic_string)),
            RestrictedStringType::VisibleString(inner) => (ast::CharsetType::Visible, span(&inner.visible_string)),
            RestrictedStringType::ISO646String(inner) => (ast::CharsetType::General, span(&inner.i_s_o646_string)),
            RestrictedStringType::GeneralString(inner) => (ast::CharsetType::General, span(&inner.general_string)),
            RestrictedStringType::UniversalString(inner) => (ast::CharsetType::Universal, span(&inner.universal_string)),
            RestrictedStringType::BMPString(inner) => (ast::CharsetType::BMP, span(&inner.b_m_p_string)),
        };
        self.type_stack.push_back(ast::AsnType::RestrictedString { charset, span: s });
        Ok(())
    }

    fn unrestricted_string_type(&mut self, arg: &UnrestrictedStringType<'t>) -> Result<()> {
        self.type_stack.push_back(ast::AsnType::UnrestrictedString { span: span(&arg.unrestricted_string_type) });
        Ok(())
    }

    fn tag_class(&mut self, arg: &TagClass<'t>) -> Result<()> {
        let tc = match arg {
            TagClass::UNIVERSAL(_) => ast::TagClass::Universal,
            TagClass::APPLICATION(_) => ast::TagClass::Application,
            TagClass::PRIVATE(_) => ast::TagClass::Private,
        };
        // Store tag class info for tagged_type
        self.str_stack.push_back(format!("__tag_class__:{:?}", tc));
        Ok(())
    }

    fn tagged_type(&mut self, arg: &TaggedType<'t>) -> Result<()> {
        let inner = self.type_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected AsnType on type_stack"))?;
        let number = self.bigint_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected BigInt on bigint_stack"))?;

        let class = if let Some(ref _opt) = arg.tagged_type_opt {
            let s = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
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

        self.type_stack.push_back(ast::AsnType::Tagged {
            class,
            number,
            implicit,
            inner: Box::new(inner),
            span: span(&arg.number_literal.number_literal),
        });
        Ok(())
    }

    fn referenced_type(&mut self, arg: &ReferencedType<'t>) -> Result<()> {
        let name = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
        self.type_stack.push_back(ast::AsnType::Referenced {
            name,
            parameters: None,
            span: span(&arg.reference.reference),
        });
        Ok(())
    }

    fn named_number(&mut self, arg: &NamedNumber<'t>) -> Result<()> {
        let name = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
        let value = match &*arg.named_number_group {
            NamedNumberGroup::NumberLiteral(inner) => {
                let text = inner.number_literal.number_literal.text();
                text.parse().map_err(|e| anyhow::anyhow!("Invalid number: {text}: {e}"))?
            }
            NamedNumberGroup::Reference(_) => BigInt::from(0),
        };
        self.named_number_stack.push_back(ast::NamedNumber { name, value });
        Ok(())
    }

    fn named_bit(&mut self, arg: &NamedBit<'t>) -> Result<()> {
        let name = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
        let value = match &*arg.named_bit_group {
            NamedBitGroup::NumberLiteral(inner) => {
                let text = inner.number_literal.number_literal.text();
                text.parse().map_err(|e| anyhow::anyhow!("Invalid number: {text}: {e}"))?
            }
            NamedBitGroup::Reference(_) => BigInt::from(0),
        };
        self.named_bit_stack.push_back(ast::NamedBit { name, value });
        Ok(())
    }

    fn enum_item(&mut self, arg: &EnumItem<'t>) -> Result<()> {
        let name = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
        let value = if let Some(ref opt) = arg.enum_item_opt {
            let text = opt.number_literal.number_literal.text();
            Some(text.parse().map_err(|e| anyhow::anyhow!("Invalid enum value: {text}: {e}"))?)
        } else {
            None
        };
        self.enum_item_stack.push_back(ast::EnumItem { name, value });
        Ok(())
    }

    fn component_type(&mut self, arg: &ComponentType<'t>) -> Result<()> {
        let (optional, default) = match &*arg.component_type_rest {
            ComponentTypeRest::DEFAULTValue(_inner) => (false, Some(self.value_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected AsnValue on value_stack"))?)),
            ComponentTypeRest::OPTIONAL(_) => (true, None),
            ComponentTypeRest::ComponentTypeRestEmpty(_) => (false, None),
        };

        let ty = self.type_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected AsnType on type_stack"))?;
        let name = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;

        self.component_stack.push_back(ast::ComponentType {
            name,
            ty,
            optional,
            default,
        });
        Ok(())
    }

    fn named_type(&mut self, _arg: &NamedType<'t>) -> Result<()> {
        let ty = self.type_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected AsnType on type_stack"))?;
        let name = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
        self.named_type_stack.push_back(ast::NamedType { name, ty });
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
            Value::BinaryString(_) => self.value_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected AsnValue on value_stack"))?,
            Value::HexString(_) => self.value_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected AsnValue on value_stack"))?,
            Value::CharString(inner) => {
                let text = inner.char_string.char_string.text();
                ast::AsnValue::CharString(text[1..text.len() - 1].to_string())
            }
            Value::LBraceValueItemsRBrace(_inner) => {
                let items: Vec<ast::NamedValue> = self.named_value_stack.drain(..).rev().collect();
                ast::AsnValue::Sequence(items)
            }
        Value::Identifier(inner) => {
            let _ = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
            ast::AsnValue::Referenced(inner.identifier.identifier.text().to_string())
        }
        Value::NullType(_) => ast::AsnValue::Null,
        Value::Reference(inner) => {
            let _ = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
            ast::AsnValue::Referenced(inner.reference.reference.text().to_string())
        }
        };
        self.value_stack.push_back(val);
        Ok(())
    }

    fn value_item(&mut self, arg: &ValueItem<'t>) -> Result<()> {
        let named = match arg {
            ValueItem::Identifier(_inner) => {
                let name = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
                ast::NamedValue {
                    name: String::new(),
                    value: ast::AsnValue::Referenced(name),
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
            ValueItem::ReferenceColonValue(_inner) => {
                let name = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
                let val = self.value_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected AsnValue on value_stack"))?;
                ast::NamedValue { name, value: val }
            }
        };
        self.named_value_stack.push_back(named);
        Ok(())
    }

    fn type_assignment(&mut self, arg: &TypeAssignment<'t>) -> Result<()> {
        let ty = self.type_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected AsnType on type_stack"))?;
        let name = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
        self.assignment_stack.push_back(ast::Assignment::Type(ast::TypeAssignment {
            name,
            parameters: None,
            ty,
            span: span(&arg.reference.reference),
        }));
        Ok(())
    }

    fn value_assignment(&mut self, arg: &ValueAssignment<'t>) -> Result<()> {
        let value = self.value_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected AsnValue on value_stack"))?;
        let ty = self.type_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected AsnType on type_stack"))?;
        let name = self.str_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected string on str_stack"))?;
        self.assignment_stack.push_back(ast::Assignment::Value(ast::ValueAssignment {
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
            self.export_stack.pop_back().map(|e| ast::Exports { symbols: e })
        } else {
            None
        };

        let imports: Vec<ast::Import> = if arg.module_body_opt0.is_some() {
            let mut list = Vec::new();
            while let Some(imp) = self.import_stack.pop_back() {
                list.push(imp);
            }
            list.reverse();
            list
        } else {
            Vec::new()
        };

        let assignments: Vec<ast::Assignment> = if arg.module_body_opt1.is_some() {
            let mut list = Vec::new();
            while let Some(a) = self.assignment_stack.pop_back() {
                list.push(a);
            }
            list.reverse();
            list
        } else {
            Vec::new()
        };

        self.module_body_stack.push_back(ast::ModuleBody {
            exports: exports.or(Some(ast::Exports { symbols: ast::ExportSymbols::All })),
            imports,
            assignments,
        });
        Ok(())
    }

    fn module(&mut self, arg: &Module<'t>) -> Result<()> {
        let module_id = self.module_id_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected ModuleIdentifier on module_id_stack"))?;
        let body = self.module_body_stack.pop_back().ok_or_else(|| anyhow::anyhow!("Parser stack underflow: expected ModuleBody on module_body_stack"))?;
        let module_span = module_id.span;

        let tag_default = if arg.module_opt.is_some() {
            self.tag_default_stack.pop_back()
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
}
