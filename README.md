# asn1c — ASN.1 Compiler

An ASN.1 compiler written in Rust using the [Parol v4](https://github.com/jsinger67/parol) parser generator. The goal is to parse `.asn1` files (ITU-T X.680–X.683, 2021) and generate source code with encode/decode support in multiple target languages, starting with Python using BER and DER encoding rules (X.690, 2021).

## Current State

**Milestones 1–4 complete.** The full pipeline works: `.asn1` → parser → IR → codegen → Python with BER encode/decode. 12 roundtrip tests pass covering SEQUENCE, SET, CHOICE, SEQUENCE OF, SET OF, DEFAULT values, BIT STRING, OCTET STRING, and OID types.

**What is NOT yet implemented:**
- DER canonicalization (sorted SET elements, minimal integer encoding, strict validation)
- Indefinite length BER support
- CHOICE enhancements (explicitly tagged alternatives, CHOICE as field within SEQUENCE)
- PER, OER, XER, JER encoding backends
- Rust, TypeScript, C, Go backends

See the roadmap below for what's next.

## Quick Start

```bash
# Build everything
cargo build

# Compile an ASN.1 file to Python
cargo run -- -o output/ test.asn1

# Run the roundtrip tests
python3 test_roundtrip.py

# See CLI options
cargo run -- --help
```

## Architecture (Planned)

```
ASN.1 source (.asn1)
    ↓
[Parser] — Parol-generated LL(k) parser → AST        ← Milestone 2 ✅
    ↓
[Semantic Analyzer] → Language-agnostic IR            ← Milestone 3 ✅
    ↓
[Code AST Builder] → Language-agnostic Code AST       ← Milestone 4 ✅
    ↓
[Language Renderer] → Target language source          ← Milestone 4 ✅
    ↓
[Python Runtime] → BER/DER encode/decode              ← Milestones 5-6
```

### Crates

| Crate | What It Does | Status |
|---|---|---|
| `asn1c` | CLI binary with `clap` argument parsing | ✅ Built |
| `asn1c-parser` | Parol grammar (`.par`), build.rs, AST types | ✅ Grammar + parser generation working |
| `asn1c-ir` | IR data structures + type resolver | ✅ Data structures built, resolver wired with cycle detection |
| `asn1c-codegen` | IR → Code AST → Python renderer | ✅ Generates Python with BER encode/decode logic |
| `asn1c-runtime-python` | Pure Python stdlib-only runtime | ✅ TLV primitives, type-specific encode/decode via generated code |

## CLI Options

```
asn1c [OPTIONS] [INPUT]...

Arguments:
  [INPUT]...  ASN.1 module files to compile

Options:
  -o, --out-dir <DIR>        Output directory for generated code
  --lang <LANG>              Target language: python, rust, ... (default: python)
  --encoding <ENC>           Target encoding: ber, der (default: der)
  --emit-runtime             Also copy runtime library to output directory
  --runtime-dir <DIR>        Custom path to runtime library
  --print-ir                 Print IR instead of generating code
  --print-ast                Print AST instead of generating code
  --print-code-ast           Print Code AST instead of generating code
  -v, --verbose              Verbose output
  -q, --quiet                Suppress non-error output
```

## Grammar Coverage

The Parol grammar (`asn1c-parser/src/asn1.par`, 172 lines) covers the full ASN.1 spec:
- Module definitions (`DEFINITIONS ::= BEGIN ... END`)
- All built-in types (BOOLEAN, INTEGER, SEQUENCE, SET, CHOICE, ENUMERATED, BIT STRING, OCTET STRING, etc.)
- Tagged types, SEQUENCE OF, SET OF
- Constraints (value range, size, permitted alphabet)
- Information objects (X.681) and parameterization (X.683)
- Values for all types

The grammar compiles via Parol v4's LL(k) generator and is fully connected to the IR builder via `GrammarTrait` callbacks and `from_ast.rs`.

## Python Runtime

Located at `asn1c-runtime-python/` — **not a pip package**. It ships as a directory copied alongside generated code.

### What's Implemented
- `BerEncoder` / `BerDecoder` — TLV tag/length/content primitives, integer encoding
- `DerEncoder` / `DerDecoder` — BER subclass with definite-length enforcement
- `Tag`, `TagClass` — Tag representation with encode/decode
- `BitString`, `ObjectIdentifier` — Custom ASN.1 types
- `AsnError` hierarchy — Error types for decode failures
- Generated per-type encode/decode for SEQUENCE, CHOICE, ENUMERATED, SEQUENCE OF, SET OF
- Constructed encoding for nested types
- DEFAULT value handling (skip encoding when at default, apply on decode)

### What's NOT Implemented
- Indefinite length support
- DER canonicalization (sorted SET elements, minimal integer encoding)

### Runtime API

```python
from asn1c_runtime import (
    AsnType, Tag, TagClass,          # Core types
    BerEncoder, BerDecoder,          # BER primitives
    DerEncoder, DerDecoder,          # DER (canonical BER)
    BitString, ObjectIdentifier,     # Custom types
    AsnError,                        # Error hierarchy
)

# Low-level encoding
enc = BerEncoder()
enc.write_tag(TagClass.UNIVERSAL, 0x02)    # INTEGER tag
enc.write_integer(42)
der_bytes = enc.finish()                   # → b'\x02\x01*'

# Low-level decoding
dec = BerDecoder(der_bytes)
tag = dec.read_tag()       # (class, number, constructed)
length = dec.read_length() # int or None (indefinite)
value = dec.read_integer() # 42
```

## Development

```bash
cargo build                     # Build all crates
cargo build -p asn1c-parser     # Build parser (triggers Parol generation)
cargo run -- -o output/ test.asn1  # Compile an ASN.1 file
python3 test_roundtrip.py       # Run roundtrip tests
cargo run -- --help             # CLI help
```

### Project Structure

```
asn1c/
├── Cargo.toml                  # Workspace root
├── AGENTS.md                   # Agent instructions
├── asn1c/                      # CLI binary
├── asn1c-parser/               # Parol grammar + AST
│   └── src/
│       ├── asn1.par            # ASN.1 grammar (172 lines)
│       ├── build.rs            # Parol code generation + attribute stripping
│       ├── lib.rs              # Module includes for generated code
│       ├── grammar.rs          # User Grammar struct implementing GrammarTrait
│       ├── ast.rs              # Hand-written AST types
│       └── error.rs            # Error types
├── asn1c-ir/                   # Intermediate representation
│   └── src/
│       ├── ir.rs               # AsnModule, AsnType, Constraints, Values
│       └── resolver.rs         # Type resolution, import/export, circular refs
├── asn1c-codegen/              # Code generation
│   └── src/
│       ├── builder.rs          # IR → Code AST transformation
│       ├── python.rs           # Python renderer with minijinja templates
│       └── templates/python/   # Jinja2 templates for struct, choice, enum, type_alias, module_header
└── asn1c-runtime-python/       # Python runtime (not a crate)
    ├── __init__.py             # Exports + Tag, TagClass, AsnType
    ├── ber.py                  # BerEncoder, BerDecoder primitives
    ├── der.py                  # DerEncoder, DerDecoder (strict BER)
    ├── types.py                # BitString, ObjectIdentifier, AsnAny
    └── errors.py               # AsnError hierarchy
```

## Roadmap

| Milestone | Status | What Needs To Happen |
|---|---|---|
| 1: Skeleton | ✅ Done | All crates compile, CLI runs, grammar parses, runtime has TLV primitives |
| 2: Core Parser | ✅ Done | Parol grammar → AST → IR bridge via GrammarTrait + from_ast.rs |
| 3: Semantic Analysis | ✅ Done | Type resolution, import/export validation, circular reference detection |
| 4: Code Generation | ✅ Done | Full BER encode/decode templates for SEQUENCE, CHOICE, SEQUENCE OF, SET OF, DEFAULT values |
| 5: BER Enhancements | 🔜 Next | DER canonicalization, indefinite length support, CHOICE enhancements |
| 6: DER Runtime | Planned | Canonical encoding, strict validation, sorted SET elements |
| 7: Integration | Planned | End-to-end with real-world specs (X.509, LDAP, SNMP) |
| 8: More Languages | Future | Rust, TypeScript, C, Go backends |

## License

MIT
