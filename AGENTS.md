# AGENTS.md — asn1c Project

## Project Overview

**asn1c** is an ASN.1 compiler written in Rust using the Parol v4 parser generator. It parses `.asn1` files (ITU-T X.680–X.683, 2021) and generates source code with encode/decode support in multiple target languages (Python first, Rust/TS/C/Go planned).

## Architecture

```
ASN.1 source (.asn1)
    ↓
[asn1c-parser] — Parol LL(k) parser → AST
    ↓
[asn1c-ir] — Semantic analyzer → Language-agnostic IR
    ↓
[asn1c-codegen] — Code AST construction + per-language renderers
    ↓
[Language Renderer] — Target language source (Python first)
    ↓
[asn1c-runtime-python] — Pure stdlib Python runtime (ships alongside generated code)
```

## Crates

| Crate | Purpose |
|---|---|
| `asn1c` | CLI binary (`asn1c <file.asn1> -o output/`) |
| `asn1c-parser` | Parol grammar (`asn1.par`), build.rs generation, AST types |
| `asn1c-ir` | Intermediate representation (resolved types, constraints, values) |
| `asn1c-codegen` | Code AST builder + Python renderer with **Askama** templates |
| `asn1c-runtime-python/` | Pure Python runtime (NOT a pip package, ships as directory) |

## Key Files

| File | What It Is |
|---|---|
| `asn1c-parser/src/asn1.par` | Full ASN.1 grammar (172 lines, X.680–X.683) |
| `asn1c-parser/build.rs` | Parol code generation + inner-attribute stripping |
| `asn1c-parser/src/lib.rs` | Module includes for generated parser/trait/scanner |
| `asn1c-parser/src/grammar.rs` | User-defined `Grammar<'t>` implementing `GrammarTrait` |
| `asn1c-parser/src/ast.rs` | Hand-written AST types for parse tree |
| `asn1c-ir/src/ir.rs` | IR data structures (AsnModule, AsnType, etc.) |
| `asn1c-ir/src/resolver.rs` | Type resolution, import/export, circular ref detection |
| `asn1c-codegen/src/builder.rs` | IR → Code AST transformation |
| `asn1c-codegen/src/python.rs` | Python renderer with **Askama** derive-based templates |
| `asn1c-codegen/templates/python/` | **Askama** templates (.txt): struct, choice, enum, type_alias, module_header, list_type |

## Critical Parol v4 Integration Notes

### Build Script Pattern
The build.rs **must** strip `#![allow(...)]` inner attributes from generated files before they're included via `include!()`. Without this, compilation fails with "inner attribute not permitted in this context."

### Module Structure
- Generated `parser.rs` is included as `mod parser { include!(...) }`
- Generated `grammar_trait.rs` is included as `mod grammar_trait { include!(...) }`
- The `scanner!` macro inside parser.rs generates `pub mod grammar_scanner { ... }` as a submodule
- `src/grammar.rs` defines the user `Grammar<'t>` struct

### Scanner Naming
- Default user type name: `"Grammar"` → scanner struct: `GrammarScanner`, submodule: `grammar_scanner`
- Scanner module resolves as `self::grammar_scanner` from within the parser module

### Common Errors
| Error | Fix |
|---|---|
| `inner attribute is not permitted` | Strip `#![allow(...)]` in build.rs |
| `unresolved import grammar_scanner` | Scanner IS generated inside parser.rs via `scanner!` macro |
| `no 'Grammar' in 'grammar'` | Create `src/grammar.rs` with `Grammar<'t>` struct |
| `ParseTree` lifetime error | `parol_runtime::ParseTree` takes type param, not lifetime |
| Build hangs for minutes | Grammar ambiguity (same token matches multiple alternatives) |

See the **parol-parser** skill for the full reference.

## Development Commands

