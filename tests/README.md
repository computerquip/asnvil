# ASN.1 Compiler Test Framework

Welcome to the `asnvil` test suite! This document explains how our testing architecture works, how to run tests, and how to add new ones. 

Our testing philosophy is built on three core principles:
1. **Co-location**: Everything needed to test a specific feature (schemas, payloads, and test scripts) lives together in a single folder.
2. **Extension-Driven Discovery**: The test runner automatically discovers what to do based on file extensions (`.asn1`, `.py`, `.rs`, `.yaml`), eliminating the need for hardcoded test lists or complex manifests.
3. **Multi-Backend Ready**: The architecture is designed to easily scale from Python to Rust, Go, or C backends in the future.

---

## 📁 Directory Structure

```text
tests/
├── README.md                          # You are here!
├── conftest.py                        # Shared pytest fixtures (e.g., compile_asn1)
├── run_integration.py                 # The dynamic, multi-backend test runner
└── vectors/                           # All test data and scenarios live here
    ├── runtime_tests/                 # Pure runtime unit tests (no .asn1 compilation needed)
    │   └── test_runtime.py            
    ├── <feature_name>/                # Example: x509_subset, explicit_choice, ber_primitives
    │   ├── schema.asn1                # The ASN.1 schema (optional for pure runtime tests)
    │   ├── test_*.py                  # Python test script(s) for this feature
    │   ├── test_*.rs                  # (Future) Rust test script(s) for this feature
    │   └── payloads.yaml              # (Optional) Hex/JSON test vectors for this feature
```

---

## 🚀 Running Tests

We use `just` as our command runner for consistency. From the repository root, you can run:

| Command | Description |
|---------|-------------|
| `just test` | Runs **all** tests (Rust, Python runtime, and Integration). |
| `just test-rust` | Runs only the Rust unit tests (parser, IR, codegen). |
| `just test-python` | Runs pure Python runtime unit tests (e.g., `tests/vectors/runtime_tests/test_runtime.py`). |
| `just test-integration` | Runs the dynamic integration test runner (`tests/run_integration.py`). |

> **Note**: You can also run `python3 tests/run_integration.py` directly if you want to see the live output of the integration test discovery and execution.

---

## ⚙️ How the Integration Runner Works

The `tests/run_integration.py` script is the heart of our end-to-end testing. It works in three phases for every folder inside `tests/vectors/`:

1. **Discovery**: It scans `tests/vectors/` for folders containing `test_*.py` or `test_*.rs` files.
2. **Compilation**: If the folder contains `.asn1` files, it compiles them using `cargo run -- -o <temp_dir> <schema.asn1>`. It then copies the `asnvil-runtime-python` into that same temp directory.
3. **Execution**: 
   - For `.py` files, it sets `PYTHONPATH` to the temp directory and runs `pytest`.
   - For `.rs` files (future), it would set up a temporary Cargo workspace and run `cargo test`.

Because it relies on file extensions, **you never have to update a central list** when adding a new test. Just create the files, and the runner finds them.

---

## ➕ How to Add a New Test

### Scenario A: Adding a Pure Runtime Test
*Use this for testing `asnvil_runtime` primitives (e.g., tag decoding, length encoding) without needing an ASN.1 schema.*

1. Create a new folder: `tests/vectors/my_new_runtime_feature/`
2. Add your test file: `tests/vectors/my_new_runtime_feature/test_my_feature.py`
3. (Optional) Add test data: `tests/vectors/my_new_runtime_feature/data.yaml`
4. Run `just test-python` to verify.

### Scenario B: Adding a Parser / AST Test
*Use this to verify the parser correctly builds the AST for specific grammar rules.*

1. Create a new folder: `tests/vectors/my_parser_feature/`
2. Add your schema: `tests/vectors/my_parser_feature/schema.asn1`
3. Open `asnvil-parser/src/lib.rs` and add a new `#[test]` function.
4. Use the built-in helper to load and parse the schema:
   ```rust
   #[test]
   fn test_my_parser_feature() {
       let source = load_vector("my_parser_feature", "schema.asn1");
       let ast = parse_source(&source);
       // ... add your assertions on the AST ...
   }
   ```
5. Run `just test-rust` to verify.

### Scenario C: Adding an End-to-End Integration Test
*Use this to test the full pipeline: ASN.1 Schema → Python Code Generation → BER/DER Roundtrip.*

1. Create a new folder: `tests/vectors/my_integration_feature/`
2. Add your schema: `tests/vectors/my_integration_feature/schema.asn1`
   *(Ensure the `MODULE IDENTIFIER` inside the file matches what you will import in Python).*
3. Add your test script: `tests/vectors/my_integration_feature/test_roundtrip.py`
   ```python
   import os
   import sys
   from pathlib import Path

   # The runner injects the output directory into the environment
   output_dir = Path(os.environ["INTEG_OUTPUT_DIR"])
   sys.path.insert(0, str(output_dir))

   # Import using the MODULE IDENTIFIER name from your .asn1 file, NOT the filename
   from MyModule import MyType

   def test_my_type_roundtrip():
       obj = MyType(field1="test", field2=42)
       encoded = obj.encode_der()
       decoded = MyType.decode_der(encoded)
       assert decoded.field1 == "test"
   ```
4. (Optional) Add `payloads.yaml` if you are testing against specific hex vectors.
5. Run `just test-integration`. The runner will automatically discover the new folder, compile the schema, and execute your pytest file.

---

## 📝 Best Practices

- **Name folders semantically**: Use descriptive names like `explicit_choice` or `ldap_subset`, not generic names like `test1`.
- **One schema per folder (usually)**: Keep `schema.asn1` focused on a single concept. If you need to test imports, you can add `imported.asn1` to the same folder; the runner will compile all `.asn1` files it finds there.
- **Keep tests self-contained**: A new developer should be able to look at a single folder in `tests/vectors/` and understand exactly what is being tested, what the inputs are, and what the expected behavior is, without jumping to other directories.
- **Use `conftest.py` fixtures**: If you need to compile a schema dynamically *inside* a pytest file (instead of relying on the pre-compilation step of `run_integration.py`), use the `compile_asn1` fixture provided in `tests/conftest.py`.

---

## ❓ Troubleshooting

- **"ModuleNotFoundError: No module named 'MyModule'"**: Ensure the name you are importing in your `.py` file exactly matches the `MODULE IDENTIFIER` defined at the top of your `schema.asn1` file, not the filename itself.
- **Tests failing on `encode_der()`**: Check if your schema uses constraints (e.g., `INTEGER (0..100)`). The generated code will call `validate()` during encoding, which will raise a `ConstraintViolationError` if the test data is out of bounds.
- **Runner not finding my new test**: Ensure your Python test file starts with `test_` (e.g., `test_my_feature.py`). The runner uses `glob("test_*.py")` for discovery.
