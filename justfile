# Default recipe
default:
    just build

build:
    cargo build --workspace

test: (test-rust) (test-python) (test-integration)

test-rust:
    cargo test --workspace

test-python:
    uv run --with pytest pytest tests/test_runtime.py

test-integration:
    python3 tests/run_integration.py

test-all: test-rust test-python test-integration

clean:
    cargo clean
    rm -rf output/