```bash
cargo build                    # Build all crates
cargo build -p asn1c-parser    # Build parser only (triggers Parol generation)
cargo run -- -o output/ test.asn1    # Compile an ASN.1 file
cargo run -- --help            # CLI help
cargo run --example demo       # Run full pipeline demo (IR → codegen → DER test)
```

## Principles

- **Fix warnings at the source** — Never suppress or work around compiler/clippy warnings. Understand the warning and fix the actual code. No `#[allow(...)]` unless the warning is from generated code (e.g. Parol output).

## Current Status

### Milestone 1: Skeleton ✅
- Workspace with all 5 crates
- Parol grammar for full ASN.1 (types, values, constraints, parameterization)
- CLI with all planned options
- IR with type resolution and circular reference detection
- Code AST builder + Python renderer
- Pure Python runtime with BER/DER encoder/decoder
- End-to-end demo: IR → Python code → DER encode/decode roundtrip

### Milestone 2: Core Parser ✅
- `GrammarTrait` callbacks in `grammar.rs` (939 lines) with stack-based AST construction
- `asn1c-ir/src/from_ast.rs` converts parser AST → IR
- CLI in `main.rs` runs real parse → AST → IR pipeline
- All 20+ ASN.1 types parsed (Sequence, Set, Choice, Enumerated, BitString, Tagged, etc.)

### Milestone 3: Semantic Analysis ✅
- `resolver.rs` (334 lines) with type resolution, import/export validation, circular ref detection
- Recursive resolution for ReferencedType, Sequence, Set, Choice, SequenceOf, SetOf, Tagged, ConstrainedType
- Cross-module reference support with module context
- Minor gap: `constraint_to_ir()` returns empty constraints (stubbed but IR structures defined)

### Milestone 4: Code Generation ✅
- Full IR → Code AST → Python pipeline with BER encode/decode logic
- `builder.rs` (500 lines) handles all type variants with `ber_info_for_type()`
- `struct.j2` template (639 lines) generates per-field encode/decode
- `choice.j2` template generates tagged union pattern with try/except fallback for constructed types
- SEQUENCE OF / SET OF list encoding with type-aware element TLV wrapping
- DEFAULT value support (extracted from parser, converted via `ValueLiteral`, rendered as Python defaults)
- BitString, ObjectIdentifier, AsnError runtime imports
- 12 roundtrip tests all passing (Person, Department, Company, Config, Certificate, CHOICE types, nested CHOICE)

### Milestone 5: DER Canonicalization ✅
- Complete DER encoder with strict validation (`DerEncoder` with minimal integer/length encoding)
- Complete DER decoder with canonicalization checks (`DerDecoder` validates minimal encoding, rejects indefinite length)
- SET element sorting by TLV for DER canonicalization (lexicographic order of encoded bytes)
- Generated Python includes `encode_der()` / `decode_der()` methods alongside BER methods
- `DerEncoder` enforces: no indefinite length, minimal integer encoding, definite length only
- `DerDecoder` enforces: no indefinite length, minimal integer validation, boolean 0x00/0xFF validation
- `read_set_elements()` validates SET elements are in canonical DER order
- `sort_set_tlv()` sorts SET elements lexicographically by their full TLV encoding
- 17 roundtrip tests passing (12 BER + 5 DER)

### Milestone 6: Integration Tests + RFC 5912 Support ✅

#### Completed
- **Grammar fixes** (`asn1c-parser/src/asn1.par`):
  - `TaggedType` now supports bracket notation `[0]`, `[1]` for context-specific tags
  - `'DEFINED'` added to `IdentifierOrKeyword` keyword list
  - `OpenType` moved before `AnyType` in Type alternatives (resolves ANY/ANY DEFINED BY ambiguity)
- **Parser fixes** (`asn1c-parser/src/grammar.rs`):
  - `open_type` callback handles new grammar structure (no lifetime param)
  - `value` callback now pops from `str_stack` for Identifier/Reference cases (fixes DEFAULT value reference pollution)
