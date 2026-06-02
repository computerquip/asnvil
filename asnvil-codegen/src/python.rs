use crate::code_ast::*;
use crate::renderer::LanguageRenderer;
use anyhow::{bail, Result};
use askama::Template;

#[derive(Template)]
#[template(path = "python/module_header.txt")]
struct ModuleTemplate<'a> {
    declarations: &'a str,
    doc_comment: &'a str,
}

#[derive(Template)]
#[template(path = "python/enum.txt")]
struct EnumTemplate<'a> {
    name: &'a str,
    variants: Vec<VariantContext<'a>>,
    repr: &'a str,
    doc_comment: &'a str,
}

#[derive(Template)]
#[template(path = "python/type_alias.txt")]
struct TypeAliasTemplate<'a> {
    name: &'a str,
    target: String,
}

#[derive(Clone)]
struct VariantContext<'a> {
    name: &'a str,
    value: i64,
}

macro_rules! askama_struct {
    ($name:ident, $path:expr, $($field:ident: $ty:ty),+ $(,)?) => {
        #[derive(Template)]
        #[template(path = $path)]
        struct $name<'a> {
            $($field: $ty),+
        }
    };
}

// ── Encode templates (leaf types only) ────────────────────────────────────────
askama_struct!(EncodeInteger, "python/encode/encode_integer.txt", indent: &'a str, encoder: &'a str, value: &'a str, target: &'a str);
askama_struct!(EncodeEnumerated, "python/encode/encode_enumerated.txt", indent: &'a str, encoder: &'a str, value: &'a str, target: &'a str);
askama_struct!(EncodeBoolean, "python/encode/encode_boolean.txt", indent: &'a str, encoder: &'a str, value: &'a str, target: &'a str);
askama_struct!(EncodeString, "python/encode/encode_string.txt", indent: &'a str, encoder: &'a str, value: &'a str, tag_number: u32, string_encoding: &'a str, target: &'a str);
askama_struct!(EncodeBytes, "python/encode/encode_bytes.txt", indent: &'a str, encoder: &'a str, value: &'a str, tag_number: u32, tlv_method: &'a str, target: &'a str);
askama_struct!(EncodeBitString, "python/encode/encode_bit_string.txt", indent: &'a str, encoder: &'a str, value: &'a str, target: &'a str);
askama_struct!(EncodeOid, "python/encode/encode_oid.txt", indent: &'a str, encoder: &'a str, value: &'a str, target: &'a str);
askama_struct!(EncodeNull, "python/encode/encode_null.txt", indent: &'a str, encoder: &'a str, target: &'a str);
askama_struct!(EncodeReal, "python/encode/encode_real.txt", indent: &'a str, encoder: &'a str, value: &'a str, target: &'a str);
askama_struct!(EncodeTime, "python/encode/encode_time.txt", indent: &'a str, encoder: &'a str, value: &'a str, tag_number: u32, target: &'a str);
askama_struct!(EncodeAny, "python/encode/encode_any.txt", indent: &'a str, value: &'a str, target: &'a str);
askama_struct!(EncodeReferenced, "python/encode/encode_referenced.txt", indent: &'a str, value: &'a str, encode_method: &'a str, target: &'a str);

// ── Decode templates (leaf types only) ────────────────────────────────────────
askama_struct!(DecodeInteger, "python/decode/decode_integer.txt", indent: &'a str, name: &'a str);
askama_struct!(DecodeBoolean, "python/decode/decode_boolean.txt", indent: &'a str, name: &'a str);
askama_struct!(DecodeString, "python/decode/decode_string.txt", indent: &'a str, name: &'a str, encoding: &'a str);
askama_struct!(DecodeBytes, "python/decode/decode_bytes.txt", indent: &'a str, name: &'a str);
askama_struct!(DecodeBitString, "python/decode/decode_bit_string.txt", indent: &'a str, name: &'a str);
askama_struct!(DecodeOid, "python/decode/decode_oid.txt", indent: &'a str, name: &'a str);
askama_struct!(DecodeNull, "python/decode/decode_null.txt", indent: &'a str, name: &'a str);
askama_struct!(DecodeReal, "python/decode/decode_real.txt", indent: &'a str, name: &'a str);
askama_struct!(DecodeTime, "python/decode/decode_time.txt", indent: &'a str, name: &'a str);
askama_struct!(DecodeAny, "python/decode/decode_any.txt", indent: &'a str, name: &'a str);
askama_struct!(DecodeAnyTlv, "python/decode/decode_any_tlv.txt", indent: &'a str, name: &'a str);
askama_struct!(DecodeReferencedTlv, "python/decode/decode_referenced_tlv.txt", indent: &'a str, name: &'a str, inner_type: &'a str, decode_method: &'a str);

fn tpl<T: Template>(t: T) -> String {
    t.render().expect("template render failed").trim_end_matches('\n').to_string()
}

pub struct PythonRenderer;

impl PythonRenderer {
    pub fn new() -> Self {
        Self
    }

    fn value_literal_to_python(v: &ValueLiteral) -> String {
        match v {
            ValueLiteral::Int(n) => n.to_string(),
            ValueLiteral::Bool(true) => "True".to_string(),
            ValueLiteral::Bool(false) => "False".to_string(),
            ValueLiteral::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            ValueLiteral::Bytes(b) => format!("b'{}'", b.iter().map(|byte| format!("\\x{:02x}", byte)).collect::<String>()),
            ValueLiteral::None => "None".to_string(),
            ValueLiteral::Any => "None".to_string(),
        }
    }
}

impl Default for PythonRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageRenderer for PythonRenderer {
    fn language_name(&self) -> &str {
        "python"
    }

    fn render_module(&self, ast: &CodeAstNode) -> Result<String> {
        match ast {
            CodeAstNode::Module {
                name: _,
                imports: _,
                declarations,
                doc_comment,
            } => {
                let decls: Vec<String> = declarations
                    .iter()
                    .map(|d| self.render_declaration(d).unwrap())
                    .collect();
                let decls_str = decls.join("\n\n");
                let tmpl = ModuleTemplate {
                    declarations: &decls_str,
                    doc_comment: doc_comment.as_deref().unwrap_or(""),
                };
                tmpl.render().map_err(|e| anyhow::anyhow!("Failed to render module template: {}", e))
            }
            _ => bail!("Expected Module node"),
        }
    }

    fn render_declaration(&self, decl: &Declaration) -> Result<String> {
        match decl {
            Declaration::Struct { name, fields, doc_comment, annotations } => {
                self.render_struct(name, fields, annotations, doc_comment.as_deref().unwrap_or(""))
            }
            Declaration::Enum { name, variants, repr, doc_comment } => {
                let mut next_value = 0i64;
                let rendered_variants: Vec<_> = variants.iter().map(|v| {
                    let value = v.value.unwrap_or(next_value);
                    next_value = value + 1;
                    VariantContext {
                        name: &v.name,
                        value,
                    }
                }).collect();
                let repr_str = match repr {
                    Some(crate::code_ast::EnumRepr::Int) => "IntEnum",
                    None => "Enum",
                };
                let tmpl = EnumTemplate {
                    name,
                    variants: rendered_variants,
                    repr: &repr_str,
                    doc_comment: doc_comment.as_deref().unwrap_or(""),
                };
                tmpl.render().map_err(|e| anyhow::anyhow!("Failed to render enum template: {}", e))
            }
            Declaration::Choice { name, alternatives, doc_comment } => {
                self.render_choice(name, alternatives, doc_comment.as_deref().unwrap_or(""))
            }
            Declaration::TypeAlias { name, target } => {
                let tmpl = TypeAliasTemplate {
                    name,
                    target: self.render_type(target)?,
                };
                tmpl.render().map_err(|e| anyhow::anyhow!("Failed to render type alias template: {}", e))
            }
            Declaration::ListType { name, element_type, ber, doc_comment } => {
                let ty_str = self.render_type(element_type)?;
                self.render_list_type(name, &ty_str, ber, doc_comment.as_deref().unwrap_or(""))
            }
            Declaration::Constant { .. } => {
                bail!("Constants not yet supported in Python renderer")
            }
            Declaration::FunctionDecl(func) => {
                self.render_function(func, "function")
            }
        }
    }

    fn render_type(&self, ty: &TypeRef) -> Result<String> {
        match ty {
            TypeRef::Builtin(builtin) => match builtin {
                BuiltinType::Integer => Ok("int".to_string()),
                BuiltinType::Boolean => Ok("bool".to_string()),
                BuiltinType::String(_) => Ok("str".to_string()),
                BuiltinType::OctetString => Ok("bytes".to_string()),
                BuiltinType::BitString => Ok("BitString".to_string()),
                BuiltinType::ObjectIdentifier => Ok("ObjectIdentifier".to_string()),
                BuiltinType::Null => Ok("None".to_string()),
                BuiltinType::Real => Ok("float".to_string()),
                BuiltinType::GeneralizedTime | BuiltinType::UTCTime => Ok("datetime".to_string()),
                BuiltinType::Any => Ok("Any".to_string()),
            },
            TypeRef::Named(name) => Ok(name.clone()),
            TypeRef::Optional(inner) => {
                let inner_type = self.render_type(inner)?;
                Ok(format!("Optional[{}]", inner_type))
            }
            TypeRef::List(inner) => {
                let inner_type = self.render_type(inner)?;
                Ok(format!("list[{}]", inner_type))
            }
        }
    }

    fn render_function(&self, _func: &Function, _template: &str) -> Result<String> {
        bail!("Function rendering not yet implemented for Askama")
    }

    fn runtime_imports(&self) -> Vec<String> {
        vec![
            "from asnvil_runtime import AsnType, Tag, TagClass, BerEncoder, BerDecoder, DerEncoder, DerDecoder, AsnError".to_string(),
            "from dataclasses import dataclass, field".to_string(),
            "from typing import Optional".to_string(),
        ]
    }
}

impl PythonRenderer {
    // ─── Encode statement rendering ────────────────────────────────────────────

    fn render_encode_field(&self, stmt: &EncodeStmt, encoder: &str, tlv_method: &str, indent: &str) -> String {
        match stmt {
            EncodeStmt::WriteInteger { value, .. } => tpl(EncodeInteger { indent, encoder, value, target: "content" }),
            EncodeStmt::WriteEnumerated { value, .. } => tpl(EncodeEnumerated { indent, encoder, value, target: "content" }),
            EncodeStmt::WriteBoolean { value, .. } => tpl(EncodeBoolean { indent, encoder, value, target: "content" }),
            EncodeStmt::WriteString { tag, value, encoding, .. } => tpl(EncodeString { indent, encoder, value, tag_number: tag.number, string_encoding: encoding, target: "content" }),
            EncodeStmt::WriteBytes { tag, value, .. } => tpl(EncodeBytes { indent, encoder, value, tag_number: tag.number, tlv_method, target: "content" }),
            EncodeStmt::WriteBitString { value, .. } => tpl(EncodeBitString { indent, encoder, value, target: "content" }),
            EncodeStmt::WriteOid { value, .. } => tpl(EncodeOid { indent, encoder, value, target: "content" }),
            EncodeStmt::WriteNull { .. } => tpl(EncodeNull { indent, encoder, target: "content" }),
            EncodeStmt::WriteReal { value, .. } => tpl(EncodeReal { indent, encoder, value, target: "content" }),
            EncodeStmt::WriteTime { tag, value, .. } => tpl(EncodeTime { indent, encoder, value, tag_number: tag.number, target: "content" }),
            EncodeStmt::WriteAny { value, .. } => tpl(EncodeAny { indent, value, target: "content" }),
            EncodeStmt::WriteReferenced { encode_method, value, .. }
            | EncodeStmt::WriteChoice { encode_method, value, .. } => tpl(EncodeReferenced { indent, value, encode_method, target: "content" }),
            EncodeStmt::WriteList { tag, value, element_info, .. } => {
                let element_encode = match element_info.encoding.as_str() {
                    "constructed" | "referenced" | "choice" | "list" => {
                        format!("{indent}    _lc.extend(_li.encode_der())")
                    }
                    "integer" => format!(
                        "{indent}    _lie = {encoder}()\n\
                         {indent}    _lie.write_integer(_li)\n\
                         {indent}    _lib = _lie.finish()\n\
                         {indent}    _lite = {encoder}()\n\
                         {indent}    _lite.write_tag(TagClass.UNIVERSAL, 2, False)\n\
                         {indent}    _lite.write_length(len(_lib))\n\
                         {indent}    _lite.write_bytes(_lib)\n\
                         {indent}    _lc.extend(_lite.finish())",
                        indent = indent, encoder = encoder
                    ),
                    "enumerated" => format!(
                        "{indent}    _lie = {encoder}()\n\
                         {indent}    _lie.write_integer(_li.value if hasattr(_li, 'value') else _li)\n\
                         {indent}    _lib = _lie.finish()\n\
                         {indent}    _lite = {encoder}()\n\
                         {indent}    _lite.write_tag(TagClass.UNIVERSAL, 10, False)\n\
                         {indent}    _lite.write_length(len(_lib))\n\
                         {indent}    _lite.write_bytes(_lib)\n\
                         {indent}    _lc.extend(_lite.finish())",
                        indent = indent, encoder = encoder
                    ),
                    "boolean" => format!(
                        "{indent}    _be = {encoder}()\n\
                         {indent}    _be.write_tag(TagClass.UNIVERSAL, 1, False)\n\
                         {indent}    _be.write_length(1)\n\
                         {indent}    _be.write_bytes(b'\\xff' if _li else b'\\x00')\n\
                         {indent}    _lc.extend(_be.finish())",
                        indent = indent, encoder = encoder
                    ),
                    "string" => format!(
                        "{indent}    _sb = _li.encode('{string_encoding}')\n\
                         {indent}    _se = {encoder}()\n\
                         {indent}    _se.write_tag(TagClass.UNIVERSAL, {tag_number}, False)\n\
                         {indent}    _se.write_length(len(_sb))\n\
                         {indent}    _se.write_bytes(_sb)\n\
                         {indent}    _lc.extend(_se.finish())",
                        indent = indent, encoder = encoder,
                        string_encoding = element_info.string_encoding,
                        tag_number = element_info.tag_number
                    ),
                    "bytes" => format!(
                        "{indent}    _be = {encoder}()\n\
                         {indent}    _be.{tlv_method}(TagClass.UNIVERSAL, 4, _li)\n\
                         {indent}    _lc.extend(_be.finish())",
                        indent = indent, encoder = encoder, tlv_method = tlv_method
                    ),
                    "bit_string" => format!(
                        "{indent}    _be = {encoder}()\n\
                         {indent}    _be.write_tag(TagClass.UNIVERSAL, 3, False)\n\
                         {indent}    _be.write_length(len(_li.data) + 1)\n\
                         {indent}    _be.write_bytes(bytes([_li.unused_bits]))\n\
                         {indent}    _be.write_bytes(_li.data)\n\
                         {indent}    _lc.extend(_be.finish())",
                        indent = indent, encoder = encoder
                    ),
                    "oid" => format!(
                        "{indent}    _ob = _li.encode()\n\
                         {indent}    _oe = {encoder}()\n\
                         {indent}    _oe.write_tag(TagClass.UNIVERSAL, 6, False)\n\
                         {indent}    _oe.write_length(len(_ob))\n\
                         {indent}    _oe.write_bytes(_ob)\n\
                         {indent}    _lc.extend(_oe.finish())",
                        indent = indent, encoder = encoder
                    ),
                    _ => format!("{indent}    if hasattr(_li, 'encode_der'):\n{indent}        _lc.extend(_li.encode_der())", indent = indent),
                };
                format!(
                    "{indent}_le = {encoder}()\n\
                     {indent}_lc = bytearray()\n\
                     {indent}for _li in {value}:\n\
                     {element_encode}\n\
                     {indent}_le.write_tag(TagClass.UNIVERSAL, {tag_num}, True)\n\
                     {indent}_le.write_length(len(_lc))\n\
                     {indent}_le.write_bytes(_lc)\n\
                     {indent}content.extend(_le.finish())",
                    indent = indent, encoder = encoder, value = value, tag_num = tag.number,
                    element_encode = element_encode
                )
            }
            EncodeStmt::WrapExplicit { .. } | EncodeStmt::WrapImplicit { .. } => String::new(),
        }
    }

    fn render_choice_encode_alt(&self, alt: &ChoiceAlternative, encoder: &str, tlv_method: &str, is_indefinite: bool) -> String {
        let ber = match &alt.ber {
            Some(b) => b,
            None => return String::new(),
        };
        let value = format!("self.{}", alt.name);
        let outer_tag_class = &ber.tag_class;
        let outer_tag_number = ber.tag_number;
        let inherent_tag_number = ber.inherent_tag_number;
        let encoding = &ber.encoding;
        let string_encoding = &ber.string_encoding;
        let tagging_mode = &ber.tagging_mode;

        let indent = "        ";
        let inner_indent = "            ";

        match tagging_mode.as_str() {
            "explicit" => {
                match encoding.as_str() {
                    "integer" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_iv = {encoder}()\n\
                         {inner_indent}_iv.write_integer({value})\n\
                         {inner_indent}_ib = _iv.finish()\n\
                         {inner_indent}_ie = {encoder}()\n\
                         {inner_indent}_ie.write_tag(TagClass.UNIVERSAL, 2, False)\n\
                         {inner_indent}_ie.write_length(len(_ib))\n\
                         {inner_indent}_ie.write_bytes(_ib)\n\
                         {inner_indent}_inner = _ie.finish()\n\
                         {inner_indent}_e = {encoder}()\n\
                         {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, True)\n\
                         {inner_indent}_e.write_length(len(_inner))\n\
                         {inner_indent}_e.write_bytes(_inner)\n\
                         {inner_indent}return _e.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "enumerated" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_iv = {encoder}()\n\
                         {inner_indent}_iv.write_integer({value}.value if hasattr({value}, 'value') else {value})\n\
                         {inner_indent}_ib = _iv.finish()\n\
                         {inner_indent}_ie = {encoder}()\n\
                         {inner_indent}_ie.write_tag(TagClass.UNIVERSAL, 10, False)\n\
                         {inner_indent}_ie.write_length(len(_ib))\n\
                         {inner_indent}_ie.write_bytes(_ib)\n\
                         {inner_indent}_inner = _ie.finish()\n\
                         {inner_indent}_e = {encoder}()\n\
                         {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, True)\n\
                         {inner_indent}_e.write_length(len(_inner))\n\
                         {inner_indent}_e.write_bytes(_inner)\n\
                         {inner_indent}return _e.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "boolean" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_be = {encoder}()\n\
                         {inner_indent}_be.write_tag(TagClass.UNIVERSAL, 1, False)\n\
                         {inner_indent}_be.write_length(1)\n\
                         {inner_indent}_be.write_bytes(b'\\xff' if {value} else b'\\x00')\n\
                         {inner_indent}_inner = _be.finish()\n\
                         {inner_indent}_e = {encoder}()\n\
                         {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, True)\n\
                         {inner_indent}_e.write_length(len(_inner))\n\
                         {inner_indent}_e.write_bytes(_inner)\n\
                         {inner_indent}return _e.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "string" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_se = {encoder}()\n\
                         {inner_indent}_sb = {value}.encode('{string_encoding}')\n\
                         {inner_indent}_se.write_tag(TagClass.UNIVERSAL, {inherent_tag_number}, False)\n\
                         {inner_indent}_se.write_length(len(_sb))\n\
                         {inner_indent}_se.write_bytes(_sb)\n\
                         {inner_indent}_inner = _se.finish()\n\
                         {inner_indent}_e = {encoder}()\n\
                         {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, True)\n\
                         {inner_indent}_e.write_length(len(_inner))\n\
                         {inner_indent}_e.write_bytes(_inner)\n\
                         {inner_indent}return _e.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        string_encoding = string_encoding, inherent_tag_number = inherent_tag_number,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "bytes" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_be = {encoder}()\n\
                         {inner_indent}_be.{tlv_method}(TagClass.UNIVERSAL, {inherent_tag_number}, {value})\n\
                         {inner_indent}_inner = _be.finish()\n\
                         {inner_indent}_e = {encoder}()\n\
                         {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, True)\n\
                         {inner_indent}_e.write_length(len(_inner))\n\
                         {inner_indent}_e.write_bytes(_inner)\n\
                         {inner_indent}return _e.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        tlv_method = tlv_method, inherent_tag_number = inherent_tag_number,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "bit_string" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_be = {encoder}()\n\
                         {inner_indent}_be.write_tag(TagClass.UNIVERSAL, 3, False)\n\
                         {inner_indent}_be.write_length(len({value}.data) + 1)\n\
                         {inner_indent}_be.write_bytes(bytes([{value}.unused_bits]))\n\
                         {inner_indent}_be.write_bytes({value}.data)\n\
                         {inner_indent}_inner = _be.finish()\n\
                         {inner_indent}_e = {encoder}()\n\
                         {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, True)\n\
                         {inner_indent}_e.write_length(len(_inner))\n\
                         {inner_indent}_e.write_bytes(_inner)\n\
                         {inner_indent}return _e.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "oid" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_oe = {encoder}()\n\
                         {inner_indent}_ob = {value}.encode()\n\
                         {inner_indent}_oe.write_tag(TagClass.UNIVERSAL, 6, False)\n\
                         {inner_indent}_oe.write_length(len(_ob))\n\
                         {inner_indent}_oe.write_bytes(_ob)\n\
                         {inner_indent}_inner = _oe.finish()\n\
                         {inner_indent}_e = {encoder}()\n\
                         {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, True)\n\
                         {inner_indent}_e.write_length(len(_inner))\n\
                         {inner_indent}_e.write_bytes(_inner)\n\
                         {inner_indent}return _e.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "null" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_ne = {encoder}()\n\
                         {inner_indent}_ne.write_tag(TagClass.UNIVERSAL, 5, False)\n\
                         {inner_indent}_ne.write_length(0)\n\
                         {inner_indent}_inner = _ne.finish()\n\
                         {inner_indent}_e = {encoder}()\n\
                         {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, True)\n\
                         {inner_indent}_e.write_length(len(_inner))\n\
                         {inner_indent}_e.write_bytes(_inner)\n\
                         {inner_indent}return _e.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "real" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}import struct\n\
                         {inner_indent}_re = {encoder}()\n\
                         {inner_indent}_rb = struct.pack('>d', {value})\n\
                         {inner_indent}_re.write_tag(TagClass.UNIVERSAL, 9, False)\n\
                         {inner_indent}_re.write_length(len(_rb))\n\
                         {inner_indent}_re.write_bytes(_rb)\n\
                         {inner_indent}_inner = _re.finish()\n\
                         {inner_indent}_e = {encoder}()\n\
                         {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, True)\n\
                         {inner_indent}_e.write_length(len(_inner))\n\
                         {inner_indent}_e.write_bytes(_inner)\n\
                         {inner_indent}return _e.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "time" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_te = {encoder}()\n\
                         {inner_indent}_tb = {value}.strftime('%Y%m%d%H%M%SZ').encode('ascii')\n\
                         {inner_indent}_te.write_tag(TagClass.UNIVERSAL, {inherent_tag_number}, False)\n\
                         {inner_indent}_te.write_length(len(_tb))\n\
                         {inner_indent}_te.write_bytes(_tb)\n\
                         {inner_indent}_inner = _te.finish()\n\
                         {inner_indent}_e = {encoder}()\n\
                         {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, True)\n\
                         {inner_indent}_e.write_length(len(_inner))\n\
                         {inner_indent}_e.write_bytes(_inner)\n\
                         {inner_indent}return _e.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        inherent_tag_number = inherent_tag_number,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "referenced" | "constructed" | "choice" => {
                        if is_indefinite {
                            let encode_method = if encoding.as_str() == "choice" { "encode_ber_indefinite" } else { "encode_ber" };
                            format!(
                                "{indent}if {value} is not None:\n\
                                 {inner_indent}_inner = {value}.{encode_method}()\n\
                                 {inner_indent}_e = {encoder}()\n\
                                 {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, True)\n\
                                 {inner_indent}_e.write_length(0, indefinite=True)\n\
                                 {inner_indent}_e.write_bytes(_inner)\n\
                                 {inner_indent}_e.write_eoc()\n\
                                 {inner_indent}return _e.finish()",
                                indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                                encode_method = encode_method,
                                outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                            )
                        } else {
                            let encode_method = "encode_der";
                            format!(
                                "{indent}if {value} is not None:\n\
                                 {inner_indent}_inner = {value}.{encode_method}()\n\
                                 {inner_indent}_e = {encoder}()\n\
                                 {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, True)\n\
                                 {inner_indent}_e.write_length(len(_inner))\n\
                                 {inner_indent}_e.write_bytes(_inner)\n\
                                 {inner_indent}return _e.finish()",
                                indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                                encode_method = encode_method,
                                outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                            )
                        }
                    }
                    _ => String::new(),
                }
            }
            "implicit" => {
                match encoding.as_str() {
                    "integer" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_iv = {encoder}()\n\
                         {inner_indent}_iv.write_integer({value})\n\
                         {inner_indent}_ib = _iv.finish()\n\
                         {inner_indent}_e = {encoder}()\n\
                         {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, False)\n\
                         {inner_indent}_e.write_length(len(_ib))\n\
                         {inner_indent}_e.write_bytes(_ib)\n\
                         {inner_indent}return _e.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "enumerated" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_iv = {encoder}()\n\
                         {inner_indent}_iv.write_integer({value}.value if hasattr({value}, 'value') else {value})\n\
                         {inner_indent}_ib = _iv.finish()\n\
                         {inner_indent}_e = {encoder}()\n\
                         {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, False)\n\
                         {inner_indent}_e.write_length(len(_ib))\n\
                         {inner_indent}_e.write_bytes(_ib)\n\
                         {inner_indent}return _e.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "boolean" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_be = {encoder}()\n\
                         {inner_indent}_be.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, False)\n\
                         {inner_indent}_be.write_length(1)\n\
                         {inner_indent}_be.write_bytes(b'\\xff' if {value} else b'\\x00')\n\
                         {inner_indent}return _be.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "string" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_sb = {value}.encode('{string_encoding}')\n\
                         {inner_indent}_se = {encoder}()\n\
                         {inner_indent}_se.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, False)\n\
                         {inner_indent}_se.write_length(len(_sb))\n\
                         {inner_indent}_se.write_bytes(_sb)\n\
                         {inner_indent}return _se.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        string_encoding = string_encoding,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "bytes" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_be = {encoder}()\n\
                         {inner_indent}_be.{tlv_method}(TagClass.{outer_tag_class}, {outer_tag_number}, {value})\n\
                         {inner_indent}return _be.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        tlv_method = tlv_method,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "bit_string" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_be = {encoder}()\n\
                         {inner_indent}_be.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, False)\n\
                         {inner_indent}_be.write_length(len({value}.data) + 1)\n\
                         {inner_indent}_be.write_bytes(bytes([{value}.unused_bits]))\n\
                         {inner_indent}_be.write_bytes({value}.data)\n\
                         {inner_indent}return _be.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "oid" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_ob = {value}.encode()\n\
                         {inner_indent}_oe = {encoder}()\n\
                         {inner_indent}_oe.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, False)\n\
                         {inner_indent}_oe.write_length(len(_ob))\n\
                         {inner_indent}_oe.write_bytes(_ob)\n\
                         {inner_indent}return _oe.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "null" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_ne = {encoder}()\n\
                         {inner_indent}_ne.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, False)\n\
                         {inner_indent}_ne.write_length(0)\n\
                         {inner_indent}return _ne.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "real" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}import struct\n\
                         {inner_indent}_rb = struct.pack('>d', {value})\n\
                         {inner_indent}_re = {encoder}()\n\
                         {inner_indent}_re.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, False)\n\
                         {inner_indent}_re.write_length(len(_rb))\n\
                         {inner_indent}_re.write_bytes(_rb)\n\
                         {inner_indent}return _re.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "time" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_tb = {value}.strftime('%Y%m%d%H%M%SZ').encode('ascii')\n\
                         {inner_indent}_te = {encoder}()\n\
                         {inner_indent}_te.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, False)\n\
                         {inner_indent}_te.write_length(len(_tb))\n\
                         {inner_indent}_te.write_bytes(_tb)\n\
                         {inner_indent}return _te.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                    ),
                    "referenced" | "constructed" | "choice" => {
                        let constructed = ber.constructed;
                        if is_indefinite {
                            format!(
                                "{indent}if {value} is not None:\n\
                                 {inner_indent}_inner = {value}.encode_ber()\n\
                                 {inner_indent}_dec = BerDecoder(_inner)\n\
                                 {inner_indent}_dec.read_tag()\n\
                                 {inner_indent}_val_len = _dec.read_length()\n\
                                 {inner_indent}_val_bytes = _dec.read_bytes(_val_len)\n\
                                 {inner_indent}_e = {encoder}()\n\
                                 {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, {constructed})\n\
                                 {inner_indent}_e.write_length(0, indefinite=True)\n\
                                 {inner_indent}_e.write_bytes(_val_bytes)\n\
                                 {inner_indent}_e.write_eoc()\n\
                                 {inner_indent}return _e.finish()",
                                indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                                constructed = constructed,
                                outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                            )
                        } else {
                            format!(
                                "{indent}if {value} is not None:\n\
                                 {inner_indent}_inner = {value}.encode_der()\n\
                                 {inner_indent}_dec = BerDecoder(_inner)\n\
                                 {inner_indent}_dec.read_tag()\n\
                                 {inner_indent}_val_len = _dec.read_length()\n\
                                 {inner_indent}_val_bytes = _dec.read_bytes(_val_len)\n\
                                 {inner_indent}_e = {encoder}()\n\
                                 {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, {constructed})\n\
                                 {inner_indent}_e.write_length(_val_len)\n\
                                 {inner_indent}_e.write_bytes(_val_bytes)\n\
                                 {inner_indent}return _e.finish()",
                                indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                                constructed = constructed,
                                outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                            )
                        }
                    }
                    _ => String::new(),
                }
            }
            _ => {
                match encoding.as_str() {
                    "integer" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_iv = {encoder}()\n\
                         {inner_indent}_iv.write_integer({value})\n\
                         {inner_indent}_ib = _iv.finish()\n\
                         {inner_indent}_ie = {encoder}()\n\
                         {inner_indent}_ie.write_tag(TagClass.UNIVERSAL, 2, False)\n\
                         {inner_indent}_ie.write_length(len(_ib))\n\
                         {inner_indent}_ie.write_bytes(_ib)\n\
                         {inner_indent}return _ie.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value
                    ),
                    "enumerated" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_iv = {encoder}()\n\
                         {inner_indent}_iv.write_integer({value}.value if hasattr({value}, 'value') else {value})\n\
                         {inner_indent}_ib = _iv.finish()\n\
                         {inner_indent}_ie = {encoder}()\n\
                         {inner_indent}_ie.write_tag(TagClass.UNIVERSAL, 10, False)\n\
                         {inner_indent}_ie.write_length(len(_ib))\n\
                         {inner_indent}_ie.write_bytes(_ib)\n\
                         {inner_indent}return _ie.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value
                    ),
                    "boolean" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_be = {encoder}()\n\
                         {inner_indent}_be.write_tag(TagClass.UNIVERSAL, 1, False)\n\
                         {inner_indent}_be.write_length(1)\n\
                         {inner_indent}_be.write_bytes(b'\\xff' if {value} else b'\\x00')\n\
                         {inner_indent}return _be.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value
                    ),
                    "string" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_se = {encoder}()\n\
                         {inner_indent}_sb = {value}.encode('{string_encoding}')\n\
                         {inner_indent}_se.write_tag(TagClass.UNIVERSAL, {tag_num}, False)\n\
                         {inner_indent}_se.write_length(len(_sb))\n\
                         {inner_indent}_se.write_bytes(_sb)\n\
                         {inner_indent}return _se.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        string_encoding = string_encoding, tag_num = ber.tag_number
                    ),
                    "bytes" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_be = {encoder}()\n\
                         {inner_indent}_be.{tlv_method}(TagClass.UNIVERSAL, {tag_num}, {value})\n\
                         {inner_indent}return _be.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        tlv_method = tlv_method, tag_num = ber.tag_number
                    ),
                    "bit_string" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_be = {encoder}()\n\
                         {inner_indent}_be.write_tag(TagClass.UNIVERSAL, 3, False)\n\
                         {inner_indent}_be.write_length(len({value}.data) + 1)\n\
                         {inner_indent}_be.write_bytes(bytes([{value}.unused_bits]))\n\
                         {inner_indent}_be.write_bytes({value}.data)\n\
                         {inner_indent}return _be.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value
                    ),
                    "oid" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_oe = {encoder}()\n\
                         {inner_indent}_ob = {value}.encode()\n\
                         {inner_indent}_oe.write_tag(TagClass.UNIVERSAL, 6, False)\n\
                         {inner_indent}_oe.write_length(len(_ob))\n\
                         {inner_indent}_oe.write_bytes(_ob)\n\
                         {inner_indent}return _oe.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value
                    ),
                    "null" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_ne = {encoder}()\n\
                         {inner_indent}_ne.write_tag(TagClass.UNIVERSAL, 5, False)\n\
                         {inner_indent}_ne.write_length(0)\n\
                         {inner_indent}return _ne.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value
                    ),
                    "real" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}import struct\n\
                         {inner_indent}_re = {encoder}()\n\
                         {inner_indent}_rb = struct.pack('>d', {value})\n\
                         {inner_indent}_re.write_tag(TagClass.UNIVERSAL, 9, False)\n\
                         {inner_indent}_re.write_length(len(_rb))\n\
                         {inner_indent}_re.write_bytes(_rb)\n\
                         {inner_indent}return _re.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value
                    ),
                    "time" => format!(
                        "{indent}if {value} is not None:\n\
                         {inner_indent}_te = {encoder}()\n\
                         {inner_indent}_tb = {value}.strftime('%Y%m%d%H%M%SZ').encode('ascii')\n\
                         {inner_indent}_te.write_tag(TagClass.UNIVERSAL, {tag_num}, False)\n\
                         {inner_indent}_te.write_length(len(_tb))\n\
                         {inner_indent}_te.write_bytes(_tb)\n\
                         {inner_indent}return _te.finish()",
                        indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                        tag_num = ber.tag_number
                    ),
                    "referenced" | "constructed" | "choice" => {
                        if is_indefinite {
                            let encode_method = if encoding.as_str() == "choice" { "encode_ber_indefinite" } else { "encode_ber" };
                            format!(
                                "{indent}if {value} is not None:\n\
                                 {inner_indent}_inner = {value}.{encode_method}()\n\
                                 {inner_indent}_e = {encoder}()\n\
                                 {inner_indent}_e.write_tag(TagClass.{outer_tag_class}, {outer_tag_number}, True)\n\
                                 {inner_indent}_e.write_length(0, indefinite=True)\n\
                                 {inner_indent}_e.write_bytes(_inner)\n\
                                 {inner_indent}_e.write_eoc()\n\
                                 {inner_indent}return _e.finish()",
                                indent = indent, inner_indent = inner_indent, encoder = encoder, value = value,
                                encode_method = encode_method,
                                outer_tag_class = outer_tag_class, outer_tag_number = outer_tag_number
                            )
                        } else {
                            format!(
                                "{indent}if {value} is not None:\n\
                                 {inner_indent}return {value}.encode_der()",
                                indent = indent, inner_indent = inner_indent, value = value
                            )
                        }
                    }
                    _ => String::new(),
                }
            }
        }
    }

    fn render_struct(&self, name: &str, fields: &[Field], annotations: &[String], doc_comment: &str) -> Result<String> {
        let mut out = String::new();
        out.push_str("@dataclass\n");
        out.push_str(&format!("class {}(AsnType):\n", name));
        if !doc_comment.is_empty() {
            out.push_str(&format!("    \"\"\"{}\"\"\"\n", doc_comment));
        }
        let sorted_fields = {
            let mut f: Vec<_> = fields.iter().collect();
            f.sort_by_key(|f| f.order);
            f
        };
        for f in &sorted_fields {
            let ty_str = self.render_type(&f.ty).unwrap_or_else(|_| "Any".to_string());
            let default_str = f.default.as_ref()
                .map(Self::value_literal_to_python)
                .unwrap_or_default();
            if !default_str.is_empty() {
                out.push_str(&format!("    {}: {} = {}\n", f.name, ty_str, default_str));
            } else if f.optional {
                out.push_str(&format!("    {}: {} = None\n", f.name, ty_str));
            } else {
                out.push_str(&format!("    {}: {}\n", f.name, ty_str));
            }
        }
        let outer_tag = if annotations.iter().any(|a| a == "set") { 17 } else { 16 };
        out.push_str("\n");
        out.push_str("    def encode_ber(self) -> bytes:\n");
        out.push_str("        content = bytearray()\n");
        for f in &sorted_fields {
            if f.ber.is_none() { continue; }
            let is_optional = f.optional || f.default.is_some();
            if is_optional {
                if let Some(ref default) = f.default {
                    let default_python = Self::value_literal_to_python(default);
                    out.push_str(&format!("        if self.{} is not None and self.{} != {}:\n", f.name, f.name, default_python));
                } else {
                    out.push_str(&format!("        if self.{} is not None:\n", f.name));
                }
                for stmt in &f.encode_stmts {
                    out.push_str(&self.render_encode_field(stmt, "BerEncoder", "write_tlv", "            "));
                    out.push('\n');
                }
            } else {
                for stmt in &f.encode_stmts {
                    out.push_str(&self.render_encode_field(stmt, "BerEncoder", "write_tlv", "        "));
                    out.push('\n');
                }
            }
        }
        out.push_str("        return bytes(content)\n");
        out.push_str("\n");
        out.push_str("    def encode_ber_indefinite(self) -> bytes:\n");
        out.push_str("        content = bytearray()\n");
        for f in &sorted_fields {
            if f.ber.is_none() { continue; }
            let is_optional = f.optional || f.default.is_some();
            if is_optional {
                if let Some(ref default) = f.default {
                    let default_python = Self::value_literal_to_python(default);
                    out.push_str(&format!("        if self.{} is not None and self.{} != {}:\n", f.name, f.name, default_python));
                } else {
                    out.push_str(&format!("        if self.{} is not None:\n", f.name));
                }
                for stmt in &f.encode_stmts {
                    out.push_str(&self.render_encode_field(stmt, "BerEncoder", "write_tlv", "            "));
                    out.push('\n');
                }
            } else {
                for stmt in &f.encode_stmts {
                    out.push_str(&self.render_encode_field(stmt, "BerEncoder", "write_tlv", "        "));
                    out.push('\n');
                }
            }
        }
        out.push_str("        _outer = BerEncoder()\n");
        out.push_str(&format!("        _outer.write_tag(TagClass.UNIVERSAL, {}, True)\n", outer_tag));
        out.push_str("        _outer.write_length(0, indefinite=True)\n");
        out.push_str("        _outer.write_bytes(content)\n");
        out.push_str("        _outer.write_eoc()\n");
        out.push_str("        return _outer.finish()\n");
        out.push_str("\n");
        out.push_str("    def encode_der(self) -> bytes:\n");
        out.push_str("        content = bytearray()\n");
        for f in &sorted_fields {
            if f.ber.is_none() { continue; }
            let is_optional = f.optional || f.default.is_some();
            if is_optional {
                if let Some(ref default) = f.default {
                    let default_python = Self::value_literal_to_python(default);
                    out.push_str(&format!("        if self.{} is not None and self.{} != {}:\n", f.name, f.name, default_python));
                } else {
                    out.push_str(&format!("        if self.{} is not None:\n", f.name));
                }
                for stmt in &f.encode_stmts {
                    out.push_str(&self.render_encode_field(stmt, "DerEncoder", "write_tlv_der", "            "));
                    out.push('\n');
                }
            } else {
                for stmt in &f.encode_stmts {
                    out.push_str(&self.render_encode_field(stmt, "DerEncoder", "write_tlv_der", "        "));
                    out.push('\n');
                }
            }
        }
        out.push_str("        _outer = DerEncoder()\n");
        out.push_str(&format!("        _outer.write_tag(TagClass.UNIVERSAL, {}, True)\n", outer_tag));
        out.push_str("        _outer.write_length(len(content))\n");
        out.push_str("        _outer.write_bytes(content)\n");
        out.push_str("        return _outer.finish()\n");
        out.push_str("\n");
        out.push_str("    @classmethod\n");
        out.push_str(&format!("    def decode_der(cls, data: bytes) -> \"{}\":\n", name));
        if annotations.iter().any(|a| a == "set") {
            out.push_str("        decoder = DerDecoder(data)\n");
            out.push_str("        _tag = decoder.read_tag()\n");
            out.push_str("        _length = decoder.read_length()\n");
            out.push_str("        # Read and validate SET elements are in canonical order\n");
            out.push_str("        _set_elements = decoder.read_set_elements(_tag[0], _tag[1], _length)\n");
            out.push_str("        # Decode each element in sorted order\n");
            out.push_str("        # Reconstruct the sorted content and decode normally\n");
            out.push_str("        _sorted_content = DerDecoder.sort_set_tlv(_set_elements)\n");
            out.push_str("        decoder2 = DerDecoder(_sorted_content)\n");
            for f in &sorted_fields {
                if f.ber.is_none() { continue; }
                let ber = f.ber.as_ref().unwrap();
                if f.optional {
                    if let Some(ref default) = f.default {
                        let default_python = Self::value_literal_to_python(default);
                        out.push_str(&format!("        _{} = {}\n", f.name, default_python));
                    } else {
                        out.push_str(&format!("        _{} = None\n", f.name));
                    }
                    out.push_str("        if decoder2._pos < len(_sorted_content):\n");
                    out.push_str(&format!("            _{}_save = decoder2._pos\n", f.name));
                    out.push_str("            _ft = decoder2.read_tag()\n");
                    if !ber.choice_alternative_tags.is_empty() {
                        let tag_checks: Vec<String> = ber.choice_alternative_tags.iter().map(|t| {
                            let constructed_check = if t.constructed { " and _ft[2]" } else { " and not _ft[2]" };
                            format!("(_ft[0] == TagClass.{} and _ft[1] == {}{})", t.tag_class, t.tag_number, constructed_check)
                        }).collect();
                        out.push_str(&format!("            if ({}):\n", tag_checks.join(" or ")));
                    } else {
                        out.push_str(&format!("            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == {} and not _ft[2]:\n", ber.tag_number));
                    }
                    out.push_str("                _fl = decoder2.read_length()\n");
                    out.push_str("                _fd = decoder2.read_bytes(_fl)\n");
                    for stmt in &f.decode_stmts {
                        let code = self.render_decode_field(stmt, "DerDecoder", "                ");
                        out.push_str(&code);
                        out.push('\n');
                    }
                    out.push_str("            else:\n");
                    out.push_str(&format!("                decoder2._pos = _{}_save\n", f.name));
                    if f.default.is_some() {
                        let default_python = Self::value_literal_to_python(f.default.as_ref().unwrap());
                        out.push_str(&format!("                _{} = {}\n", f.name, default_python));
                    }
                } else {
                    out.push_str(&format!("        _ft = decoder2.read_tag()\n"));
                    out.push_str(&format!("        _fl = decoder2.read_length()\n"));
                    out.push_str(&format!("        _fd = decoder2.read_bytes(_fl)\n"));
                    for stmt in &f.decode_stmts {
                        let code = self.render_decode_field(stmt, "DerDecoder", "        ");
                        out.push_str(&code);
                        out.push('\n');
                    }
                }
            }
            out.push_str("        return cls(\n");
            for (i, f) in sorted_fields.iter().enumerate() {
                if f.ber.is_some() {
                    let comma = if i < sorted_fields.len() - 1 { "," } else { "" };
                    out.push_str(&format!("            {}=_{}{}\n", f.name, f.name, comma));
                }
            }
            out.push_str("        )\n");
        } else {
            out.push_str("        decoder = DerDecoder(data)\n");
            out.push_str("        _tag = decoder.read_tag()\n");
            out.push_str("        _length = decoder.read_length()\n");
            out.push_str("        _end = decoder._pos + _length\n");
            for f in &sorted_fields {
                if f.ber.is_none() { continue; }
                let ber = f.ber.as_ref().unwrap();
                if f.optional {
                    if let Some(ref default) = f.default {
                        let default_python = Self::value_literal_to_python(default);
                        out.push_str(&format!("        _{} = {}\n", f.name, default_python));
                    } else {
                        out.push_str(&format!("        _{} = None\n", f.name));
                    }
                    out.push_str("        if decoder._pos < _end:\n");
                    out.push_str(&format!("            _{}_save = decoder._pos\n", f.name));
                    out.push_str("            _ft = decoder.read_tag()\n");
                    if !ber.choice_alternative_tags.is_empty() {
                        let tag_checks: Vec<String> = ber.choice_alternative_tags.iter().map(|t| {
                            let constructed_check = if t.constructed { " and _ft[2]" } else { " and not _ft[2]" };
                            format!("(_ft[0] == TagClass.{} and _ft[1] == {}{})", t.tag_class, t.tag_number, constructed_check)
                        }).collect();
                        out.push_str(&format!("            if ({}):\n", tag_checks.join(" or ")));
                    } else {
                        out.push_str(&format!("            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == {} and not _ft[2]:\n", ber.tag_number));
                    }
                    out.push_str("                _fl = decoder.read_length()\n");
                    out.push_str("                _fd = decoder.read_bytes(_fl)\n");
                    for stmt in &f.decode_stmts {
                        let code = self.render_decode_field(stmt, "DerDecoder", "                ");
                        out.push_str(&code);
                        out.push('\n');
                    }
                    out.push_str("            else:\n");
                    out.push_str(&format!("                decoder._pos = _{}_save\n", f.name));
                    if f.default.is_some() {
                        let default_python = Self::value_literal_to_python(f.default.as_ref().unwrap());
                        out.push_str(&format!("                _{} = {}\n", f.name, default_python));
                    }
                } else {
                    out.push_str(&format!("        _ft = decoder.read_tag()\n"));
                    out.push_str(&format!("        _fl = decoder.read_length()\n"));
                    out.push_str(&format!("        _fd = decoder.read_bytes(_fl)\n"));
                    for stmt in &f.decode_stmts {
                        let code = self.render_decode_field(stmt, "DerDecoder", "        ");
                        out.push_str(&code);
                        out.push('\n');
                    }
                }
            }
            out.push_str("        return cls(\n");
            for (i, f) in sorted_fields.iter().enumerate() {
                if f.ber.is_some() {
                    let comma = if i < sorted_fields.len() - 1 { "," } else { "" };
                    out.push_str(&format!("            {}=_{}{}\n", f.name, f.name, comma));
                }
            }
            out.push_str("        )\n");
        }
        out.push_str("\n");
        out.push_str("    @classmethod\n");
        out.push_str(&format!("    def decode_ber_indefinite(cls, data: bytes) -> \"{}\":\n", name));
        out.push_str("        decoder = BerDecoder(data)\n");
        out.push_str("        _tag = decoder.read_tag()\n");
        out.push_str("        _length = decoder.read_length()\n");
        out.push_str("        if _length is not None:\n");
        out.push_str("            raise InvalidLengthError(\"Expected indefinite length\")\n");
        out.push_str("        _content = decoder.read_constructed_indefinite()\n");
        out.push_str("        decoder2 = BerDecoder(_content)\n");
        for f in &sorted_fields {
            if f.ber.is_none() { continue; }
            let ber = f.ber.as_ref().unwrap();
            if f.optional {
                if let Some(ref default) = f.default {
                    let default_python = Self::value_literal_to_python(default);
                    out.push_str(&format!("        _{} = {}\n", f.name, default_python));
                } else {
                    out.push_str(&format!("        _{} = None\n", f.name));
                }
                out.push_str("        if not decoder2.at_end():\n");
                out.push_str(&format!("            _{}_save = decoder2._pos\n", f.name));
                out.push_str("            _ft = decoder2.read_tag()\n");
                if !ber.choice_alternative_tags.is_empty() {
                    let tag_checks: Vec<String> = ber.choice_alternative_tags.iter().map(|t| {
                        let constructed_check = if t.constructed { " and _ft[2]" } else { " and not _ft[2]" };
                        format!("(_ft[0] == TagClass.{} and _ft[1] == {}{})", t.tag_class, t.tag_number, constructed_check)
                    }).collect();
                    out.push_str(&format!("            if ({}):\n", tag_checks.join(" or ")));
                } else {
                    out.push_str(&format!("            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == {} and not _ft[2]:\n", ber.tag_number));
                }
                out.push_str("                _fl = decoder2.read_length()\n");
                out.push_str("                _fd = decoder2.read_bytes(_fl)\n");
                for stmt in &f.decode_stmts {
                    let code = self.render_decode_field_indefinite(stmt, "BerDecoder", "                ");
                    out.push_str(&code);
                    out.push('\n');
                }
                out.push_str("            else:\n");
                out.push_str(&format!("                decoder2._pos = _{}_save\n", f.name));
                if f.default.is_some() {
                    let default_python = Self::value_literal_to_python(f.default.as_ref().unwrap());
                    out.push_str(&format!("                _{} = {}\n", f.name, default_python));
                }
            } else {
                out.push_str(&format!("        _ft = decoder2.read_tag()\n"));
                out.push_str(&format!("        _fl = decoder2.read_length()\n"));
                out.push_str(&format!("        _fd = decoder2.read_bytes(_fl)\n"));
                for stmt in &f.decode_stmts {
                    let code = self.render_decode_field_indefinite(stmt, "BerDecoder", "        ");
                    out.push_str(&code);
                    out.push('\n');
                }
            }
        }
        out.push_str("        return cls(\n");
        for (i, f) in sorted_fields.iter().enumerate() {
            if f.ber.is_some() {
                let comma = if i < sorted_fields.len() - 1 { "," } else { "" };
                out.push_str(&format!("            {}=_{}{}\n", f.name, f.name, comma));
            }
        }
        out.push_str("        )\n");
        Ok(out)
    }

    fn render_choice(&self, name: &str, alternatives: &[ChoiceAlternative], doc_comment: &str) -> Result<String> {
        let mut out = String::new();
        out.push_str("@dataclass\n");
        out.push_str(&format!("class {}(AsnType):\n", name));
        if !doc_comment.is_empty() {
            out.push_str(&format!("    \"\"\"{}\"\"\"\n", doc_comment));
        }
        for a in alternatives {
            let ty_str = self.render_type(&a.ty).unwrap_or_else(|_| "Any".to_string());
            out.push_str(&format!("    {}: Optional[{}] = None\n", a.name, ty_str));
        }
        out.push_str("\n");
        out.push_str("    def encode_ber(self) -> bytes:\n");
        for a in alternatives {
            if a.ber.is_none() { continue; }
            out.push_str(&self.render_choice_encode_alt(a, "BerEncoder", "write_tlv", false));
            out.push('\n');
        }
        out.push_str("        raise ValueError(\"No choice alternative set\")\n");
        out.push_str("\n");
        out.push_str("    def encode_ber_indefinite(self) -> bytes:\n");
        for a in alternatives {
            if a.ber.is_none() { continue; }
            out.push_str(&self.render_choice_encode_alt(a, "BerEncoder", "write_tlv", true));
            out.push('\n');
        }
        out.push_str("        raise ValueError(\"No choice alternative set\")\n");
        out.push_str("\n");
        out.push_str("    def encode_der(self) -> bytes:\n");
        for a in alternatives {
            if a.ber.is_none() { continue; }
            out.push_str(&self.render_choice_encode_alt(a, "DerEncoder", "write_tlv_der", false));
            out.push('\n');
        }
        out.push_str("        raise ValueError(\"No choice alternative set\")\n");
        out.push_str("\n");
        out.push_str("    @classmethod\n");
        out.push_str(&format!("    def decode_ber(cls, data: bytes) -> \"{}\":\n", name));
        out.push_str("        decoder = BerDecoder(data)\n");
        out.push_str("        _tag = decoder.read_tag()\n");
        out.push_str("        _fl = decoder.read_length()\n");
        out.push_str("        _fd = decoder.read_bytes(_fl)\n");
        for a in alternatives {
            if a.ber.is_none() { continue; }
            let ber = a.ber.as_ref().unwrap();
            if ber.encoding == "referenced" || ber.encoding == "constructed" || ber.encoding == "choice" {
                out.push_str(&self.render_choice_decode_referenced(a, "BerDecoder", "decode_ber"));
                out.push('\n');
            } else {
                out.push_str(&self.render_choice_decode_alt(a, "BerDecoder", "decode_ber"));
                out.push('\n');
            }
        }
        out.push_str("        raise ValueError(f\"Unknown choice tag: {{_tag}}\")\n");
        out.push_str("\n");
        out.push_str("    @classmethod\n");
        out.push_str(&format!("    def decode_ber_indefinite(cls, data: bytes) -> \"{}\":\n", name));
        out.push_str("        decoder = BerDecoder(data)\n");
        out.push_str("        _tag = decoder.read_tag()\n");
        out.push_str("        _length = decoder.read_length()\n");
        out.push_str("        if _length is None:\n");
        out.push_str("            _content = decoder.read_constructed_indefinite()\n");
        out.push_str("        else:\n");
        out.push_str("            _content = decoder.read_bytes(_length)\n");
        for a in alternatives {
            if a.ber.is_none() { continue; }
            let ber = a.ber.as_ref().unwrap();
            if ber.encoding == "referenced" || ber.encoding == "constructed" || ber.encoding == "choice" {
                out.push_str(&self.render_choice_decode_referenced_indefinite(a));
                out.push('\n');
            } else {
                out.push_str(&self.render_choice_decode_alt_indefinite(a));
                out.push('\n');
            }
        }
        out.push_str("        raise ValueError(f\"Unknown choice tag: {{_tag}}\")\n");
        out.push_str("\n");
        out.push_str("    @classmethod\n");
        out.push_str(&format!("    def decode_der(cls, data: bytes) -> \"{}\":\n", name));
        out.push_str("        decoder = DerDecoder(data)\n");
        out.push_str("        _tag = decoder.read_tag()\n");
        out.push_str("        _fl = decoder.read_length()\n");
        out.push_str("        _fd = decoder.read_bytes(_fl)\n");
        for a in alternatives {
            if a.ber.is_none() { continue; }
            let ber = a.ber.as_ref().unwrap();
            if ber.encoding == "referenced" || ber.encoding == "constructed" || ber.encoding == "choice" {
                out.push_str(&self.render_choice_decode_referenced(a, "DerDecoder", "decode_der"));
                out.push('\n');
            } else {
                out.push_str(&self.render_choice_decode_alt(a, "DerDecoder", "decode_der"));
                out.push('\n');
            }
        }
        out.push_str("        raise ValueError(f\"Unknown choice tag: {{_tag}}\")\n");
        Ok(out)
    }

    fn render_choice_decode_alt(&self, alt: &ChoiceAlternative, decoder_type: &str, _decode_method: &str) -> String {
        let ber = match &alt.ber {
            Some(b) => b,
            None => return String::new(),
        };
        let tag_class = &ber.tag_class;
        let tag_number = ber.tag_number;
        let constructed = ber.constructed;
        let tagging_mode = &ber.tagging_mode;
        let encoding = &ber.encoding;
        let string_encoding = &ber.string_encoding;

        let constructed_check = if !constructed && tagging_mode == "inherent" { " and not _tag[2]" } else { "" };
        let tag_check = format!("_tag[0] == TagClass.{tag_class} and _tag[1] == {tag_number}{constructed_check}");

        let mut out = String::new();
        match tagging_mode.as_str() {
            "explicit" => {
                out.push_str(&format!("        if {}:\n", tag_check));
                out.push_str(&format!("            _inner_dec = {}(_fd)\n", decoder_type));
                out.push_str("            _inner_dec.read_tag()\n");
                out.push_str("            _inner_len = _inner_dec.read_length()\n");
                out.push_str("            _inner_val = _inner_dec.read_bytes(_inner_len)\n");
                match encoding.as_str() {
                    "integer" | "enumerated" => out.push_str(&format!("            return cls({}=int.from_bytes(_inner_val, byteorder='big', signed=True))\n", alt.name)),
                    "boolean" => out.push_str(&format!("            return cls({}=_inner_val[0] != 0)\n", alt.name)),
                    "string" => out.push_str(&format!("            return cls({}=_inner_val.decode('{}'))\n", alt.name, string_encoding)),
                    "bytes" => out.push_str(&format!("            return cls({}=_inner_val)\n", alt.name)),
                    "bit_string" => out.push_str(&format!("            return cls({}=BitString(_inner_val[1:], _inner_val[0]))\n", alt.name)),
                    "oid" => {
                        out.push_str("            _val, _ = ObjectIdentifier.decode(_inner_val)\n");
                        out.push_str(&format!("            return cls({}=_val)\n", alt.name));
                    }
                    "null" => out.push_str(&format!("            return cls({}=None)\n", alt.name)),
                    "real" => {
                        out.push_str("            import struct\n");
                        out.push_str(&format!("            return cls({}=struct.unpack('>d', _inner_val)[0])\n", alt.name));
                    }
                    "time" => {
                        out.push_str("            from datetime import datetime\n");
                        out.push_str(&format!("            return cls({}=datetime.strptime(_inner_val.decode('ascii'), '%Y%m%d%H%M%SZ'))\n", alt.name));
                    }
                    _ => {}
                }
            }
            _ => {
                out.push_str(&format!("        if {}:\n", tag_check));
                match encoding.as_str() {
                    "integer" | "enumerated" => out.push_str(&format!("            return cls({}=int.from_bytes(_fd, byteorder='big', signed=True))\n", alt.name)),
                    "boolean" => out.push_str(&format!("            return cls({}=_fd[0] != 0)\n", alt.name)),
                    "string" => out.push_str(&format!("            return cls({}=_fd.decode('{}'))\n", alt.name, string_encoding)),
                    "bytes" => out.push_str(&format!("            return cls({}=_fd)\n", alt.name)),
                    "bit_string" => out.push_str(&format!("            return cls({}=BitString(_fd[1:], _fd[0]))\n", alt.name)),
                    "oid" => {
                        out.push_str("            _val, _ = ObjectIdentifier.decode(_fd)\n");
                        out.push_str(&format!("            return cls({}=_val)\n", alt.name));
                    }
                    "null" => out.push_str(&format!("            return cls({}=None)\n", alt.name)),
                    "real" => {
                        out.push_str("            import struct\n");
                        out.push_str(&format!("            return cls({}=struct.unpack('>d', _fd)[0])\n", alt.name));
                    }
                    "time" => {
                        out.push_str("            from datetime import datetime\n");
                        out.push_str(&format!("            return cls({}=datetime.strptime(_fd.decode('ascii'), '%Y%m%d%H%M%SZ'))\n", alt.name));
                    }
                    _ => {}
                }
            }
        }
        out
    }

    fn render_choice_decode_alt_indefinite(&self, alt: &ChoiceAlternative) -> String {
        let ber = match &alt.ber {
            Some(b) => b,
            None => return String::new(),
        };
        let tag_class = &ber.tag_class;
        let tag_number = ber.tag_number;
        let tagging_mode = &ber.tagging_mode;
        let encoding = &ber.encoding;
        let string_encoding = &ber.string_encoding;

        let tag_check = format!("_tag[0] == TagClass.{tag_class} and _tag[1] == {tag_number}");

        let mut out = String::new();
        match tagging_mode.as_str() {
            "explicit" => {
                out.push_str(&format!("        if {}:\n", tag_check));
                out.push_str("            _inner_dec = BerDecoder(_content)\n");
                out.push_str("            _inner_dec.read_tag()\n");
                out.push_str("            _inner_len = _inner_dec.read_length()\n");
                out.push_str("            _inner_val = _inner_dec.read_bytes(_inner_len)\n");
                match encoding.as_str() {
                    "integer" | "enumerated" => out.push_str(&format!("            return cls({}=int.from_bytes(_inner_val, byteorder='big', signed=True))\n", alt.name)),
                    "boolean" => out.push_str(&format!("            return cls({}=_inner_val[0] != 0)\n", alt.name)),
                    "string" => out.push_str(&format!("            return cls({}=_inner_val.decode('{}'))\n", alt.name, string_encoding)),
                    "bytes" => out.push_str(&format!("            return cls({}=_inner_val)\n", alt.name)),
                    "bit_string" => out.push_str(&format!("            return cls({}=BitString(_inner_val[1:], _inner_val[0]))\n", alt.name)),
                    "oid" => {
                        out.push_str("            _val, _ = ObjectIdentifier.decode(_inner_val)\n");
                        out.push_str(&format!("            return cls({}=_val)\n", alt.name));
                    }
                    "null" => out.push_str(&format!("            return cls({}=None)\n", alt.name)),
                    "real" => {
                        out.push_str("            import struct\n");
                        out.push_str(&format!("            return cls({}=struct.unpack('>d', _inner_val)[0])\n", alt.name));
                    }
                    "time" => {
                        out.push_str("            from datetime import datetime\n");
                        out.push_str(&format!("            return cls({}=datetime.strptime(_inner_val.decode('ascii'), '%Y%m%d%H%M%SZ'))\n", alt.name));
                    }
                    _ => {}
                }
            }
            _ => {
                out.push_str(&format!("        if {}:\n", tag_check));
                match encoding.as_str() {
                    "integer" | "enumerated" => out.push_str(&format!("            return cls({}=int.from_bytes(_content, byteorder='big', signed=True))\n", alt.name)),
                    "boolean" => out.push_str(&format!("            return cls({}=_content[0] != 0)\n", alt.name)),
                    "string" => out.push_str(&format!("            return cls({}=_content.decode('{}'))\n", alt.name, string_encoding)),
                    "bytes" => out.push_str(&format!("            return cls({}=_content)\n", alt.name)),
                    "bit_string" => out.push_str(&format!("            return cls({}=BitString(_content[1:], _content[0]))\n", alt.name)),
                    "oid" => {
                        out.push_str("            _val, _ = ObjectIdentifier.decode(_content)\n");
                        out.push_str(&format!("            return cls({}=_val)\n", alt.name));
                    }
                    "null" => out.push_str(&format!("            return cls({}=None)\n", alt.name)),
                    "real" => {
                        out.push_str("            import struct\n");
                        out.push_str(&format!("            return cls({}=struct.unpack('>d', _content)[0])\n", alt.name));
                    }
                    "time" => {
                        out.push_str("            from datetime import datetime\n");
                        out.push_str(&format!("            return cls({}=datetime.strptime(_content.decode('ascii'), '%Y%m%d%H%M%SZ'))\n", alt.name));
                    }
                    _ => {}
                }
            }
        }
        out
    }

    fn render_choice_decode_referenced(&self, alt: &ChoiceAlternative, _decoder_type: &str, decode_method: &str) -> String {
        let ber = match &alt.ber {
            Some(b) => b,
            None => return String::new(),
        };
        let tag_class = &ber.tag_class;
        let tag_number = ber.tag_number;
        let constructed = ber.constructed;
        let tagging_mode = &ber.tagging_mode;
        let inner_type = ber.referenced_type.as_deref().unwrap_or("object");

        let constructed_check = if !constructed && tagging_mode == "inherent" { " and not _tag[2]" } else { "" };
        let tag_check = format!("_tag[0] == TagClass.{tag_class} and _tag[1] == {tag_number}{constructed_check}");

        let mut out = String::new();
        out.push_str(&format!("        if {}:\n", tag_check));
        match tagging_mode.as_str() {
            "explicit" => {
                out.push_str(&format!("            return cls({}={}.{}(_fd))\n", alt.name, inner_type, decode_method));
            }
            "implicit" => {
                let inherent_tag_class = &ber.inherent_tag_class;
                let inherent_tag_number = ber.inherent_tag_number;
                out.push_str("            _re = BerEncoder()\n");
                out.push_str(&format!("            _re.write_tag(TagClass.{}, {}, {})\n", inherent_tag_class, inherent_tag_number, constructed));
                out.push_str("            _re.write_length(len(_fd))\n");
                out.push_str("            _re.write_bytes(_fd)\n");
                out.push_str(&format!("            return cls({}={}.{}(_re.finish()))\n", alt.name, inner_type, decode_method));
            }
            _ => {
                out.push_str("            _re = BerEncoder()\n");
                out.push_str("            _re.write_tag(_tag[0], _tag[1], _tag[2])\n");
                out.push_str("            _re.write_length(_fl)\n");
                out.push_str("            _re.write_bytes(_fd)\n");
                out.push_str(&format!("            return cls({}={}.{}(_re.finish()))\n", alt.name, inner_type, decode_method));
            }
        }
        out
    }

    fn render_choice_decode_referenced_indefinite(&self, alt: &ChoiceAlternative) -> String {
        let ber = match &alt.ber {
            Some(b) => b,
            None => return String::new(),
        };
        let tag_class = &ber.tag_class;
        let tag_number = ber.tag_number;
        let tagging_mode = &ber.tagging_mode;
        let inner_type = ber.referenced_type.as_deref().unwrap_or("object");

        let tag_check = format!("_tag[0] == TagClass.{tag_class} and _tag[1] == {tag_number}");

        let mut out = String::new();
        out.push_str(&format!("        if {}:\n", tag_check));
        match tagging_mode.as_str() {
            "explicit" => {
                out.push_str(&format!("            return cls({}={}.decode_ber(_content))\n", alt.name, inner_type));
            }
            "implicit" => {
                let inherent_tag_class = &ber.inherent_tag_class;
                let inherent_tag_number = ber.inherent_tag_number;
                let constructed = ber.constructed;
                out.push_str("            _re = BerEncoder()\n");
                out.push_str(&format!("            _re.write_tag(TagClass.{}, {}, {})\n", inherent_tag_class, inherent_tag_number, constructed));
                out.push_str("            _re.write_length(len(_content))\n");
                out.push_str("            _re.write_bytes(_content)\n");
                out.push_str(&format!("            return cls({}={}.decode_ber(_re.finish()))\n", alt.name, inner_type));
            }
            _ => {
                out.push_str("            _re = BerEncoder()\n");
                out.push_str("            _re.write_tag(_tag[0], _tag[1], _tag[2])\n");
                out.push_str("            _re.write_length(len(_content))\n");
                out.push_str("            _re.write_bytes(_content)\n");
                out.push_str(&format!("            return cls({}={}.decode_ber(_re.finish()))\n", alt.name, inner_type));
            }
        }
        out
    }

    fn render_list_type(&self, name: &str, element_type: &str, ber: &BerFieldInfo, doc_comment: &str) -> Result<String> {
        let mut out = String::new();
        out.push_str(&format!("class {name}(list, AsnType):\n"));
        if !doc_comment.is_empty() {
            out.push_str(&format!("    \"\"\"{}\"\"\"\n", doc_comment));
        }
        out.push_str(&format!("    def __init__(self, items: list[{element_type}] | None = None):\n"));
        out.push_str("        super().__init__(items or [])\n");
        out.push_str("\n");
        out.push_str("    def encode_ber(self) -> bytes:\n");
        out.push_str("        _outer = BerEncoder()\n");
        out.push_str("        _content = bytearray()\n");
        out.push_str("        for _item in self:\n");
        if let Some(inner) = &ber.list_element_ber {
            let elem_encode = match inner.encoding.as_str() {
                "constructed" | "referenced" => "            _content.extend(_item.encode_der())\n",
                "choice" => "            _content.extend(_item.encode_der())\n",
                "list" => "            _content.extend(_item.encode_der())\n",
                "integer" => "            _ie = BerEncoder()\n            _ie.write_integer(_item)\n            _ib = _ie.finish()\n            _ite = BerEncoder()\n            _ite.write_tag(TagClass.UNIVERSAL, 2, False)\n            _ite.write_length(len(_ib))\n            _ite.write_bytes(_ib)\n            _content.extend(_ite.finish())\n",
                "enumerated" => "            _ie = BerEncoder()\n            _ie.write_integer(_item.value if hasattr(_item, 'value') else _item)\n            _ib = _ie.finish()\n            _ite = BerEncoder()\n            _ite.write_tag(TagClass.UNIVERSAL, 10, False)\n            _ite.write_length(len(_ib))\n            _ite.write_bytes(_ib)\n            _content.extend(_ite.finish())\n",
                "boolean" => "            _be = BerEncoder()\n            _be.write_tag(TagClass.UNIVERSAL, 1, False)\n            _be.write_length(1)\n            _be.write_bytes(b'\\xff' if _item else b'\\x00')\n            _content.extend(_be.finish())\n",
                "string" => &format!("            _sb = _item.encode('{}')\n            _se = BerEncoder()\n            _se.write_tag(TagClass.UNIVERSAL, {}, False)\n            _se.write_length(len(_sb))\n            _se.write_bytes(_sb)\n            _content.extend(_se.finish())\n", inner.string_encoding, inner.tag_number),
                "bytes" => "            _be = BerEncoder()\n            _be.write_tlv(TagClass.UNIVERSAL, 4, _item)\n            _content.extend(_be.finish())\n",
                "bit_string" => "            _be = BerEncoder()\n            _be.write_tag(TagClass.UNIVERSAL, 3, False)\n            _be.write_length(len(_item.data) + 1)\n            _be.write_bytes(bytes([_item.unused_bits]))\n            _be.write_bytes(_item.data)\n            _content.extend(_be.finish())\n",
                "oid" => "            _ob = _item.encode()\n            _oe = BerEncoder()\n            _oe.write_tag(TagClass.UNIVERSAL, 6, False)\n            _oe.write_length(len(_ob))\n            _oe.write_bytes(_ob)\n            _content.extend(_oe.finish())\n",
                _ => "            if hasattr(_item, 'encode_ber'):\n                _content.extend(_item.encode_ber())\n",
            };
            out.push_str(elem_encode);
        } else {
            out.push_str("            if hasattr(_item, 'encode_ber'):\n");
            out.push_str("                _content.extend(_item.encode_ber())\n");
        }
        out.push_str(&format!("        _outer.write_tag(TagClass.UNIVERSAL, {}, True)\n", ber.tag_number));
        out.push_str("        _outer.write_length(len(_content))\n");
        out.push_str("        _outer.write_bytes(_content)\n");
        out.push_str("        return _outer.finish()\n");
        out.push_str("\n");
        out.push_str("    @classmethod\n");
        out.push_str(&format!("    def decode_ber(cls, data: bytes) -> \"{name}\":\n"));
        out.push_str("        return cls.decode_der(data)\n");
        out.push_str("\n");
        out.push_str("    def encode_der(self) -> bytes:\n");
        out.push_str("        return self.encode_ber()\n");
        out.push_str("\n");
        out.push_str("    @classmethod\n");
        out.push_str(&format!("    def decode_der(cls, data: bytes) -> \"{name}\":\n"));
        out.push_str("        decoder = DerDecoder(data)\n");
        out.push_str("        _tag = decoder.read_tag()\n");
        out.push_str("        _length = decoder.read_length()\n");
        out.push_str("        _end = decoder._pos + _length\n");
        out.push_str(&format!("        items: list[{element_type}] = []\n"));
        out.push_str("        while decoder._pos < _end:\n");
        if let Some(inner) = &ber.list_element_ber {
            let elem_decode = match inner.encoding.as_str() {
                "constructed" | "referenced" => {
                    let ref_type = inner.referenced_type.as_deref().unwrap_or("object");
                    format!("            _item_start = decoder._pos\n            _lt = decoder.read_tag()\n            _ll = decoder.read_length()\n            _ = decoder.read_bytes(_ll)\n            items.append({ref_type}.decode_der(decoder._data[_item_start:decoder._pos]))\n")
                }
                "choice" => {
                    let ref_type = inner.referenced_type.as_deref().unwrap_or("object");
                    format!("            _item_start = decoder._pos\n            _lt = decoder.read_tag()\n            _ll = decoder.read_length()\n            _ = decoder.read_bytes(_ll)\n            items.append({ref_type}.decode_der(decoder._data[_item_start:decoder._pos]))\n")
                }
                "list" => {
                    let ref_type = inner.referenced_type.as_deref().unwrap_or("object");
                    format!("            _item_start = decoder._pos\n            _lt = decoder.read_tag()\n            _ll = decoder.read_length()\n            _ = decoder.read_bytes(_ll)\n            items.append({ref_type}.decode_der(decoder._data[_item_start:decoder._pos]))\n")
                }
                "integer" | "enumerated" => format!("            _lt = decoder.read_tag()\n            _ll = decoder.read_length()\n            _lv = decoder.read_bytes(_ll)\n            items.append(int.from_bytes(_lv, byteorder='big', signed=True))\n"),
                "boolean" => format!("            _lt = decoder.read_tag()\n            _ll = decoder.read_length()\n            _lv = decoder.read_bytes(_ll)\n            items.append(_lv[0] != 0)\n"),
                "bytes" => format!("            _lt = decoder.read_tag()\n            _ll = decoder.read_length()\n            items.append(decoder.read_bytes(_ll))\n"),
                "bit_string" => format!("            _lt = decoder.read_tag()\n            _ll = decoder.read_length()\n            _lv = decoder.read_bytes(_ll)\n            from asnvil_runtime import BitString\n            items.append(BitString(_lv[1:], _lv[0]))\n"),
                "oid" => format!("            _lt = decoder.read_tag()\n            _ll = decoder.read_length()\n            _lv = decoder.read_bytes(_ll)\n            _oid, _ = ObjectIdentifier.decode(_lv)\n            items.append(_oid)\n"),
                "string" => format!("            _lt = decoder.read_tag()\n            _ll = decoder.read_length()\n            _lv = decoder.read_bytes(_ll)\n            items.append(_lv.decode('{}'))\n", inner.string_encoding),
                "null" => format!("            _lt = decoder.read_tag()\n            _ll = decoder.read_length()\n            _ld.read_bytes(_ll)\n            items.append(None)\n"),
                "real" => format!("            import struct\n            _lt = decoder.read_tag()\n            _ll = decoder.read_length()\n            _lv = decoder.read_bytes(_ll)\n            items.append(struct.unpack('>d', _lv)[0])\n"),
                "time" => format!("            from datetime import datetime\n            _lt = decoder.read_tag()\n            _ll = decoder.read_length()\n            _lv = decoder.read_bytes(_ll)\n            items.append(datetime.strptime(_lv.decode('ascii'), '%Y%m%d%H%M%SZ'))\n"),
                _ => format!("            _lt = decoder.read_tag()\n            _ll = decoder.read_length()\n            _lv = decoder.read_bytes(_ll)\n            items.append(_lv)\n"),
            };
            out.push_str(&elem_decode);
        } else {
            out.push_str("            _lt = decoder.read_tag()\n");
            out.push_str("            _ll = decoder.read_length()\n");
            out.push_str("            _lv = decoder.read_bytes(_ll)\n");
            out.push_str("            items.append(_lv)\n");
        }
        out.push_str("        return cls(items)\n");
        out.push_str("\n");
        out.push_str("    def encode_ber_indefinite(self) -> bytes:\n");
        out.push_str("        _outer = BerEncoder()\n");
        out.push_str("        _content = bytearray()\n");
        out.push_str("        for _item in self:\n");
        if let Some(inner) = &ber.list_element_ber {
            let elem_encode = match inner.encoding.as_str() {
                "constructed" | "referenced" => "            _content.extend(_item.encode_der())\n",
                "choice" => "            _content.extend(_item.encode_der())\n",
                "list" => "            _content.extend(_item.encode_der())\n",
                "integer" => "            _ie = BerEncoder()\n            _ie.write_integer(_item)\n            _ib = _ie.finish()\n            _ite = BerEncoder()\n            _ite.write_tag(TagClass.UNIVERSAL, 2, False)\n            _ite.write_length(len(_ib))\n            _ite.write_bytes(_ib)\n            _content.extend(_ite.finish())\n",
                "enumerated" => "            _ie = BerEncoder()\n            _ie.write_integer(_item.value if hasattr(_item, 'value') else _item)\n            _ib = _ie.finish()\n            _ite = BerEncoder()\n            _ite.write_tag(TagClass.UNIVERSAL, 10, False)\n            _ite.write_length(len(_ib))\n            _ite.write_bytes(_ib)\n            _content.extend(_ite.finish())\n",
                "boolean" => "            _be = BerEncoder()\n            _be.write_tag(TagClass.UNIVERSAL, 1, False)\n            _be.write_length(1)\n            _be.write_bytes(b'\\xff' if _item else b'\\x00')\n            _content.extend(_be.finish())\n",
                "string" => &format!("            _sb = _item.encode('{}')\n            _se = BerEncoder()\n            _se.write_tag(TagClass.UNIVERSAL, {}, False)\n            _se.write_length(len(_sb))\n            _se.write_bytes(_sb)\n            _content.extend(_se.finish())\n", inner.string_encoding, inner.tag_number),
                "bytes" => "            _be = BerEncoder()\n            _be.write_tlv(TagClass.UNIVERSAL, 4, _item)\n            _content.extend(_be.finish())\n",
                "bit_string" => "            _be = BerEncoder()\n            _be.write_tag(TagClass.UNIVERSAL, 3, False)\n            _be.write_length(len(_item.data) + 1)\n            _be.write_bytes(bytes([_item.unused_bits]))\n            _be.write_bytes(_item.data)\n            _content.extend(_be.finish())\n",
                "oid" => "            _ob = _item.encode()\n            _oe = BerEncoder()\n            _oe.write_tag(TagClass.UNIVERSAL, 6, False)\n            _oe.write_length(len(_ob))\n            _oe.write_bytes(_ob)\n            _content.extend(_oe.finish())\n",
                _ => "            if hasattr(_item, 'encode_ber'):\n                _content.extend(_item.encode_ber())\n",
            };
            out.push_str(elem_encode);
        } else {
            out.push_str("            if hasattr(_item, 'encode_ber'):\n");
            out.push_str("                _content.extend(_item.encode_ber())\n");
        }
        out.push_str(&format!("        _outer.write_tag(TagClass.UNIVERSAL, {}, True)\n", ber.tag_number));
        out.push_str("        _outer.write_length(0, indefinite=True)\n");
        out.push_str("        _outer.write_bytes(_content)\n");
        out.push_str("        _outer.write_eoc()\n");
        out.push_str("        return _outer.finish()\n");
        out.push_str("\n");
        out.push_str("    @classmethod\n");
        out.push_str(&format!("    def decode_ber_indefinite(cls, data: bytes) -> \"{name}\":\n"));
        out.push_str("        decoder = BerDecoder(data)\n");
        out.push_str("        _tag = decoder.read_tag()\n");
        out.push_str("        _length = decoder.read_length()\n");
        out.push_str("        if _length is None:\n");
        out.push_str("            _content = decoder.read_constructed_indefinite()\n");
        out.push_str("        else:\n");
        out.push_str("            _content = decoder.read_bytes(_length)\n");
        out.push_str(&format!("        items: list[{element_type}] = []\n"));
        out.push_str("        _ld = BerDecoder(_content)\n");
        out.push_str("        while not _ld.at_end():\n");
        if let Some(inner) = &ber.list_element_ber {
            let elem_decode = match inner.encoding.as_str() {
                "constructed" | "referenced" => {
                    let ref_type = inner.referenced_type.as_deref().unwrap_or("object");
                    format!("            _item_start = _ld._pos\n            _lt = _ld.read_tag()\n            _ll = _ld.read_length()\n            _ = _ld.read_bytes(_ll)\n            items.append({ref_type}.decode_der(_ld._data[_item_start:_ld._pos]))\n")
                }
                "choice" => {
                    let ref_type = inner.referenced_type.as_deref().unwrap_or("object");
                    format!("            _item_start = _ld._pos\n            _lt = _ld.read_tag()\n            _ll = _ld.read_length()\n            _ = _ld.read_bytes(_ll)\n            items.append({ref_type}.decode_der(_ld._data[_item_start:_ld._pos]))\n")
                }
                "list" => {
                    let ref_type = inner.referenced_type.as_deref().unwrap_or("object");
                    format!("            _item_start = _ld._pos\n            _lt = _ld.read_tag()\n            _ll = _ld.read_length()\n            _ = _ld.read_bytes(_ll)\n            items.append({ref_type}.decode_der(_ld._data[_item_start:_ld._pos]))\n")
                }
                "integer" | "enumerated" => format!("            _lt = _ld.read_tag()\n            _ll = _ld.read_length()\n            _lv = _ld.read_bytes(_ll)\n            items.append(int.from_bytes(_lv, byteorder='big', signed=True))\n"),
                "boolean" => format!("            _lt = _ld.read_tag()\n            _ll = _ld.read_length()\n            _lv = _ld.read_bytes(_ll)\n            items.append(_lv[0] != 0)\n"),
                "bytes" => format!("            _lt = _ld.read_tag()\n            _ll = _ld.read_length()\n            items.append(_ld.read_bytes(_ll))\n"),
                "bit_string" => format!("            _lt = _ld.read_tag()\n            _ll = _ld.read_length()\n            _lv = _ld.read_bytes(_ll)\n            from asnvil_runtime import BitString\n            items.append(BitString(_lv[1:], _lv[0]))\n"),
                "oid" => format!("            _lt = _ld.read_tag()\n            _ll = _ld.read_length()\n            _lv = _ld.read_bytes(_ll)\n            _oid, _ = ObjectIdentifier.decode(_lv)\n            items.append(_oid)\n"),
                "string" => format!("            _lt = _ld.read_tag()\n            _ll = _ld.read_length()\n            _lv = _ld.read_bytes(_ll)\n            items.append(_lv.decode('{}'))\n", inner.string_encoding),
                "null" => format!("            _lt = _ld.read_tag()\n            _ll = _ld.read_length()\n            _ld.read_bytes(_ll)\n            items.append(None)\n"),
                "real" => format!("            import struct\n            _lt = _ld.read_tag()\n            _ll = _ld.read_length()\n            _lv = _ld.read_bytes(_ll)\n            items.append(struct.unpack('>d', _lv)[0])\n"),
                "time" => format!("            from datetime import datetime\n            _lt = _ld.read_tag()\n            _ll = _ld.read_length()\n            _lv = _ld.read_bytes(_ll)\n            items.append(datetime.strptime(_lv.decode('ascii'), '%Y%m%d%H%M%SZ'))\n"),
                _ => format!("            _lt = _ld.read_tag()\n            _ll = _ld.read_length()\n            _lv = _ld.read_bytes(_ll)\n            items.append(_lv)\n"),
            };
            out.push_str(&elem_decode);
        } else {
            out.push_str("            _lt = _ld.read_tag()\n");
            out.push_str("            _ll = _ld.read_length()\n");
            out.push_str("            _lv = _ld.read_bytes(_ll)\n");
            out.push_str("            items.append(_lv)\n");
        }
        out.push_str("        return cls(items)\n");
        Ok(out)
    }

    fn render_decode_field(&self, stmt: &DecodeStmt, decoder_type: &str, indent: &str) -> String {
        match stmt {
            DecodeStmt::ReadInteger { name } | DecodeStmt::ReadEnumerated { name } => tpl(DecodeInteger { indent, name }),
            DecodeStmt::ReadBoolean { name } => tpl(DecodeBoolean { indent, name }),
            DecodeStmt::ReadString { name, encoding } => tpl(DecodeString { indent, name, encoding }),
            DecodeStmt::ReadBytes { name } => tpl(DecodeBytes { indent, name }),
            DecodeStmt::ReadBitString { name } => tpl(DecodeBitString { indent, name }),
            DecodeStmt::ReadOid { name } => tpl(DecodeOid { indent, name }),
            DecodeStmt::ReadNull { name } => tpl(DecodeNull { indent, name }),
            DecodeStmt::ReadReal { name } => tpl(DecodeReal { indent, name }),
            DecodeStmt::ReadTime { name } => tpl(DecodeTime { indent, name }),
            DecodeStmt::ReadAny { name, reconstruct_tlv: true } => tpl(DecodeAnyTlv { indent, name }),
            DecodeStmt::ReadAny { name, .. } => tpl(DecodeAny { indent, name }),
            DecodeStmt::ReadReferenced { name, inner_type, .. } | DecodeStmt::ReadChoice { name, inner_type, .. } => tpl(DecodeReferencedTlv { indent, name, inner_type, decode_method: "decode_der" }),
            DecodeStmt::ReadList { name, element_info } => {
                let ii = format!("{indent}    ");
                let element_decode = match element_info.encoding.as_str() {
                    "constructed" | "referenced" | "choice" | "list" => {
                        let ref_type = if element_info.referenced_type.is_empty() { "object" } else { &element_info.referenced_type };
                        format!("{ii}_{name}_start = _{name}_ld._pos\n{ii}_lt = _{name}_ld.read_tag()\n{ii}_ll = _{name}_ld.read_length()\n{ii}_lv = _{name}_ld.read_bytes(_ll)\n{ii}_{name}.append({ref_type}.decode_der(_{name}_ld._data[_{name}_start:_{name}_ld._pos]))", name = name, ref_type = ref_type)
                    }
                    "integer" | "enumerated" => {
                        format!("{ii}_lt = _{name}_ld.read_tag()\n{ii}_ll = _{name}_ld.read_length()\n{ii}_lv = _{name}_ld.read_bytes(_ll)\n{ii}_{name}.append(int.from_bytes(_lv, byteorder='big', signed=True))", name = name)
                    }
                    "boolean" => {
                        format!("{ii}_lt = _{name}_ld.read_tag()\n{ii}_ll = _{name}_ld.read_length()\n{ii}_lv = _{name}_ld.read_bytes(_ll)\n{ii}_{name}.append(_lv[0] != 0)", name = name)
                    }
                    "string" => {
                        format!("{ii}_lt = _{name}_ld.read_tag()\n{ii}_ll = _{name}_ld.read_length()\n{ii}_lv = _{name}_ld.read_bytes(_ll)\n{ii}_{name}.append(_lv.decode('{enc}'))", name = name, enc = element_info.string_encoding)
                    }
                    "bytes" => {
                        format!("{ii}_lt = _{name}_ld.read_tag()\n{ii}_ll = _{name}_ld.read_length()\n{ii}_{name}.append(_{name}_ld.read_bytes(_ll))", name = name)
                    }
                    "bit_string" => {
                        format!("{ii}_lt = _{name}_ld.read_tag()\n{ii}_ll = _{name}_ld.read_length()\n{ii}_lv = _{name}_ld.read_bytes(_ll)\n{ii}_{name}.append(BitString(_lv[1:], _lv[0]))", name = name)
                    }
                    "oid" => {
                        format!("{ii}_lt = _{name}_ld.read_tag()\n{ii}_ll = _{name}_ld.read_length()\n{ii}_lv = _{name}_ld.read_bytes(_ll)\n{ii}_li_oid, _ = ObjectIdentifier.decode(_lv)\n{ii}_{name}.append(_li_oid)", name = name)
                    }
                    "null" => {
                        format!("{ii}_lt = _{name}_ld.read_tag()\n{ii}_ll = _{name}_ld.read_length()\n{ii}_{name}_ld.read_bytes(_ll)\n{ii}_{name}.append(None)", name = name)
                    }
                    "real" => {
                        format!("{ii}import struct\n{ii}_lt = _{name}_ld.read_tag()\n{ii}_ll = _{name}_ld.read_length()\n{ii}_lv = _{name}_ld.read_bytes(_ll)\n{ii}_{name}.append(struct.unpack('>d', _lv)[0])", name = name)
                    }
                    "time" => {
                        format!("{ii}from datetime import datetime\n{ii}_lt = _{name}_ld.read_tag()\n{ii}_ll = _{name}_ld.read_length()\n{ii}_lv = _{name}_ld.read_bytes(_ll)\n{ii}_{name}.append(datetime.strptime(_lv.decode('ascii'), '%Y%m%d%H%M%SZ'))", name = name)
                    }
                    _ => {
                        format!("{ii}_lt = _{name}_ld.read_tag()\n{ii}_ll = _{name}_ld.read_length()\n{ii}_{name}.append(_{name}_ld.read_bytes(_ll))", name = name)
                    }
                };
                format!(
                    "{indent}_{name}_ld = {decoder_type}(_fd)\n\
                     {indent}_{name} = []\n\
                     {indent}while not _{name}_ld.at_end():\n\
                     {element_decode}",
                    indent = indent, name = name, decoder_type = decoder_type,
                    element_decode = element_decode
                )
            }
        }
    }

    fn render_decode_field_indefinite(&self, stmt: &DecodeStmt, decoder_type: &str, indent: &str) -> String {
        match stmt {
            DecodeStmt::ReadInteger { name } | DecodeStmt::ReadEnumerated { name } => tpl(DecodeInteger { indent, name }),
            DecodeStmt::ReadBoolean { name } => tpl(DecodeBoolean { indent, name }),
            DecodeStmt::ReadString { name, encoding } => tpl(DecodeString { indent, name, encoding }),
            DecodeStmt::ReadBytes { name } => tpl(DecodeBytes { indent, name }),
            DecodeStmt::ReadBitString { name } => tpl(DecodeBitString { indent, name }),
            DecodeStmt::ReadOid { name } => tpl(DecodeOid { indent, name }),
            DecodeStmt::ReadNull { name } => tpl(DecodeNull { indent, name }),
            DecodeStmt::ReadReal { name } => tpl(DecodeReal { indent, name }),
            DecodeStmt::ReadTime { name } => tpl(DecodeTime { indent, name }),
            DecodeStmt::ReadAny { name, .. } => tpl(DecodeAny { indent, name }),
            DecodeStmt::ReadReferenced { name, inner_type, decode_method, .. } | DecodeStmt::ReadChoice { name, inner_type, decode_method, .. } => {
                tpl(DecodeReferencedTlv { indent, name, inner_type, decode_method })
            }
            DecodeStmt::ReadList { name, .. } => {
                let ii = format!("{indent}    ");
                let element_decode = format!("{ii}_lt = _{name}_ld.read_tag()\n{ii}_ll = _{name}_ld.read_length()\n{ii}_{name}.append(_{name}_ld.read_bytes(_ll))", name = name);
                format!(
                    "{indent}_{name}_ld = {decoder_type}(_fd)\n\
                     {indent}_{name} = []\n\
                     {indent}while not _{name}_ld.at_end():\n\
                     {element_decode}",
                    indent = indent, name = name, decoder_type = decoder_type,
                    element_decode = element_decode
                )
            }
        }
    }
}
