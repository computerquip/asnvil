# AGENTS.md — asnvil Project

## Project Overview

**asnvil** is an ASN.1 compiler written in Rust using the Parol v4 parser generator. It parses `.asn1` files (ITU-T X.680–X.683, 2021) and generates source code with encode/decode support in multiple target languages (Python first, Rust/TS/C/Go planned).

## Architecture

```
ASN.1 source (.asn1)
    ↓
[asnvil-parser] — Parol LL(k) parser → AST
    ↓
[asnvil-ir] — Semantic analyzer → Language-agnostic IR
    ↓
[asnvil-codegen] — Code AST construction + per-language renderers
    ↓
[Language Renderer] — Target language source (Python first)
    ↓
[asnvil-runtime-python] — Pure stdlib Python runtime (ships alongside generated code)
```

## Crates

| Crate | Purpose |
|---|---|
| `asnvil` | CLI binary (`asnvil <file.asn1> -o output/`) |
| `asnvil-parser` | Parol grammar (`asn1.par`), build.rs generation, AST types |
| `asnvil-ir` | Intermediate representation (resolved types, constraints, values) |
| `asnvil-codegen` | Code AST builder + Python renderer with **Askama** templates |
| `asnvil-runtime-python/` | Pure Python runtime (NOT a pip package, ships as directory) |

## Key Files

| File | What It Is |
|---|---|
| `asnvil-parser/src/asn1.par` | Full ASN.1 grammar (172 lines, X.680–X.683) |
| `asnvil-parser/build.rs` | Parol code generation + inner-attribute stripping |
| `asnvil-parser/src/lib.rs` | Module includes for generated parser/trait/scanner |
| `asnvil-parser/src/grammar.rs` | User-defined `Grammar<'t>` implementing `GrammarTrait` |
| `asnvil-parser/src/ast.rs` | Hand-written AST types for parse tree |
| `asnvil-ir/src/ir.rs` | IR data structures (AsnModule, AsnType, etc.) |
| `asnvil-ir/src/resolver.rs` | Type resolution, import/export, circular ref detection |
| `asnvil-codegen/src/builder.rs` | IR → Code AST transformation |
| `asnvil-codegen/src/python.rs` | Python renderer with **Askama** derive-based templates |
| `asnvil-codegen/templates/python/` | **Askama** templates (.txt): struct, choice, enum, type_alias, module_header, list_type |

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
cargo build -p asnvil-parser    # Build parser only (triggers Parol generation)
cargo run -- -o output/ test.asn1    # Compile an ASN.1 file
cargo run -- --help            # CLI help
cargo run --example demo       # Run full pipeline demo (IR → codegen → DER test)
cargo test --workspace         # All Rust unit tests (48 tests)
just test-rust                 # Alias for cargo test --workspace
just test-python               # Python runtime unit tests (55 tests)
just test-integration          # Self-contained integration test runner
just test-all                  # Run all tests (Rust + Python + integration)
```

## Principles

- **Fix warnings at the source** — Never suppress or work around compiler/clippy warnings. Understand the warning and fix the actual code. No `#[allow(...)]` unless the warning is from generated code (e.g. Parol output).

## Testing

**Always run tests through the Justfile** for consistency:
```bash
just test-rust          # 48 Rust unit tests
just test-python        # 55 Python runtime tests
just test-integration   # 5 integration suites (41 tests)
just test-all           # Everything
```

**How runtime imports work:** `conftest.py` uses importlib to load `asnvil-runtime-python/` as the `asnvil_runtime` package. No symlink is needed — Python can't import directories with hyphens, so the conftest manually registers the module. Do not create `asnvil_runtime` symlinks.

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
- `asnvil-ir/src/from_ast.rs` converts parser AST → IR
- CLI in `main.rs` runs real parse → AST → IR pipeline
- All 20+ ASN.1 types parsed (Sequence, Set, Choice, Enumerated, BitString, Tagged, etc.)