- **AST/IR fixes** (`asn1c-ir/src/from_ast.rs`, `asn1c-ir/src/ir.rs`):
  - `OpenType` variant now has `defined_by: Option<String>` field
  - `TaggedType` with no tag class now maps to `ContextSpecific` (was incorrectly mapping to `Universal`)
- **Codegen fixes** (`asn1c-codegen/src/builder.rs`, `asn1c-codegen/src/code_ast.rs`):
  - `Field` struct has `order: usize` field for tracking original ASN.1 field position
  - SEQUENCE/SET fields are reordered: non-default fields first, default/optional fields last (Python dataclass compatibility)
  - `Declaration::ListType` variant added for SEQUENCE OF / SET OF type alias classes
  - `ber_info_for_type` correctly generates BER info for list element types
  - `CodeAstBuilder` now holds a type map (`HashMap<String, AsnType>`) for resolving `ReferencedType` during BER info generation
  - `resolve_type()` resolves referenced types through the type map with cycle detection
  - `SequenceOf`/`SetOf` BER info now preserves `referenced_type` name when resolving from `ReferencedType`
- **Template migration**: Migrated from Minijinja to **Askama** v0.16.0 (compile-time templates). Old `.j2` files replaced by `.txt` files in `asn1c-codegen/templates/python/`. Template logic now uses type-safe context structs in `python.rs` with `#[derive(Template)]`.
- **Resolver fix** (`asn1c-ir/src/resolver.rs`):
  - SequenceOf/SetOf element types are NOT resolved inline (preserves `ReferencedType` name for codegen)
- **Template fixes** (Milestone 6):
  - `struct.txt`: Fixed missing `_iv = DerEncoder()` for optional integer fields in `encode_der`
  - `struct.txt`: Added "list" encoding case for nested list types (SEQUENCE OF / SET OF within SEQUENCE/SET)
  - `list_type.txt`: Changed `decode_der` to do actual decoding (was delegating to `decode_ber`), changed `decode_ber` to delegate to `decode_der`
  - `list_type.txt`: Changed `encode_ber` to call `encode_der()` on referenced/constructed element types (was calling `encode_ber` which lacked outer wrapper)
  - `choice.txt`: Fixed `decode_der` to reconstruct full TLV for constructed/choice alternatives (was passing raw value bytes)

#### Integration Tests
- `tests/integration/x509_certificate.asn1` — RFC 5912-based X.509 simplified spec (15 types)
- `tests/integration/test_x509_roundtrip.py` — 9 X.509 roundtrip tests ✅ ALL PASS
- `tests/integration/ldap_protocol.asn1` — RFC 4511-based LDAP simplified spec
- `tests/integration/test_ldap_roundtrip.py` — 9 LDAP roundtrip tests ✅ ALL PASS

#### Test Status
- All 12 existing roundtrip tests PASS
- All 9 X.509 roundtrip tests PASS
- All 9 LDAP roundtrip tests PASS
- All 9 explicit CHOICE roundtrip tests PASS
- All 9 indefinite BER roundtrip tests PASS
- ANY DEFINED BY roundtrip verified
- **Total: 48 roundtrip tests + 9 indefinite BER tests passing**

#### Known Limitations
- `decode_ber` not generated for non-CHOICE types (only `decode_der` exists — DER is the target)
- Inline CHOICE as SEQUENCE field: type annotation becomes `Any` instead of CHOICE class name (referenced CHOICE types work correctly)
- Nested SEQUENCE OF with SEQUENCE elements: list encoding uses inner content without per-element TLV wrapper (pre-existing issue, not specific to new features)

#### BLOCKING ISSUE: struct.j2 Template Corruption — RESOLVED ✅
The template nesting issue was resolved by **migrating from Minijinja to Askama** (see Template Engine section above). The old `struct.j2` with corrupted nesting has been replaced by `struct.txt` using Askama's compile-time templates with proper type-safe context structs.

### Template Engine: Askama (v0.16.0)

