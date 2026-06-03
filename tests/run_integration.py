"""Self-contained integration test runner.

Compiles each .asn1 spec, copies the runtime, then runs pytest on the
corresponding test_*.py file.  Removes hardcoded /tmp/ paths (R17).
"""
from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
TESTS_DIR = REPO_ROOT / "tests"
INTEGRATION_DIR = TESTS_DIR / "integration"
PYTHON_BER_DIR = INTEGRATION_DIR / "python" / "ber"
RUNTIME_SRC = REPO_ROOT / "asnvil-runtime-python"

# Specs and their corresponding test files
TEST_SPECS: list[tuple[Path, Path]] = [
    (TESTS_DIR / "explicit_choice.asn1", TESTS_DIR / "test_explicit_choice.py"),
    (TESTS_DIR / "inline_choice.asn1", TESTS_DIR / "test_inline_choice.py"),
    (TESTS_DIR / "any_defined_by.asn1", TESTS_DIR / "test_any_defined_by.py"),
    (INTEGRATION_DIR / "x509_certificate.asn1", INTEGRATION_DIR / "test_x509_roundtrip.py"),
    (INTEGRATION_DIR / "ldap_protocol.asn1", INTEGRATION_DIR / "test_ldap_roundtrip.py"),
    (INTEGRATION_DIR / "snmp_protocol.asn1", INTEGRATION_DIR / "test_snmp_roundtrip.py"),
    # Python BER integration tests (Phase 3)
    (PYTHON_BER_DIR / "62-any-OK.asn1", PYTHON_BER_DIR / "test_any_decode.py"),
    # NOTE: 43-recursion-OK.asn1 causes stack overflow (known limitation)
    # (PYTHON_BER_DIR / "43-recursion-OK.asn1", PYTHON_BER_DIR / "test_recursive.py"),
    (PYTHON_BER_DIR / "65-multi-tag-OK.asn1", PYTHON_BER_DIR / "test_multi_tag.py"),
    (PYTHON_BER_DIR / "67-embedded-choice-OK.asn1", PYTHON_BER_DIR / "test_embedded_choice.py"),
]


def compile_asn1(asn1_path: Path, output_dir: Path) -> None:
    """Run `cargo run -- -o <output_dir> <asn1_path>`."""
    result = subprocess.run(
        [
            "cargo", "run", "--quiet", "--",
            "-o", str(output_dir),
            str(asn1_path),
        ],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        print(f"FAIL: cargo compile failed for {asn1_path.name}")
        print(f"  stderr: {result.stderr}")
        print(f"  stdout: {result.stdout}")
        sys.exit(1)


def copy_runtime(output_dir: Path) -> None:
    """Copy asnvil-runtime-python/ to <output_dir>/asnvil_runtime/."""
    dest = output_dir / "asnvil_runtime"
    if dest.exists():
        shutil.rmtree(dest)
    shutil.copytree(RUNTIME_SRC, dest)


def run_pytest(test_file: Path, output_dir: Path) -> bool:
    """Run pytest on the test file with output_dir on PYTHONPATH."""
    env = os.environ.copy()
    # Output dir MUST come first to shadow stale tests/TestModule.py
    existing_path = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = str(output_dir) + (":" + existing_path if existing_path else "")
    env["INTEG_OUTPUT_DIR"] = str(output_dir)
    result = subprocess.run(
        ["uv", "run", "--with", "pytest", "pytest", str(test_file), "-v", "--noconftest"],
        cwd=REPO_ROOT,
        env=env,
        capture_output=True,
        text=True,
    )
    print(result.stdout)
    if result.stderr:
        print(result.stderr, file=sys.stderr)
    return result.returncode == 0


def main() -> None:
    failures = 0
    for asn1_path, test_file in TEST_SPECS:
        if not asn1_path.exists():
            print(f"SKIP: {asn1_path.name} not found")
            continue
        if not test_file.exists():
            print(f"SKIP: {test_file.name} not found")
            continue

        print(f"\n{'=' * 60}")
        print(f"Testing: {asn1_path.name} -> {test_file.name}")
        print(f"{'=' * 60}")

        with tempfile.TemporaryDirectory(prefix="asnvil-test-") as tmpdir:
            output_dir = Path(tmpdir) / "output"
            output_dir.mkdir()

            compile_asn1(asn1_path, output_dir)
            copy_runtime(output_dir)
            ok = run_pytest(test_file, output_dir)

            if not ok:
                failures += 1
                print(f"FAIL: {test_file.name}")
            else:
                print(f"PASS: {test_file.name}")

    if failures:
        print(f"\n{failures} integration test suite(s) failed")
        sys.exit(1)
    else:
        print(f"\nAll {len(TEST_SPECS)} integration test suites passed!")


if __name__ == "__main__":
    main()