### Milestone 3: Semantic Analysis ✅
- `resolver.rs` (334 lines) with type resolution, import/export validation, circular ref detection
- Recursive resolution for ReferencedType, Sequence, Set, Choice, SequenceOf, SetOf, Tagged, ConstrainedType
- Cross-module reference support with module context
- Minor gap: `constraint_to_ir()` returns empty constraints (stubbed but IR structures defined)

### Milestone 4: Code Generation ✅
- Full IR → Code AST → Python pipeline with BER/DER encode/decode logic
- `builder.rs` (1008 lines) handles all type variants with `ber_info_for_type()`
- `struct.txt` template generates per-field encode/decode
- `choice.txt` template generates tagged union pattern with try/except fallback for constructed types
- SEQUENCE OF / SET OF list encoding with type-aware element TLV wrapping
- DEFAULT value support (extracted from parser, converted via `ValueLiteral`, rendered as Python defaults)
- BitString, ObjectIdentifier, AsnError runtime imports
- Templates use **Askama** (compile-time, derive-based) — migrated from Minijinja

### Milestone 5: DER Canonicalization ✅
- Complete DER encoder with strict validation (`DerEncoder` with minimal integer/length encoding)
- Complete DER decoder with canonicalization checks (`DerDecoder` validates minimal encoding, rejects indefinite length)
- SET element sorting by TLV for DER canonicalization (lexicographic order of encoded bytes)
- Generated Python includes `encode_der()` / `decode_der()` methods alongside BER methods
- `DerEncoder` enforces: no indefinite length, minimal integer encoding, definite length only
- `DerDecoder` enforces: no indefinite length, minimal integer validation, boolean 0x00/0xFF validation
- `read_set_elements()` validates SET elements are in canonical DER order
- `sort_set_tlv()` sorts SET elements lexicographically by their full TLV encoding

### Milestone 6: Integration Tests + RFC 5912 Support ✅

#### Completed
- **Grammar fixes** (`asnvil-parser/src/asn1.par`):
  - `TaggedType` now supports bracket notation `[0]`, `[1]` for context-specific tags
  - `'DEFINED'` added to `IdentifierOrKeyword` keyword list
  - `OpenType` moved before `AnyType` in Type alternatives (resolves ANY/ANY DEFINED BY ambiguity)
- **Parser fixes** (`asnvil-parser/src/grammar.rs`):
  - `open_type` callback handles new grammar structure (no lifetime param)
  - `value` callback now pops from `str_stack` for Identifier/Reference cases (fixes DEFAULT value reference pollution)
- **AST/IR fixes** (`asnvil-ir/src/from_ast.rs`, `asnvil-ir/src/ir.rs`):
  - `OpenType` variant now has `defined_by: Option<String>` field
  - `TaggedType` with no tag class now maps to `ContextSpecific` (was incorrectly mapping to `Universal`)
- **Codegen fixes** (`asnvil-codegen/src/builder.rs`, `asnvil-codegen/src/code_ast.rs`):
  - `Field` struct has `order: usize` field for tracking original ASN.1 field position
  - SEQUENCE/SET fields are reordered: non-default fields first, default/optional fields last (Python dataclass compatibility)
  - `Declaration::ListType` variant added for SEQUENCE OF / SET OF type alias classes
  - `ber_info_for_type` correctly generates BER info for list element types
  - `CodeAstBuilder` now holds a type map (`HashMap<String, AsnType>`) for resolving `ReferencedType` during BER info generation
  - `resolve_type()` resolves referenced types through the type map with cycle detection
  - `SequenceOf`/`SetOf` BER info now preserves `referenced_type` name when resolving from `ReferencedType`
- **Template migration**: Migrated from Minijinja to **Askama** v0.16.0 (compile-time templates). Old `.j2` files replaced by `.txt` files in `asnvil-codegen/templates/python/`. Template logic now uses type-safe context structs in `python.rs` with `#[derive(Template)]`.
- **Resolver fix** (`asnvil-ir/src/resolver.rs`):
  - SequenceOf/SetOf element types are NOT resolved inline (preserves `ReferencedType` name for codegen)

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