Templates use **Askama** (compile-time, derive-based). See the **`askama`** skill. **The `minijinja` skill is obsolete.**

**Key files:**
- `asn1c-codegen/src/python.rs` — Python renderer with Askama `#[derive(Template)]` structs
- `asn1c-codegen/templates/python/` — Askama templates (.txt extension = no escaping)
- `asn1c-codegen/askama.toml` — Askama configuration

**Key patterns:**
- Context structs with `has_*` booleans for optional fields (Askama can't `{% if opt %}`)
- `list_element_ber` uses `Vec<T>` (0 or 1 elements) instead of `Option<Box<T>>`
- Template syntax: `{% else if %}` or `{% elif %}`, `||`/`&&`/`!` in conditions
- **Never** replace `or`/`and`/`not` → `||`/`&&`/`!` globally — only inside `{% %}` blocks
- `{% if !x.is_empty() %}` for strings, `{% if field.has_ber %}` for optional structs
- Sort in Rust before passing to template (Askama doesn't support `|sort(attribute='x')`)

### Milestone 7+: Backlog

**Milestone 7: CHOICE Enhancements ✅ COMPLETE**
- Explicitly tagged CHOICE alternatives now supported (`[0] EXPLICIT Type`)
- Implicit tagging in CHOICE alternatives supported (`[0] IMPLICIT Type`)
- Mixed CHOICE (some explicit, some inherent) works correctly
- `BerFieldInfo` extended with `tagging_mode`, `inherent_tag_class`, `inherent_tag_number`
- `choice.txt` template handles all three tagging modes (inherent/explicit/implicit)
- New integration test: `tests/explicit_choice.asn1` + `tests/test_explicit_choice.py` (9 tests)

**Milestone 8: Indefinite Length BER Support ✅ COMPLETE**
- Runtime (`ber.py`): Added `write_eoc()`, `write_tlv_indefinite()`, `is_eoc()`, `read_eoc()`, `read_constructed_indefinite()`
- `struct.txt`: Added `encode_ber_indefinite()` and `decode_ber_indefinite()` for SEQUENCE/SET types
- `choice.txt`: Added `encode_ber_indefinite()` and `decode_ber_indefinite()` for CHOICE types (also restored missing `decode_ber`)
- `list_type.txt`: Added `encode_ber_indefinite()` and `decode_ber_indefinite()` for SEQUENCE OF/SET OF types
- Tests: `test_indefinite_ber.py` — 9 indefinite BER roundtrip tests ✅ ALL PASS

**Milestone 9: ANY DEFINED BY Full Support ✅ COMPLETE**
- Grammar (`asn1.par`): `OpenType: 'ANY'^ 'DEFINED'^ 'BY'^ Identifier` (captures identifier after BY)
- Parser (`grammar.rs`): `open_type` callback extracts identifier AND pops `str_stack` (prevents component name corruption)
- Code AST (`code_ast.rs`): Added `defined_by: Option<String>` to `BerFieldInfo`
- Builder (`builder.rs`): Propagates `defined_by` through `ber_info_for_type()` for `AsnType::OpenType`
- Type annotation: `OpenType` fields now generate `bytes` type (raw TLV storage)
- Template (`struct.txt`): Added "any" encoding for encode_ber, encode_ber_indefinite, encode_der, decode_der, decode_ber_indefinite with full TLV reconstruction
- Test: `tests/any_defined_by.asn1` + verified roundtrip (INTEGER 42 as raw TLV in ANY DEFINED BY field)

**Remaining Backlog:**
- [ ] SNMP integration test (RFC 3416 based)
- [ ] PER, OER, XER, JER encoding backends
- [ ] Rust, TypeScript, C, Go backends
- [ ] CHOICE as field within SEQUENCE (works for referenced CHOICE types; inline CHOICE type annotation needs improvement)

**Load the `parol-parser` skill** before working on parser/grammar changes. **Load the `rust-best-practices` skill** before writing or reviewing Rust code. **Load the `askama` skill** before working on templates.

## Python Runtime

Located at `asn1c-runtime-python/` — **NOT a pip package**. It ships as a directory copied alongside generated code. Generated Python imports via `from asn1c_runtime import ...`.

Files:
- `__init__.py` — Exports: `AsnType`, `Tag`, `TagClass`, `BerEncoder`, `BerDecoder`, `DerEncoder`, `DerDecoder`, `BitString`, `ObjectIdentifier`
- `ber.py` — BER TLV encoder/decoder primitives
- `der.py` — DER (canonical BER) encoder/decoder
- `types.py` — `BitString`, `ObjectIdentifier`, `AsnAny`
- `errors.py` — `AsnError` hierarchy

Requires Python 3.9+ (uses `from __future__ import annotations`).

## Generated Python Example

```python
from asn1c_runtime import AsnType, Tag, TagClass, BerEncoder, BerDecoder, DerEncoder, DerDecoder
from dataclasses import dataclass

@dataclass
class Person(AsnType):
    name: str
    age: int
    active: bool

    def encode_ber(self) -> bytes: ...
    def encode_der(self) -> bytes: ...
    @classmethod
    def decode_der(cls, data: bytes) -> "Person": ...
```

## Review Backlog (Session: 2026-05-31 Proactive Maintenance)

*Findings from comprehensive codebase review across all crates. Work through items in severity order (Serious first, then Design, then Minor). Mark [x] when complete.*

### 🔴 Serious Issues

#### asn1c-parser
- [x] **R1: Broken `{ ValueItems }` collection** — `grammar.rs:815-818`. Values pushed by `value_item` callbacks are discarded; branch creates `Vec::new()`. Any ASN.1 value list silently becomes empty. **Fixed**: Added `named_value_stack` to collect `NamedValue` items; `LBraceValueItemsRBrace` drains it. Also fixed `value_item` to pop identifiers from `str_stack` (was causing downstream parse corruption where field names became type names). Tests added in `asn1c-parser/src/lib.rs`.
- [x] **R2: Broken `import_symbol` fallback** — `grammar.rs:314`. For keyword variants it does `format!("{:?}", arg.identifier_or_keyword)`, producing debug strings. Import lists corrupted. Match all `IdentifierOrKeyword` variants like `export_symbol` does.
- [ ] **R3: All spans hardcoded to `0..0`** — throughout `grammar.rs`. Every AST node uses `SourceSpan::from(0..0)`. Real `Token<'t>.location()` data is available but never used. All error messages point to line 1, column 0.
- [ ] **R4: ~30 `.unwrap()` calls on stack operations** — throughout `grammar.rs`. Any grammar mismatch panics instead of producing a parse error. Replace with `.ok_or_else(|| anyhow!(...))`.
- [ ] **R5: Hex string parsing silently swallows errors** — `grammar.rs:97, 103`. Invalid hex digits become `0` via `unwrap_or(0)`. Should return parse error.
- [ ] **R41: `IdentifierOrKeyword` doesn't include `Reference`** — `asn1.par:154-170`. Import/export symbols only accept lowercase `Identifier` (`/[a-z].../`) or ~75 keyword literals. Real ASN.1 type names (uppercase, like `Person`, `X509Certificate`) are `Reference` tokens (`/[A-Z].../`) and **cannot appear in import/export positions**. `IdentifierOrKeyword` needs `Reference` as an alternative.

#### asn1c-ir
- [ ] **R6: Silent error suppression in parameter conversion** — `from_ast.rs:101`. `asn_type_to_ir(t).unwrap_or(ir::AsnType::Any)` silently converts malformed parameter types to `Any`.
- [ ] **R7: Invalid tag number silently becomes 0** — `from_ast.rs:174`. Negative or out-of-range tag numbers silently coerce to tag `0`.
- [ ] **R8: Enum value defaults to 0 instead of computing sequentially** — `from_ast.rs:209-214`. Missing enum values should be previous value + 1, not always `0`.
- [ ] **R9: No duplicate type/name validation** — entire crate. Two types with the same name silently coexist; `resolve_type` finds only the first one via `.find()`.
- [ ] **R10: Import existence not validated** — `resolver.rs:45-76`. A module can import `"NonExistentType"` and pass validation — the symbol is never checked to actually exist in the target module.

#### asn1c-codegen
- [ ] **R11: SET elements not sorted during `encode_der`** — `struct.txt`. DER requires SET elements in canonical TLV order. Template encodes fields in declaration order, not by encoded byte order. Re-encoding produces different bytes.
- [ ] **R12: DER time encoding uses `BerEncoder` instead of `DerEncoder`** — `struct.txt:787`. GeneralizedTime/UTCTime fields in `encode_der` use non-canonical BER encoder.
- [ ] **R13: `list_type.txt` `encode_der` delegates to `encode_ber`** — `list_type.txt:82-83`. SET OF elements should be sorted for DER; this bypasses canonicalization.
- [ ] **R14: String escaping incomplete** — `python.rs:135`. `ValueLiteral::String` escaping doesn't handle `\n`, `\t`, `\r`, or control characters. Produces invalid Python output.

#### asn1c-runtime-python
- [ ] **R15: Negative integer encoding broken** — `ber.py:56-64` and `der.py:32-37`. All negative integers from -127 to -1 encode as `[0xFF]` (decodes to -1). -128 encodes as `[0xFF]` instead of `[0x80]`. Missing `num_bytes.insert(0, temp & 0xFF)` after the while loop.
- [ ] **R16: Missing bounds checks in `read_set_elements`** — `der.py:147-165`. Long-form tag parsing does `content[pos]` without bounds check; truncated input raises `IndexError` instead of `TruncatedInputError`.

#### tests
- [ ] **R17: Integration tests not runnable from repo** — `test_x509_roundtrip.py`, `test_ldap_roundtrip.py`. Hardcoded `/tmp/asn1c-integration-test/` paths and imports from non-existent `.py` files. Tests only work after manual pre-generation.
- [ ] **R18: No test coverage for negative integers** — no test file exercises encoding/decoding of negative integers (the broken encoding in R15 would never be caught).

### 🟠 Design / Architecture Issues

#### asn1c-parser
- [ ] **R19: OID string marker protocol is fragile** — `grammar.rs:132-191`. OIDs serialized as comma-joined strings with `__oid_name__:`/`__oid_num__:` prefixes. Should use a dedicated stack.
- [ ] **R20: ASN.1 semantic decision in parser layer** — `grammar.rs:916`. Absent EXPORTS defaults to "ALL" in the parser; should be an IR-layer concern.
- [ ] **R21: Parameterized types unsupported despite AST definition** — `asn1.par:113` vs `ast.rs:194`. Grammar has `ReferencedType: Reference;` with no parameters.
- [ ] **R22: No constraint parsing** — `asn1.par`. Grammar has no constraint syntax. `INTEGER (0..255)`, `OCTET STRING (SIZE(1..100))` cannot be parsed.
- [ ] **R23: 15 stacks with no helper abstraction** — every callback repeats push/pop/reverse patterns.
- [ ] **R42: `reference()` callback pollutes `str_stack`** — `grammar.rs:71-73`. The generic `reference()` callback fires for **every** `Reference` token, pushing raw text. When a more specific callback (e.g., `module_reference`, `open_type`) handles the same non-terminal, two entries end up on the stack. Downstream callbacks like `import_item()` accumulated dead code (shadowed variables) to silently consume duplicates. Fix: either don't push in `reference()`, or use dedicated stacks per domain.

#### asn1c-ir
- [ ] **R24: `ConversionError` and `IrError` disconnected** — two separate error types with no `From` impl. Pipeline error handling is verbose and inconsistent.
- [ ] **R25: ObjectClass/Object/ObjectSet assignments silently dropped** — `from_ast.rs:37`. Wildcard match with no diagnostic.
- [ ] **R26: ~60 lines of duplicated field resolution logic** — `resolver.rs:132-194`. Sequence, Set, and Choice resolution arms are nearly identical.

#### asn1c-codegen
- [ ] **R27: Massive template duplication** — `struct.txt` (2014 lines), `choice.txt` (1576 lines). Four nearly-identical method blocks per template. ~5000+ lines of duplicated logic. Root cause of most consistency bugs.
- [ ] **R28: Stringly-typed encoding enum** — `BerFieldInfo.encoding` uses raw strings. Typos silently fall through to wrong encoding paths.
- [ ] **R29: `thiserror` dependency declared but never used** — `Cargo.toml:8`.
- [ ] **R30: Dead code** — `code_ast.rs`: `Function`, `TemplateRef`, `FunctionDecl`, `Constant` variants are never used.
- [ ] **R31: `render_function()` always bails** — `python.rs:331-333`. Should be removed along with `FunctionDecl` variant.

#### CLI
- [ ] **R32: `--encoding` argument parsed but never used** — `main.rs:27`. Generated code always includes both BER and DER methods regardless.
- [ ] **R33: `miette` and `num-bigint` dependencies declared but unused** — `asn1c/Cargo.toml`.
- [ ] **R34: `copy_dir` reimplementation** — `main.rs:153-166`. Doesn't handle symlinks or permissions.

### 🟡 Minor Issues
- [ ] **R35: Export "ALL" detection by string value** — `grammar.rs:299`. Treats keyword `ALL` and identifier `ALL` identically.
- [ ] **R36: `extension_default` callback is dead code** — `grammar.rs:127-130`.
- [ ] **R37: 4/6 `IrError` variants never used** — `error.rs`.
- [ ] **R38: `AsnAny` has no `__eq__` or `__repr__`** — `types.py:98-102`.
- [ ] **R39: `capitalize()` doesn't handle Unicode** — `builder.rs:35-41`.
- [ ] **R40: `BerContext.list_element_ber` uses `Vec` instead of `Option`** — `python.rs:81`.

### Original Backlog (Milestone 10+)
1. SNMP integration test (RFC 3416 based)
2. PER, OER, XER, JER encoding backends
3. Rust, TypeScript, C, Go backends
4. CHOICE as field within SEQUENCE (works for referenced CHOICE types; inline CHOICE type annotation needs improvement)

## Next Session Notes

**What's done:**
- **R1 Fixed: Broken `{ ValueItems }` collection** — Added `named_value_stack` to Grammar struct; `value_item` pushes `NamedValue` items; `LBraceValueItemsRBrace` drains and collects via `rev().collect()`. Also fixed `value_item` Identifier/ReferenceColonValue cases to pop from `str_stack` (was leaking identifiers and corrupting downstream parse). 2 Rust unit tests added. 18 Python roundtrip tests verified passing.
- **R2 Fixed: Broken `import_symbol` fallback** — Replaced `_ => format!("{:?}", ...)` with exhaustive match on all 76 `IdentifierOrKeyword` variants (matching `export_symbol` pattern). Also cleaned up a latent bug where the `reference()` callback's raw push duplicated the module reference on the stack (added `_raw_ref` pop with documenting comment). 2 Rust unit tests added (`test_import_keyword_symbols`, `test_import_multiple_keyword_symbols`). All 4 parser tests pass.
- Template engine migrated from Minijinja to Askama v0.16.0
- struct.j2 template nesting corruption resolved
- All 7 classes generate valid Python with encode_ber, encode_der, decode_der
- Askama skill verified and updated with corrections from source review
- test_roundtrip.py updated to use decode_der instead of decode_ber
- list_type.txt fixed: decode_ber delegates to decode_der, encode_ber calls encode_der on referenced types
- choice.txt fixed: decode_der reconstructs full TLV for constructed/choice alternatives
- builder.rs fixed: ReferencedType resolution through type map for correct BER encoding
- builder.rs fixed: SequenceOf/SetOf cases preserve referenced_type name
- struct.txt: Added "list" encoding case for nested list types
- X.509 integration tests: 9/9 passing
- LDAP integration tests: 9/9 passing
- CHOICE enhancements: explicitly tagged alternatives implemented
- `BerFieldInfo` extended with `tagging_mode`, `inherent_tag_class`, `inherent_tag_number`
- `choice.txt` template handles inherent/explicit/implicit tagging modes
- Explicit CHOICE integration tests: 9/9 passing
- **Indefinite length BER support: 9/9 tests passing**
  - Runtime: `write_eoc()`, `write_tlv_indefinite()`, `is_eoc()`, `read_eoc()`, `read_constructed_indefinite()`
  - Templates: `encode_ber_indefinite`/`decode_ber_indefinite` added to struct.txt, choice.txt, list_type.txt
- **ANY DEFINED BY full support: verified roundtrip**
  - Grammar: `OpenType` captures identifier after `BY`
  - Parser: `open_type` extracts identifier AND pops `str_stack` (fixes component name corruption)
  - Code AST: `BerFieldInfo` has `defined_by: Option<String>`
  - Builder: propagates `defined_by` through `ber_info_for_type()`
  - Template: "any" encoding generates bytes type with full TLV encode/decode
- **Total: 48 roundtrip tests + 9 indefinite BER tests passing**

**Known gaps:**
- `decode_ber` not generated for non-CHOICE types (only `decode_der` exists — DER is the target)
- Inline CHOICE as SEQUENCE field: type annotation becomes `Any` instead of CHOICE class name (referenced CHOICE types work correctly)
- Nested SEQUENCE OF with SEQUENCE elements: list encoding uses inner content without per-element TLV wrapper (pre-existing issue, not specific to new features)
- `defined_or` filter does NOT exist in Askama (use `assigned_or` or `is defined` check)
- `|linebreaks` family marks output as HTML-safe but does NOT escape input

**Backlog (Milestone 10+):**
1. SNMP integration test (RFC 3416 based)
2. PER, OER, XER, JER encoding backends
3. Rust, TypeScript, C, Go backends
4. CHOICE as field within SEQUENCE (works for referenced CHOICE types; inline CHOICE type annotation needs improvement)

**Key files modified for recent work:**
- `asn1c-runtime-python/ber.py` — Added indefinite length methods: `write_eoc()`, `write_tlv_indefinite()`, `is_eoc()`, `read_eoc()`, `read_constructed_indefinite()`
- `asn1c-parser/src/asn1.par` — `OpenType` grammar now captures `Identifier` after `BY`
- `asn1c-parser/src/grammar.rs` — `open_type` callback extracts `defined_by` and pops `str_stack`; added `decode_ber` to CHOICE
- `asn1c-codegen/src/code_ast.rs` — Added `defined_by: Option<String>` to `BerFieldInfo`
- `asn1c-codegen/src/builder.rs` — Propagates `defined_by`; split `OpenType`/`Any` match arms; OpenType maps to `bytes` type
- `asn1c-codegen/templates/python/struct.txt` — Added `encode_ber_indefinite`, `decode_ber_indefinite`, "any" encoding for encode/decode
- `asn1c-codegen/templates/python/choice.txt` — Added `encode_ber_indefinite`, `decode_ber_indefinite`, restored `decode_ber`
- `asn1c-codegen/templates/python/list_type.txt` — Added `encode_ber_indefinite`, `decode_ber_indefinite`
- `asn1c-codegen/templates/python/module_header.txt` — Added `InvalidLengthError` to imports
- `tests/test_indefinite_ber.py` — 9 indefinite BER roundtrip tests
- `tests/any_defined_by.asn1` — ANY DEFINED BY integration test
