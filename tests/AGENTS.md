# AGENTS.md — asnvil Test Framework

This document provides technical guidance for AI agents working on the `asnvil` test suite. Read this before modifying, adding, or debugging tests.

## 🏗️ Architecture Overview
The test framework is **flat, extension-driven, and co-located**. 
- There are no hardcoded test lists or complex manifests.
- The test runner (`tests/run_integration.py`) dynamically discovers test suites by scanning `tests/vectors/` for folders containing `test_*.py` or `test_*.rs` files.
- All artifacts for a specific test scenario (schemas, payloads, test scripts) live together in a single feature folder.

## 📁 Directory Structure
```text
tests/
├── conftest.py                  # Shared pytest fixtures (e.g., `compile_asn1`)
├── run_integration.py           # Dynamic, multi-backend test runner
└── vectors/                     # ALL test data and scenarios live here
    ├── runtime_tests/           # Pure runtime unit tests (no .asn1 compilation)
    │   └── test_runtime.py      
    ├── <feature_name>/          # Example: x509_subset, explicit_choice, ber_primitives
    │   ├── *.asn1               # One or more ASN.1 schemas (e.g., schema.asn1, imports.asn1)
    │   ├── test_*.py            # Python test script(s) for this feature
    │   ├── test_*.rs            # (Future) Rust test script(s) for this feature
    │   └── *.yaml               # (Optional) Hex/JSON test vectors for this feature
```

## ⚙️ How the Integration Runner Works
When `python3 tests/run_integration.py` (or `just test-integration`) is executed:
1. **Discovery**: Iterates through all directories in `tests/vectors/`.
2. **Validation**: A directory is considered a valid test suite if it contains at least one `test_*.py` or `test_*.rs` file.
3. **Compilation**: If the directory contains `*.asn1` files, it compiles *all* of them into a temporary output directory using `cargo run -- -o <temp_dir> <schema.asn1>`.
4. **Runtime Setup**: Copies `asnvil-runtime-python/` into the temporary output directory.
5. **Execution**: 
   - For `.py` files: Sets `PYTHONPATH` and `INTEG_OUTPUT_DIR` environment variables, then runs `pytest`.
   - For `.rs` files (future): Sets up a temporary Cargo workspace and runs `cargo test`.

## ➕ How to Add New Tests

### 1. Parser / AST Tests
- **Location**: `asnvil-parser/src/lib.rs` (inside the `#[cfg(test)] mod tests` block).
- **Action**: Use the built-in `load_vector("<folder_name>", "schema.asn1")` helper to read the ASN.1 source, then assert on the resulting `ast::Module`.
- **Rule**: Do not use inline ASN.1 strings. Always create a corresponding folder in `tests/vectors/<feature_name>/` with a `schema.asn1` file.

### 2. Pure Runtime Tests
- **Location**: `tests/vectors/runtime_tests/`
- **Action**: Add a new `test_*.py` file. These tests import `asnvil_runtime` directly and do not require `.asn1` compilation.
- **Rule**: Update `justfile` `test-python` recipe if a new top-level runtime test file is added.

### 3. End-to-End Integration Tests
- **Location**: Create a new folder `tests/vectors/<feature_name>/`.
- **Action**: 
  1. Add `schema.asn1` (and `imports.asn1` if needed). The runner will compile all `.asn1` files in this folder.
  2. Add `test_*.py`. Import generated modules using the **ASN.1 MODULE IDENTIFIER** (e.g., `from TestModule import Person`), *not* the filename.
  3. (Optional) Add `payloads.yaml` for data-driven hex/byte assertions.
- **Rule**: The test file name *must* start with `test_` (e.g., `test_roundtrip.py`) for the runner's `glob("test_*.py")` to discover it.

## ⚠️ Critical Constraints & Anti-Patterns
- **NEVER hardcode test paths** in `run_integration.py`. The runner must remain purely extension-driven.
- **NEVER use inline ASN.1 strings** in parser tests. Always use `load_vector()`.
- **Module Naming**: Python imports in integration tests must match the `MODULE IDENTIFIER` defined inside the `.asn1` file, not the `.asn1` filename.
- **Co-location**: Do not scatter related test files across different directories. If a test needs a schema and a YAML payload, they belong in the same `tests/vectors/<feature_name>/` folder.
- **Cleanup**: If you move or rename a test vector, ensure no orphaned files (like old `test_*.py` or `schema.asn1`) are left behind in the old location.

## 🔧 Troubleshooting for Agents
- **"ModuleNotFoundError" in integration test**: The Python file is trying to import the filename instead of the `MODULE IDENTIFIER`. Check the `.asn1` file's first line.
- **Runner skips a new folder**: Ensure the Python test file is named `test_*.py` (not just `*.py`).
- **Constraint validation failures**: If `encode_der()` throws a `ConstraintViolationError`, the test data violates a size/value range defined in the `.asn1` schema. Adjust the test data, not the schema.
