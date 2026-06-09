# Default recipe
default:
    just build

build:
    cargo build --workspace

test: (test-rust) (test-python) (test-integration)

test-rust:
    cargo test --workspace

test-python:
    uv run --with pytest --with pyyaml pytest tests/vectors/runtime_tests/test_runtime.py

test-integration:
    python3 tests/run_integration.py

test-all: test-rust test-python test-integration test-rust-integration

test-rust-integration:
    cargo build -p asnvil
    mkdir -p tests/rust_integration/generated
    cargo run -p asnvil -- tests/vectors/explicit_choice/schema.asn1 -o tests/rust_integration/generated --lang rust
    cargo test -p rust_integration

clean:
    cargo clean
    rm -rf output/