### Template Engine: Askama (v0.16.0)

Templates use **Askama** (compile-time, derive-based). See the **`askama`** skill. **The `minijinja` skill is obsolete.**

**Key files:**
- `asnvil-codegen/src/python.rs` — Python renderer with Askama `#[derive(Template)]` structs
- `asnvil-codegen/templates/python/` — Askama templates (.txt extension = no escaping)
- `asnvil-codegen/askama.toml` — Askama configuration

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

Located at `asnvil-runtime-python/` — **NOT a pip package**. It ships as a directory copied alongside generated code. Generated Python imports via `from asnvil_runtime import ...`.

Files:
- `__init__.py` — Exports: `AsnType`, `Tag`, `TagClass`, `BerEncoder`, `BerDecoder`, `DerEncoder`, `DerDecoder`, `BitString`, `ObjectIdentifier`
- `ber.py` — BER TLV encoder/decoder primitives
- `der.py` — DER (canonical BER) encoder/decoder
- `types.py` — `BitString`, `ObjectIdentifier`, `AsnAny`
- `errors.py` — `AsnError` hierarchy

Requires Python 3.9+ (uses `from __future__ import annotations`).

## Generated Python Example

```python
from asnvil_runtime import AsnType, Tag, TagClass, BerEncoder, BerDecoder, DerEncoder, DerDecoder
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

#### asnvil-parser
- [x] **R1: Broken `{ ValueItems }` collection** — `grammar.rs:815-818`. Values pushed by `value_item` callbacks are discarded; branch creates `Vec::new()`. Any ASN.1 value list silently becomes empty. **Fixed**: Added `named_value_stack` to collect `NamedValue` items; `LBraceValueItemsRBrace` drains it. Also fixed `value_item` to pop identifiers from `str_stack` (was causing downstream parse corruption where field names became type names). Tests added in `asnvil-parser/src/lib.rs`.
- [x] **R2: Broken `import_symbol` fallback** — `grammar.rs:314`. For keyword variants it does `format!("{:?}", arg.identifier_or_keyword)`, producing debug strings. Import lists corrupted. Match all `IdentifierOrKeyword` variants like `export_symbol` does.
 - [x] **R3: All spans hardcoded to `0..0`** — throughout `grammar.rs`. **Fixed**: All AST nodes now extract real `SourceSpan` from token locations instead of `0..0`. Added `Spanned` impl for `AsnType`.
- [x] **R4: ~30 `.unwrap()` calls on stack operations** — throughout `grammar.rs`. Any grammar mismatch panics instead of producing a parse error. Replace with `.ok_or_else(|| anyhow!(...))`.
 - [x] **R5: Hex string parsing silently swallows errors** — `grammar.rs:97, 103`. Invalid hex digits become `0` via `unwrap_or(0)`. Should return parse error. **Fixed**: Replaced `.unwrap_or(0)` with `.map_err()` returning `parol_runtime::ParolError::UserError`. Also fixed a latent bug: slice was `text[1..text.len()-1]` which left a trailing `'` in the hex data; corrected to `text[1..text.len()-2]` to strip both `'` and `H` suffix. 3 tests added: valid hex string, odd-length hex string (zero-padding), and invalid hex string (verifies error is returned).
 - [x] **R15: Negative integer encoding broken** — `ber.py:56-64` and `der.py:32-37`. Missing `num_bytes.insert(0, temp & 0xFF)` after the while loop. **Fixed**: Added the missing line to both `ber.py` and `der.py`. Also fixed `DerEncoder.write_boolean` tag class (was APPLICATION, should be UNIVERSAL). 55 runtime tests cover this.
 - [x] **R16: Missing bounds checks in `read_set_elements`** — `der.py:147-165`. Long-form tag parsing does `content[pos]` without bounds check; truncated input raises `IndexError` instead of `TruncatedInputError`. **Fixed**: Added bounds checks before each `content[pos]` access in the tag/length parsing loops.
 - [x] **R17: Integration tests not runnable from repo** — `test_x509_roundtrip.py`, `test_ldap_roundtrip.py`. Hardcoded `/tmp/asnvil-integration-test/` paths and imports from non-existent `.py` files. Tests only work after manual pre-generation. **Fixed**: Created `tests/run_integration.py` self-contained runner that compiles ASN.1, copies runtime, then runs pytest. Removed hardcoded `/tmp/` paths from X.509 and LDAP tests. Moved `test_indefinite_ber.py` to `tests/`. Added `test_any_defined_by.py`. Created pytest fixtures in `tests/conftest.py`.
 - [x] **R18: No test coverage for negative integers** — no test file exercises encoding/decoding of negative integers. **Fixed**: Covered by 55 runtime tests in `tests/test_runtime.py`.
 - [x] **R41: `IdentifierOrKeyword` doesn't include `Reference`** — `asn1.par:154-170`. Import/export symbols now accept uppercase type names (`Person`, `X509Certificate`). Also fixed R42 `reference()` callback stack pollution: `export_symbol` and `import_symbol` pop the duplicate entry pushed by `reference()` before extracting the name. 2 tests added.

#### asnvil-ir
- [ ] **R6: Silent error suppression in parameter conversion** — `from_ast.rs:101`. `asn_type_to_ir(t).unwrap_or(ir::AsnType::Any)` silently converts malformed parameter types to `Any`.
- [ ] **R7: Invalid tag number silently becomes 0** — `from_ast.rs:174`. Negative or out-of-range tag numbers silently coerce to tag `0`.
- [ ] **R8: Enum value defaults to 0 instead of computing sequentially** — `from_ast.rs:209-214`. Missing enum values should be previous value + 1, not always `0`.
- [ ] **R9: No duplicate type/name validation** — entire crate. Two types with the same name silently coexist; `resolve_type` finds only the first one via `.find()`.
- [ ] **R10: Import existence not validated** — `resolver.rs:45-76`. A module can import `"NonExistentType"` and pass validation — the symbol is never checked to actually exist in the target module.

#### asnvil-codegen
- [ ] **R11: SET elements not sorted during `encode_der`** — `struct.txt`. DER requires SET elements in canonical TLV order. Template encodes fields in declaration order, not by encoded byte order. Re-encoding produces different bytes.
- [ ] **R12: DER time encoding uses `BerEncoder` instead of `DerEncoder`** — `struct.txt:787`. GeneralizedTime/UTCTime fields in `encode_der` use non-canonical BER encoder.
- [ ] **R13: `list_type.txt` `encode_der` delegates to `encode_ber`** — `list_type.txt:82-83`. SET OF elements should be sorted for DER; this bypasses canonicalization.
- [ ] **R14: String escaping incomplete** — `python.rs:135`. `ValueLiteral::String` escaping doesn't handle `\n`, `\t`, `\r`, or control characters. Produces invalid Python output.

#### asnvil-runtime-python
- [x] **R15: Negative integer encoding broken** — Fixed (see asnvil-parser section above)
- [x] **R16: Missing bounds checks in `read_set_elements`** — Fixed (see asnvil-parser section above)

#### tests
- [x] **R17: Integration tests not runnable from repo** — Fixed (see asnvil-parser section above)
- [x] **R18: No test coverage for negative integers** — Fixed (see asnvil-parser section above)

### 🟠 Design / Architecture Issues

#### asnvil-parser
- [ ] **R19: OID string marker protocol is fragile** — `grammar.rs:132-191`. OIDs serialized as comma-joined strings with `__oid_name__:`/`__oid_num__:` prefixes. Should use a dedicated stack.
- [ ] **R20: ASN.1 semantic decision in parser layer** — `grammar.rs:916`. Absent EXPORTS defaults to "ALL" in the parser; should be an IR-layer concern.
- [ ] **R21: Parameterized types unsupported despite AST definition** — `asn1.par:113` vs `ast.rs:194`. Grammar has `ReferencedType: Reference;` with no parameters.
- [ ] **R22: No constraint parsing** — `asn1.par`. Grammar has no constraint syntax. `INTEGER (0..255)`, `OCTET STRING (SIZE(1..100))` cannot be parsed.
- [ ] **R23: 15 stacks with no helper abstraction** — every callback repeats push/pop/reverse patterns.
 - [x] **R42: `reference()` callback pollutes `str_stack`** — `grammar.rs:71-73`. The generic `reference()` callback fires for **every** `Reference` token, pushing raw text. When a more specific callback (e.g., `module_reference`, `open_type`) handles the same non-terminal, two entries end up on the stack. Fixed as part of R41: `export_symbol` and `import_symbol` now pop the duplicate entry pushed by `reference()` before extracting the name.

#### asnvil-ir
- [ ] **R24: `ConversionError` and `IrError` disconnected** — two separate error types with no `From` impl. Pipeline error handling is verbose and inconsistent.
- [ ] **R25: ObjectClass/Object/ObjectSet assignments silently dropped** — `from_ast.rs:37`. Wildcard match with no diagnostic.
- [ ] **R26: ~60 lines of duplicated field resolution logic** — `resolver.rs:132-194`. Sequence, Set, and Choice resolution arms are nearly identical.

#### asnvil-codegen
- [ ] **R27: Massive template duplication** — `struct.txt` (2014 lines), `choice.txt` (1576 lines). Four nearly-identical method blocks per template. ~5000+ lines of duplicated logic. Root cause of most consistency bugs.
- [ ] **R28: Stringly-typed encoding enum** — `BerFieldInfo.encoding` uses raw strings. Typos silently fall through to wrong encoding paths.
- [ ] **R29: `thiserror` dependency declared but never used** — `Cargo.toml:8`.
- [ ] **R30: Dead code** — `code_ast.rs`: `Function`, `TemplateRef`, `FunctionDecl`, `Constant` variants are never used.
- [ ] **R31: `render_function()` always bails** — `python.rs:331-333`. Should be removed along with `FunctionDecl` variant.

#### CLI
- [ ] **R32: `--encoding` argument parsed but never used** — `main.rs:27`. Generated code always includes both BER and DER methods regardless.
- [ ] **R33: `miette` and `num-bigint` dependencies declared but unused** — `asnvil/Cargo.toml`.
- [ ] **R34: `copy_dir` reimplementation** — `main.rs:153-166`. Doesn't handle symlinks or permissions.

### 🟡 Minor Issues
- [ ] **R35: Export "ALL" detection by string value** — `grammar.rs:299`. Treats keyword `ALL` and identifier `ALL` identically.
- [ ] **R36: `extension_default` callback is dead code** — `grammar.rs:127-130`.
- [ ] **R37: 4/6 `IrError` variants never used** — `error.rs`.
- [ ] **R38: `AsnAny` has no `__eq__` or `__repr__`** — `types.py:98-102`.
- [ ] **R39: `capitalize()` doesn't handle Unicode** — `builder.rs:35-41`.
- [ ] **R40: `BerContext.list_element_ber` uses `Vec` instead of `Option`** — `python.rs:81`.

### Remaining Milestones
1. SNMP integration test (RFC 3416 based)
2. PER, OER, XER, JER encoding backends
3. Rust, TypeScript, C, Go backends
4. Inline CHOICE as SEQUENCE field (type annotation improvement)

### Current Test Counts
- Rust: 48 tests (9 parser + 14 IR + 12 codegen + 13 CLI)
- Python: 55 runtime unit tests
- Integration: 5 suites, 41 roundtrip tests
- **Total: 103 Rust + 96 Python tests**
