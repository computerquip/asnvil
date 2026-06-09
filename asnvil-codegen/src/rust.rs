use crate::code_ast::*;
use crate::renderer::LanguageRenderer;
use anyhow::{bail, Result};
use askama::Template;

macro_rules! rust_tpl {
    ($name:ident, $path:expr, $($field:ident: $ty:ty),+ $(,)?) => {
        #[derive(Template)]
        #[template(path = $path, escape = "none")]
        struct $name<'a> {
            $($field: $ty),+
        }
    };
}

rust_tpl!(EncodeInteger, "rust/encode_integer.txt", indent: &'a str, encoder_type: &'a str, encoder_var: &'a str, value: &'a str, tag_class: &'a str, tag_number: u32);
rust_tpl!(EncodeBoolean, "rust/encode_boolean.txt", indent: &'a str, encoder_type: &'a str, encoder_var: &'a str, value: &'a str, tag_class: &'a str, tag_number: u32);
rust_tpl!(EncodeString, "rust/encode_string.txt", indent: &'a str, encoder_type: &'a str, encoder_var: &'a str, value: &'a str, tag_class: &'a str, tag_number: u32);
rust_tpl!(EncodeBytes, "rust/encode_bytes.txt", indent: &'a str, encoder_var: &'a str, value: &'a str, tag_class: &'a str, tag_number: u32);
rust_tpl!(EncodeBitString, "rust/encode_bit_string.txt", indent: &'a str, encoder_type: &'a str, encoder_var: &'a str, value: &'a str, tag_class: &'a str, tag_number: u32);
rust_tpl!(EncodeOid, "rust/encode_oid.txt", indent: &'a str, encoder_var: &'a str, value: &'a str, tag_class: &'a str, tag_number: u32);
rust_tpl!(EncodeNull, "rust/encode_null.txt", indent: &'a str, encoder_var: &'a str, tag_class: &'a str, tag_number: u32);
rust_tpl!(EncodeAny, "rust/encode_any.txt", indent: &'a str, encoder_var: &'a str, value: &'a str);
rust_tpl!(EncodeReferenced, "rust/encode_referenced.txt", indent: &'a str, encoder_var: &'a str, value: &'a str, encode_method: &'a str);
rust_tpl!(EncodeList, "rust/encode_list.txt", indent: &'a str, encoder_type: &'a str, encoder_var: &'a str, value: &'a str, tag_class: &'a str, tag_number: u32, element_encode: String);
rust_tpl!(EncodeWrapExplicit, "rust/encode_wrap_explicit.txt", indent: &'a str, encoder_type: &'a str, encoder_var: &'a str, outer_tag_class: &'a str, outer_tag_number: u32, inner_code: String);
rust_tpl!(EncodeWrapImplicit, "rust/encode_wrap_implicit.txt", indent: &'a str, encoder_type: &'a str, encoder_var: &'a str, outer_tag_class: &'a str, outer_tag_number: u32, inner_code: String);

rust_tpl!(DecodeInteger, "rust/decode_integer.txt", indent: &'a str, decoder_var: &'a str, name: &'a str, tag_class: &'a str, tag_number: u32);
rust_tpl!(DecodeBoolean, "rust/decode_boolean.txt", indent: &'a str, decoder_var: &'a str, name: &'a str, tag_class: &'a str, tag_number: u32);
rust_tpl!(DecodeString, "rust/decode_string.txt", indent: &'a str, decoder_var: &'a str, name: &'a str, tag_class: &'a str, tag_number: u32);
rust_tpl!(DecodeBytes, "rust/decode_bytes.txt", indent: &'a str, decoder_var: &'a str, name: &'a str, tag_class: &'a str, tag_number: u32);
rust_tpl!(DecodeBitString, "rust/decode_bit_string.txt", indent: &'a str, decoder_var: &'a str, name: &'a str, tag_class: &'a str, tag_number: u32);
rust_tpl!(DecodeOid, "rust/decode_oid.txt", indent: &'a str, decoder_var: &'a str, name: &'a str, tag_class: &'a str, tag_number: u32);
rust_tpl!(DecodeNull, "rust/decode_null.txt", indent: &'a str, decoder_var: &'a str, name: &'a str, tag_class: &'a str, tag_number: u32);
rust_tpl!(DecodeAny, "rust/decode_any.txt", indent: &'a str, decoder_var: &'a str, name: &'a str, tag_class: &'a str, tag_number: u32, reconstruct_tlv: bool);
rust_tpl!(DecodeReferenced, "rust/decode_referenced.txt", indent: &'a str, decoder_var: &'a str, name: &'a str, inner_type: &'a str, decode_method: &'a str);

fn render_tpl<T: Template>(t: T) -> String {
    t.render().expect("template render failed")
}

pub struct RustRenderer;

impl RustRenderer {
    pub fn new() -> Self {
        Self
    }

    fn render_type_internal(&self, ty: &TypeRef) -> Result<String> {
        match ty {
            TypeRef::Builtin(builtin) => match builtin {
                BuiltinType::Integer => Ok("num_bigint::BigInt".to_string()),
                BuiltinType::Boolean => Ok("bool".to_string()),
                BuiltinType::String(_) => Ok("String".to_string()),
                BuiltinType::OctetString => Ok("Vec<u8>".to_string()),
                BuiltinType::BitString => Ok("asnvil_runtime_rust::BitString".to_string()),
                BuiltinType::ObjectIdentifier => Ok("asnvil_runtime_rust::ObjectIdentifier".to_string()),
                BuiltinType::Null => Ok("()".to_string()),
                BuiltinType::Real => Ok("f64".to_string()),
                BuiltinType::GeneralizedTime | BuiltinType::UTCTime => Ok("String".to_string()),
                BuiltinType::Any => Ok("asnvil_runtime_rust::AsnAny".to_string()),
            },
            TypeRef::Named(name) => Ok(name.clone()),
            TypeRef::Optional(inner) => {
                let inner_type = self.render_type_internal(inner)?;
                Ok(format!("Option<{}>", inner_type))
            }
            TypeRef::List(inner) => {
                let inner_type = self.render_type_internal(inner)?;
                Ok(format!("Vec<{}>", inner_type))
            }
        }
    }

    fn tag_class_to_rust(&self, class: &str) -> String {
        match class.to_lowercase().as_str() {
            "universal" => "Universal".to_string(),
            "context" | "context-specific" => "Context".to_string(),
            "application" => "Application".to_string(),
            "private" => "Private".to_string(),
            _ => "Universal".to_string(),
        }
    }

    fn render_encode_stmt(&self, stmt: &EncodeStmt, encoder_var: &str, indent: &str, prefix: &str) -> Result<String> {
        let encoder_type = format!("{}Encoder", prefix);
        let tag_class_to_rust = |class: &str| self.tag_class_to_rust(class);
        
        match stmt {
            EncodeStmt::WriteInteger { value, tag, .. } => {
                Ok(render_tpl(EncodeInteger {
                    indent,
                    encoder_type: &encoder_type,
                    encoder_var,
                    value,
                    tag_class: &tag_class_to_rust(&tag.class),
                    tag_number: tag.number,
                }))
            }
            EncodeStmt::WriteBoolean { value, tag, .. } => {
                let val_expr = if value.starts_with("self.") {
                    format!("{}.unwrap_or(false)", value)
                } else if value.starts_with("val") {
                    format!("*{}", value)
                } else {
                    value.clone()
                };
                Ok(render_tpl(EncodeBoolean {
                    indent,
                    encoder_type: &encoder_type,
                    encoder_var,
                    value: &val_expr,
                    tag_class: &tag_class_to_rust(&tag.class),
                    tag_number: tag.number,
                }))
            }
            EncodeStmt::WriteString { value, tag, .. } => {
                Ok(render_tpl(EncodeString {
                    indent,
                    encoder_type: &encoder_type,
                    encoder_var,
                    value,
                    tag_class: &tag_class_to_rust(&tag.class),
                    tag_number: tag.number,
                }))
            }
            EncodeStmt::WriteBytes { value, tag, .. } => {
                Ok(render_tpl(EncodeBytes {
                    indent,
                    encoder_var,
                    value,
                    tag_class: &tag_class_to_rust(&tag.class),
                    tag_number: tag.number,
                }))
            }
            EncodeStmt::WriteBitString { value, tag, .. } => {
                Ok(render_tpl(EncodeBitString {
                    indent,
                    encoder_type: &encoder_type,
                    encoder_var,
                    value,
                    tag_class: &tag_class_to_rust(&tag.class),
                    tag_number: tag.number,
                }))
            }
            EncodeStmt::WriteOid { value, tag, .. } => {
                Ok(render_tpl(EncodeOid {
                    indent,
                    encoder_var,
                    value,
                    tag_class: &tag_class_to_rust(&tag.class),
                    tag_number: tag.number,
                }))
            }
            EncodeStmt::WriteNull { tag, .. } => {
                Ok(render_tpl(EncodeNull {
                    indent,
                    encoder_var,
                    tag_class: &tag_class_to_rust(&tag.class),
                    tag_number: tag.number,
                }))
            }
            EncodeStmt::WriteAny { value, .. } => {
                Ok(render_tpl(EncodeAny {
                    indent,
                    encoder_var,
                    value,
                }))
            }
            EncodeStmt::WriteReferenced { encode_method, value, .. } |
            EncodeStmt::WriteChoice { encode_method, value, .. } => {
                Ok(render_tpl(EncodeReferenced {
                    indent,
                    encoder_var,
                    value,
                    encode_method,
                }))
            }
            EncodeStmt::WriteList { tag, value, element_info, .. } => {
                let element_encode = match element_info.encoding.as_str() {
                    "constructed" | "referenced" | "choice" | "list" => {
                        format!("{indent}    let _encoded = _li.encode_{}()?", prefix.to_lowercase())
                    }
                    "integer" => format!(
                        "{indent}    let mut _e = asnvil_runtime_rust::{encoder_type}::new();\n\
                         {indent}    _e.write_integer(&_li)?;\n\
                         {indent}    let _encoded = _e.finish();",
                        indent = indent, encoder_type = encoder_type
                    ),
                    "boolean" => format!(
                        "{indent}    let mut _e = asnvil_runtime_rust::{encoder_type}::new();\n\
                         {indent}    _e.write_boolean(_li);\n\
                         {indent}    let _encoded = _e.finish();",
                        indent = indent, encoder_type = encoder_type
                    ),
                    "string" => format!(
                        "{indent}    let mut _e = asnvil_runtime_rust::{encoder_type}::new();\n\
                         {indent}    _e.write_bytes(_li.as_bytes());\n\
                         {indent}    let _encoded = _e.finish();",
                        indent = indent, encoder_type = encoder_type
                    ),
                    "bytes" => format!(
                        "{indent}    let _encoded = _li.clone();",
                        indent = indent
                    ),
                    _ => format!("{indent}    let _encoded = _li.encode_{}()?", prefix.to_lowercase()),
                };
                Ok(render_tpl(EncodeList {
                    indent,
                    encoder_type: &encoder_type,
                    encoder_var,
                    value,
                    tag_class: &tag_class_to_rust(&tag.class),
                    tag_number: tag.number,
                    element_encode,
                }))
            }
            EncodeStmt::WrapExplicit { outer_tag, inner_stmt } => {
                let inner_code = self.render_encode_stmt(inner_stmt, "_inner_encoder", "        ", prefix)?;
                Ok(render_tpl(EncodeWrapExplicit {
                    indent,
                    encoder_type: &encoder_type,
                    encoder_var,
                    outer_tag_class: &tag_class_to_rust(&outer_tag.class),
                    outer_tag_number: outer_tag.number,
                    inner_code,
                }))
            }
            EncodeStmt::WrapImplicit { outer_tag, inner_stmt, .. } => {
                let inner_code = self.render_encode_stmt(inner_stmt, "_inner_encoder", "        ", prefix)?;
                Ok(render_tpl(EncodeWrapImplicit {
                    indent,
                    encoder_type: &encoder_type,
                    encoder_var,
                    outer_tag_class: &tag_class_to_rust(&outer_tag.class),
                    outer_tag_number: outer_tag.number,
                    inner_code,
                }))
            }
            _ => Ok(format!("{indent}// Unsupported encode stmt: {:?}", stmt)),
        }
    }

    fn render_decode_stmt(&self, stmt: &DecodeStmt, decoder_var: &str, indent: &str, ber: Option<&BerFieldInfo>, prefix: &str) -> Result<String> {
        let tag_class = ber.map(|b| self.tag_class_to_rust(&b.tag_class)).unwrap_or_else(|| "Universal".to_string());
        let tag_number = ber.map(|b| b.tag_number).unwrap_or(0);
        
        match stmt {
            DecodeStmt::ReadInteger { name } => {
                Ok(render_tpl(DecodeInteger {
                    indent,
                    decoder_var,
                    name,
                    tag_class: &tag_class,
                    tag_number,
                }))
            }
            DecodeStmt::ReadBoolean { name } => {
                Ok(render_tpl(DecodeBoolean {
                    indent,
                    decoder_var,
                    name,
                    tag_class: &tag_class,
                    tag_number,
                }))
            }
            DecodeStmt::ReadString { name, .. } => {
                Ok(render_tpl(DecodeString {
                    indent,
                    decoder_var,
                    name,
                    tag_class: &tag_class,
                    tag_number,
                }))
            }
            DecodeStmt::ReadBytes { name } => {
                Ok(render_tpl(DecodeBytes {
                    indent,
                    decoder_var,
                    name,
                    tag_class: &tag_class,
                    tag_number,
                }))
            }
            DecodeStmt::ReadBitString { name } => {
                Ok(render_tpl(DecodeBitString {
                    indent,
                    decoder_var,
                    name,
                    tag_class: &tag_class,
                    tag_number,
                }))
            }
            DecodeStmt::ReadOid { name } => {
                Ok(render_tpl(DecodeOid {
                    indent,
                    decoder_var,
                    name,
                    tag_class: &tag_class,
                    tag_number,
                }))
            }
            DecodeStmt::ReadNull { name } => {
                Ok(render_tpl(DecodeNull {
                    indent,
                    decoder_var,
                    name,
                    tag_class: &tag_class,
                    tag_number,
                }))
            }
            DecodeStmt::ReadAny { name, reconstruct_tlv } => {
                Ok(render_tpl(DecodeAny {
                    indent,
                    decoder_var,
                    name,
                    tag_class: &tag_class,
                    tag_number,
                    reconstruct_tlv: *reconstruct_tlv,
                }))
            }
            DecodeStmt::ReadReferenced { name, inner_type, decode_method: _, .. } |
            DecodeStmt::ReadChoice { name, inner_type, decode_method: _, .. } => {
                let decode_method = if prefix == "Oer" { "decode_oer_from" } else { "decode_der_from" };
                Ok(render_tpl(DecodeReferenced {
                    indent,
                    decoder_var,
                    name,
                    inner_type,
                    decode_method,
                }))
            }
            _ => Ok(format!("{indent}// Unsupported decode stmt: {:?}", stmt)),
        }
    }
}

impl Default for RustRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageRenderer for RustRenderer {
    fn language_name(&self) -> &str {
        "rust"
    }

    fn render_module(&self, ast: &CodeAstNode) -> Result<String> {
        match ast {
            CodeAstNode::Module {
                name: _,
                imports: _,
                declarations,
                doc_comment,
            } => {
                let mut output = String::new();
                output.push_str("// Generated by asnvil\n");
                if let Some(doc) = doc_comment {
                    output.push_str(&format!("/// {}\n\n", doc));
                }
                output.push_str("use asnvil_runtime_rust::{AsnError, Tag, TagClass, DerEncoder, DerDecoder, OerEncoder, OerDecoder, BitString, ObjectIdentifier, AsnAny};\n");
                output.push_str("use num_bigint::BigInt;\n\n");

                for decl in declarations {
                    output.push_str(&self.render_declaration(decl)?);
                    output.push_str("\n\n");
                }

                Ok(output)
            }
            _ => bail!("Expected Module node"),
        }
    }

    fn render_declaration(&self, decl: &Declaration) -> Result<String> {
        match decl {
            Declaration::Struct { name, fields, doc_comment, annotations: _ } => {
                let mut output = String::new();
                if let Some(doc) = doc_comment {
                    output.push_str(&format!("/// {}\n", doc));
                }
                output.push_str("#[derive(Debug, Clone, PartialEq)]\n");
                output.push_str(&format!("pub struct {} {{\n", name));
                for field in fields {
                    if let Some(doc) = &field.doc_comment {
                        output.push_str(&format!("    /// {}\n", doc));
                    }
                    let ty = self.render_type_internal(&field.ty)?;
                    output.push_str(&format!("    pub {}: {},\n", field.name, ty));
                }
                output.push_str("}\n\n");

                // Generate encode_der
                output.push_str(&format!("impl {} {{\n", name));
                output.push_str("    pub fn encode_der(&self) -> Result<Vec<u8>, AsnError> {\n");
                output.push_str("        let mut encoder = DerEncoder::new();\n");
                output.push_str("        encoder.write_tag(TagClass::Universal, 16, true)?; // SEQUENCE\n");
                output.push_str("        let mut content_encoder = DerEncoder::new();\n");
                
                for field in fields {
                    for stmt in &field.encode_stmts {
                        output.push_str(&self.render_encode_stmt(stmt, "content_encoder", "        ", "Der")?);
                        output.push_str("\n");
                    }
                }
                
                output.push_str("        let content = content_encoder.finish();\n");
                output.push_str("        encoder.write_length(content.len())?;\n");
                output.push_str("        encoder.write_bytes(&content);\n");
                output.push_str("        Ok(encoder.finish())\n");
                output.push_str("    }\n\n");

                // Generate decode_der
                output.push_str("    pub fn decode_der(data: &[u8]) -> Result<Self, AsnError> {\n");
                output.push_str("        let mut decoder = DerDecoder::new(data);\n");
                output.push_str("        Self::decode_der_from(&mut decoder)\n");
                output.push_str("    }\n\n");

                output.push_str("    pub fn decode_der_from(decoder: &mut DerDecoder) -> Result<Self, AsnError> {\n");
                output.push_str("        let (tag_class, tag_number, constructed) = decoder.read_tag()?;\n");
                output.push_str("        if tag_class != TagClass::Universal || tag_number != 16 || !constructed {\n");
                output.push_str("            return Err(AsnError::UnexpectedTag { \n");
                output.push_str("                expected: Tag { tag_class: TagClass::Universal, number: 16, constructed: true },\n");
                output.push_str("                actual: Tag { tag_class, number: tag_number, constructed },\n");
                output.push_str("            });\n");
                output.push_str("        }\n");
                output.push_str("        let _len = decoder.read_length()?;\n");
                output.push_str("        let _end_pos = decoder.pos + _len;\n\n");

                for field in fields {
                    if field.optional {
                        let _ty = self.render_type_internal(&field.ty)?;
                        output.push_str(&format!("        let mut {} = None;\n", field.name));
                    } else {
                        output.push_str(&format!("        let mut {} = None;\n", field.name)); // We'll assign Some, then unwrap
                    }
                }
                output.push_str("\n");
                
                // Decode fields in order
                for field in fields {
                    if field.optional {
                        output.push_str(&format!("        if decoder.pos < _end_pos {{\n"));
                        output.push_str(&format!("            let _start_pos = decoder.pos;\n"));
                        output.push_str(&format!("            if let Ok(_tag) = decoder.read_tag() {{\n"));
                        if let Some(ber) = &field.ber {
                            output.push_str(&format!("                if _tag.0 == asnvil_runtime_rust::TagClass::{} && _tag.1 == {} {{\n", self.tag_class_to_rust(&ber.tag_class), ber.tag_number));
                            output.push_str(&format!("                    decoder.rewind_to(_start_pos);\n"));
                            for stmt in &field.decode_stmts {
                                output.push_str(&self.render_decode_stmt(stmt, "decoder", "                    ", Some(ber), "Der")?);
                                output.push_str("\n");
                            }
                            output.push_str(&format!("                }}\n"));
                        }
                        output.push_str(&format!("            }}\n"));
                        output.push_str(&format!("        }}\n"));
                    } else {
                        for stmt in &field.decode_stmts {
                            output.push_str(&self.render_decode_stmt(stmt, "decoder", "        ", field.ber.as_ref(), "Der")?);
                            output.push_str("\n");
                        }
                    }
                }

                output.push_str("        decoder.pos = _end_pos;\n\n");
                output.push_str("        Ok(Self {\n");
                for field in fields {
                    if field.optional {
                        output.push_str(&format!("            {},\n", field.name));
                    } else {
                        output.push_str(&format!("            {}: {}.unwrap(),\n", field.name, field.name));
                    }
                }
                output.push_str("        })\n");
                output.push_str("    }\n\n");

                // Generate encode_oer
                output.push_str("    pub fn encode_oer(&self) -> Result<Vec<u8>, AsnError> {\n");
                output.push_str("        let mut encoder = OerEncoder::new();\n");
                output.push_str("        encoder.write_tag(TagClass::Universal, 16, true)?; // SEQUENCE\n");
                output.push_str("        let mut content_encoder = OerEncoder::new();\n");
                
                for field in fields {
                    for stmt in &field.encode_stmts {
                        output.push_str(&self.render_encode_stmt(stmt, "content_encoder", "        ", "Oer")?);
                        output.push_str("\n");
                    }
                }
                
                output.push_str("        let content = content_encoder.finish();\n");
                output.push_str("        encoder.write_length(content.len())?;\n");
                output.push_str("        encoder.write_bytes(&content);\n");
                output.push_str("        Ok(encoder.finish())\n");
                output.push_str("    }\n\n");

                // Generate decode_oer
                output.push_str("    pub fn decode_oer(data: &[u8]) -> Result<Self, AsnError> {\n");
                output.push_str("        let mut decoder = OerDecoder::new(data);\n");
                output.push_str("        Self::decode_oer_from(&mut decoder)\n");
                output.push_str("    }\n\n");

                output.push_str("    pub fn decode_oer_from(decoder: &mut OerDecoder) -> Result<Self, AsnError> {\n");
                output.push_str("        let (tag_class, tag_number, constructed) = decoder.read_tag()?;\n");
                output.push_str("        if tag_class != TagClass::Universal || tag_number != 16 || !constructed {\n");
                output.push_str("            return Err(AsnError::UnexpectedTag { \n");
                output.push_str("                expected: Tag { tag_class: TagClass::Universal, number: 16, constructed: true },\n");
                output.push_str("                actual: Tag { tag_class, number: tag_number, constructed },\n");
                output.push_str("            });\n");
                output.push_str("        }\n");
                output.push_str("        let _len = decoder.read_length()?;\n");
                output.push_str("        let _end_pos = decoder.pos + _len;\n\n");

                for field in fields {
                    if field.optional {
                        output.push_str(&format!("        let mut {} = None;\n", field.name));
                    } else {
                        output.push_str(&format!("        let mut {} = None;\n", field.name));
                    }
                }
                output.push_str("\n");
                
                for field in fields {
                    if field.optional {
                        output.push_str(&format!("        if decoder.pos < _end_pos {{\n"));
                        output.push_str(&format!("            let _start_pos = decoder.pos;\n"));
                        output.push_str(&format!("            if let Ok(_tag) = decoder.read_tag() {{\n"));
                        if let Some(ber) = &field.ber {
                            output.push_str(&format!("                if _tag.0 == asnvil_runtime_rust::TagClass::{} && _tag.1 == {} {{\n", self.tag_class_to_rust(&ber.tag_class), ber.tag_number));
                            output.push_str(&format!("                    decoder.rewind_to(_start_pos);\n"));
                            for stmt in &field.decode_stmts {
                                output.push_str(&self.render_decode_stmt(stmt, "decoder", "                    ", Some(ber), "Oer")?);
                                output.push_str("\n");
                            }
                            output.push_str(&format!("                }}\n"));
                        }
                        output.push_str(&format!("            }}\n"));
                        output.push_str(&format!("        }}\n"));
                    } else {
                        for stmt in &field.decode_stmts {
                            output.push_str(&self.render_decode_stmt(stmt, "decoder", "        ", field.ber.as_ref(), "Oer")?);
                            output.push_str("\n");
                        }
                    }
                }

                output.push_str("        decoder.pos = _end_pos;\n\n");
                output.push_str("        Ok(Self {\n");
                for field in fields {
                    if field.optional {
                        output.push_str(&format!("            {},\n", field.name));
                    } else {
                        output.push_str(&format!("            {}: {}.unwrap(),\n", field.name, field.name));
                    }
                }
                output.push_str("        })\n");
                output.push_str("    }\n");
                output.push_str("}\n");

                Ok(output)
            }
            Declaration::Enum { name, variants, repr, doc_comment } => {
                let mut output = String::new();
                if let Some(doc) = doc_comment {
                    output.push_str(&format!("/// {}\n", doc));
                }
                let repr_str = match repr {
                    Some(EnumRepr::Int) => "#[repr(i64)]\n",
                    None => "",
                };
                output.push_str(repr_str);
                output.push_str("#[derive(Debug, Clone, PartialEq)]\n");
                output.push_str(&format!("pub enum {} {{\n", name));
                let mut next_value = 0i64;
                for variant in variants {
                    if let Some(doc) = &variant.doc_comment {
                        output.push_str(&format!("    /// {}\n", doc));
                    }
                    let value = variant.value.unwrap_or(next_value);
                    next_value = value + 1;
                    output.push_str(&format!("    {} = {},\n", variant.name, value));
                }
                output.push_str("}\n");
                Ok(output)
            }
            Declaration::Choice { name, alternatives, doc_comment } => {
                let mut output = String::new();
                if let Some(doc) = doc_comment {
                    output.push_str(&format!("/// {}\n", doc));
                }
                output.push_str("#[derive(Debug, Clone, PartialEq)]\n");
                output.push_str(&format!("pub enum {} {{\n", name));
                for alt in alternatives {
                    let ty = self.render_type_internal(&alt.ty)?;
                    // Capitalize the variant name for Rust conventions
                    let mut variant_name = alt.name.clone();
                    if let Some(first_char) = variant_name.chars().next() {
                        variant_name = first_char.to_uppercase().collect::<String>() + &variant_name[1..];
                    }
                    output.push_str(&format!("    {}({}),\n", variant_name, ty));
                }
                output.push_str("}\n\n");

                // Generate encode_der
                output.push_str(&format!("impl {} {{\n", name));
                output.push_str("    pub fn encode_der(&self) -> Result<Vec<u8>, AsnError> {\n");
                output.push_str("        match self {\n");
                for (_i, alt) in alternatives.iter().enumerate() {
                    if let Some(ber) = &alt.ber {
                        let mut variant_name = alt.name.clone();
                        if let Some(first_char) = variant_name.chars().next() {
                            variant_name = first_char.to_uppercase().collect::<String>() + &variant_name[1..];
                        }
                        output.push_str(&format!("            {}::{}(val) => {{\n", name, variant_name));
                        output.push_str("                let mut encoder = DerEncoder::new();\n");
                        
                        // Check if the first statement is WrapExplicit
                        let is_wrap_explicit = alt.encode_stmts.first().map_or(false, |s| matches!(s, EncodeStmt::WrapExplicit { .. }));
                        
                        if is_wrap_explicit {
                            output.push_str("                let mut content_encoder = DerEncoder::new();\n");
                            for stmt in &alt.encode_stmts {
                                let mut stmt_str = self.render_encode_stmt(stmt, "content_encoder", "                ", "Der")?;
                                let variant_name_lower = &alt.name;
                                stmt_str = stmt_str.replace(&format!("self.{}.unwrap_or(false)", variant_name_lower), "*val");
                                stmt_str = stmt_str.replace(&format!("self.{}", variant_name_lower), "val");
                                stmt_str = stmt_str.replace("self.", "val.");
                                output.push_str(&stmt_str);
                                output.push_str("\n");
                            }
                            output.push_str("                let content = content_encoder.finish();\n");
                            output.push_str("                encoder.write_bytes(&content);\n");
                        } else {
                            output.push_str("                let mut content_encoder = DerEncoder::new();\n");
                            for stmt in &alt.encode_stmts {
                                let mut stmt_str = self.render_encode_stmt(stmt, "content_encoder", "                ", "Der")?;
                                let variant_name_lower = &alt.name;
                                stmt_str = stmt_str.replace(&format!("self.{}.unwrap_or(false)", variant_name_lower), "*val");
                                stmt_str = stmt_str.replace(&format!("self.{}", variant_name_lower), "val");
                                stmt_str = stmt_str.replace("self.", "val.");
                                output.push_str(&stmt_str);
                                output.push_str("\n");
                            }
                            output.push_str("                let content = content_encoder.finish();\n");
                            output.push_str(&format!("                encoder.write_tag(asnvil_runtime_rust::TagClass::{}, {}, true)?;\n", self.tag_class_to_rust(&ber.tag_class), ber.tag_number));
                            output.push_str("                encoder.write_length(content.len())?;\n");
                            output.push_str("                encoder.write_bytes(&content);\n");
                        }
                        output.push_str("                Ok(encoder.finish())\n");
                        output.push_str("            }\n");
                    }
                }
                output.push_str("        }\n");
                output.push_str("    }\n\n");

                // Generate decode_der
                output.push_str("    pub fn decode_der(data: &[u8]) -> Result<Self, AsnError> {\n");
                output.push_str("        let mut decoder = DerDecoder::new(data);\n");
                output.push_str("        Self::decode_der_from(&mut decoder)\n");
                output.push_str("    }\n\n");

                output.push_str("    pub fn decode_der_from(decoder: &mut DerDecoder) -> Result<Self, AsnError> {\n");
                output.push_str("        let _start_pos = decoder.pos;\n");
                output.push_str("        let (tag_class, tag_number, constructed) = decoder.read_tag()?;\n");
                output.push_str("        let _len = decoder.read_length()?;\n\n");
                
                output.push_str("        match (tag_class, tag_number) {\n");
                for alt in alternatives {
                    if let Some(ber) = &alt.ber {
                        let mut variant_name = alt.name.clone();
                        if let Some(first_char) = variant_name.chars().next() {
                            variant_name = first_char.to_uppercase().collect::<String>() + &variant_name[1..];
                        }
                        output.push_str(&format!("            (asnvil_runtime_rust::TagClass::{}, {}) => {{\n", self.tag_class_to_rust(&ber.tag_class), ber.tag_number));
                        output.push_str(&format!("                let mut {} = None;\n", alt.name));
                        for stmt in &alt.decode_stmts {
                            let stmt_str = self.render_decode_stmt(stmt, "decoder", "                ", Some(ber), "Der")?;
                            output.push_str(&stmt_str);
                            output.push_str("\n");
                        }
                        output.push_str(&format!("                Ok({}::{}({}.unwrap()))\n", name, variant_name, alt.name));
                        output.push_str("            }\n");
                    }
                }
                output.push_str("            _ => Err(AsnError::UnexpectedTag {\n");
                output.push_str("                expected: Tag { tag_class: TagClass::Universal, number: 0, constructed: false },\n");
                output.push_str("                actual: Tag { tag_class, number: tag_number, constructed },\n");
                output.push_str("            }),\n");
                output.push_str("        }\n");
                output.push_str("    }\n\n");

                // Generate decode_oer
                output.push_str("    pub fn decode_oer(data: &[u8]) -> Result<Self, AsnError> {\n");
                output.push_str("        let mut decoder = OerDecoder::new(data);\n");
                output.push_str("        Self::decode_oer_from(&mut decoder)\n");
                output.push_str("    }\n\n");

                output.push_str("    pub fn decode_oer_from(decoder: &mut OerDecoder) -> Result<Self, AsnError> {\n");
                output.push_str("        let _start_pos = decoder.pos;\n");
                output.push_str("        let (tag_class, tag_number, constructed) = decoder.read_tag()?;\n");
                output.push_str("        let _len = decoder.read_length()?;\n\n");
                
                output.push_str("        match (tag_class, tag_number) {\n");
                for alt in alternatives {
                    if let Some(ber) = &alt.ber {
                        let mut variant_name = alt.name.clone();
                        if let Some(first_char) = variant_name.chars().next() {
                            variant_name = first_char.to_uppercase().collect::<String>() + &variant_name[1..];
                        }
                        output.push_str(&format!("            (asnvil_runtime_rust::TagClass::{}, {}) => {{\n", self.tag_class_to_rust(&ber.tag_class), ber.tag_number));
                        output.push_str(&format!("                let mut {} = None;\n", alt.name));
                        for stmt in &alt.decode_stmts {
                            let stmt_str = self.render_decode_stmt(stmt, "decoder", "                ", Some(ber), "Oer")?;
                            output.push_str(&stmt_str);
                            output.push_str("\n");
                        }
                        output.push_str(&format!("                Ok({}::{}({}.unwrap()))\n", name, variant_name, alt.name));
                        output.push_str("            }\n");
                    }
                }
                output.push_str("            _ => Err(AsnError::UnexpectedTag {\n");
                output.push_str("                expected: Tag { tag_class: TagClass::Universal, number: 0, constructed: false },\n");
                output.push_str("                actual: Tag { tag_class, number: tag_number, constructed },\n");
                output.push_str("            }),\n");
                output.push_str("        }\n");
                output.push_str("    }\n");
                output.push_str("}\n");

                Ok(output)
            }
            Declaration::TypeAlias { name, target } => {
                let ty = self.render_type_internal(target)?;
                Ok(format!("pub type {} = {};\n", name, ty))
            }
            Declaration::ListType { name, element_type, ber: _, doc_comment } => {
                let ty = self.render_type_internal(element_type)?;
                let mut output = String::new();
                if let Some(doc) = doc_comment {
                    output.push_str(&format!("/// {}\n", doc));
                }
                output.push_str(&format!("pub type {} = Vec<{}>;\n", name, ty));
                Ok(output)
            }
        }
    }

    fn render_type(&self, ty: &TypeRef) -> Result<String> {
        self.render_type_internal(ty)
    }

    fn runtime_imports(&self) -> Vec<String> {
        vec![
            "use asnvil_runtime_rust::{AsnError, Tag, TagClass, DerEncoder, DerDecoder, BitString, ObjectIdentifier, AsnAny};".to_string(),
            "use num_bigint::BigInt;".to_string(),
        ]
    }
}
