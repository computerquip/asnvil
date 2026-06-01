# asnvil — ASN.1 Compiler

An ASN.1 compiler written in Rust using the [Parol v4](https://github.com/jsinger67/parol) parser generator. It parses `.asn1` files (ITU-T X.680–X.683, 2021) and generates source code with BER/DER encode/decode support, starting with Python.

## Quick Start

```bash
# Build everything
cargo build

# Compile an ASN.1 file to Python
cargo run -- -o output/ test.asn1

# Run all tests
just test-all

# See CLI options
cargo run -- --help
```

## Architecture

```
ASN.1 source (.asn1)
    ↓
[Parser] — Parol-generated LL(k) parser → AST
    ↓
[Semantic Analyzer] → Language-agnostic IR
    ↓
[Code AST Builder] → Code AST with EncodeStmt/DecodeStmt encoding operations
    ↓
[Python Renderer] → Target language source (Askama templates + renderer methods)
    ↓
[Python Runtime] → BER/DER encode/decode
```

**Code AST:** The Code AST (`code_ast.rs`) is the language-agnostic representation of generated code. It carries both metadata (`BerFieldInfo`) and encoding operations (`EncodeStmt`/`DecodeStmt`). Each field knows exactly how to encode and decode itself. `TypeRef`/`BuiltinType` are fully language-agnostic — `PythonRenderer` maps `BuiltinType::Integer` → `"int"`, a future `RustRenderer` would map it → `"i64"`, etc.

### Crates

| Crate | What It Does |
|---|---|
| `asnvil` | CLI binary with `clap` argument parsing |
| `asnvil-parser` | Parol grammar (`.par`), build.rs, AST types |
| `asnvil-ir` | IR data structures + type resolver |
| `asnvil-codegen` | IR → Code AST (with `EncodeStmt`/`DecodeStmt`) → Python renderer |
| `asnvil-runtime-python` | Pure Python stdlib-only runtime (shipped as directory, not a pip package) |

## CLI Options

```
asnvil [OPTIONS] [INPUT]...

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

## Test Infrastructure

| Layer | Tests | Status |
|---|---|---|
| asnvil-parser | 9 Rust unit tests | ✅ |
| asnvil-ir | 14 Rust unit tests | ✅ |
| asnvil-codegen | 12 Rust unit tests | ✅ |
| asnvil (CLI) | 13 Rust unit tests | ✅ |
| Python runtime | 55 unit tests | ✅ |
| Integration | 5 suites, 41 roundtrip tests | ✅ |

**Total: 103 Rust tests + 96 Python tests**

### Running Tests

```bash
just test-rust          # All Rust unit tests
just test-python        # Python runtime unit tests
just test-integration   # Integration tests (compile + roundtrip)
just test-all           # Everything
```

## Python Runtime

Located at `asnvil-runtime-python/` — ships as a directory alongside generated code. Generated Python imports via `from asnvil_runtime import ...`.

### Implemented

- `BerEncoder` / `BerDecoder` — TLV tag/length/content primitives
- `DerEncoder` / `DerDecoder` — DER canonicalization (minimal encoding, sorted SET elements, strict validation)
- Indefinite length BER support (`encode_ber_indefinite` / `decode_ber_indefinite`)
- `ANY DEFINED BY` support (raw TLV storage)
- CHOICE with explicit/implicit/inherent tagging
- `Tag`, `TagClass`, `BitString`, `ObjectIdentifier`, `AsnAny`
- `AsnError` hierarchy for decode failures
- Generated per-type encode/decode for SEQUENCE, SET, CHOICE, ENUMERATED, SEQUENCE OF, SET OF
- DEFAULT value handling

### Runtime API

```python
from asnvil_runtime import (
    AsnType, Tag, TagClass,
    BerEncoder, BerDecoder,
    DerEncoder, DerDecoder,
    BitString, ObjectIdentifier,
    AsnError,
)

# Low-level encoding
enc = BerEncoder()
enc.write_tag(TagClass.UNIVERSAL, 2)
enc.write_integer(42)
der_bytes = enc.finish()  # → b'\x02\x01*'

# Low-level decoding
dec = BerDecoder(der_bytes)
tag = dec.read_tag()       # (class, number, constructed)
value = dec.read_integer() # 42
```

## Development

```bash
cargo build                     # Build all crates
cargo build -p asnvil-parser     # Build parser (triggers Parol generation)
cargo run -- -o output/ test.asn1  # Compile an ASN.1 file
just test-all                   # Run all tests
cargo run -- --help             # CLI help
```

## Roadmap

| Milestone | Status | Description |
|---|---|---|
| 1: Skeleton | ✅ Done | Workspace, CLI, grammar, IR structures, Python runtime |
| 2: Core Parser | ✅ Done | GrammarTrait callbacks, AST → IR bridge |
| 3: Semantic Analysis | ✅ Done | Type resolution, import/export validation |
| 4: Code Generation | ✅ Done | BER encode/decode templates for all type variants |
| 5: DER Canonicalization | ✅ Done | Sorted SET elements, minimal encoding, strict validation |
| 6: Integration Tests | ✅ Done | X.509, LDAP roundtrip tests, self-contained runner |
| 7: CHOICE + Indefinite + ANY DEFINED BY | ✅ Done | Explicit tagging, indefinite BER, raw TLV fields |
| 8: Test Infrastructure | ✅ Done | 100+ unit tests, integration runner, justfile |
| R27b: Encoding Logic in Code AST | ✅ Done | EncodeStmt/DecodeStmt in Code AST, TypeRef language-agnostic, ~2000 lines of template encoding dispatch moved to Rust |
| 9: SNMP Integration | Planned | RFC 3416-based integration test |
| 10: PER/OER/XER/JER | Future | Additional encoding backends |
| 11: More Languages | Future | Rust, TypeScript, C, Go backends |

## License

MIT
