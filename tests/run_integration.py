"""Self-contained, multi-backend integration test runner.

Dynamically discovers test suites in tests/vectors/integration/ based on
file extensions (.asn1 for schemas, test_*.py for Python tests, etc.).
Compiles schemas for detected backends and runs the corresponding test commands.
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
VECTORS_DIR = TESTS_DIR / "vectors" / "integration"
RUNTIME_SRC = REPO_ROOT / "asnvil-runtime-python"


def compile_asn1_files(asn1_files: list[Path], output_dir: Path) -> None:
    """Run `cargo run` to compile all provided .asn1 files into the output directory."""
    for asn1_path in asn1_files:
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


def run_python_tests(test_files: list[Path], output_dir: Path) -> bool:
    """Run pytest on the provided Python test files with output_dir on PYTHONPATH."""
    env = os.environ.copy()
    existing_path = env.get("PYTHONPATH", "")
    env["PYTHONPATH"] = str(output_dir) + (":" + existing_path if existing_path else "")
    env["INTEG_OUTPUT_DIR"] = str(output_dir)
    
    test_files_str = [str(f) for f in test_files]
    result = subprocess.run(
        ["uv", "run", "--with", "pytest", "pytest"] + test_files_str + ["-v", "--noconftest"],
        cwd=REPO_ROOT,
        env=env,
        capture_output=True,
        text=True,
    )
    print(result.stdout)
    if result.stderr:
        print(result.stderr, file=sys.stderr)
    return result.returncode == 0


def discover_suites() -> list[dict]:
    """Dynamically discover all integration test suites based on file extensions."""
    suites = []
    for folder in VECTORS_DIR.iterdir():
        if not folder.is_dir():
            continue
        
        asn1_files = sorted(folder.glob("*.asn1"))
        py_test_files = sorted(folder.glob("test_*.py"))
        # Future: rs_test_files = sorted(folder.glob("test_*.rs"))
        # Future: go_test_files = sorted(folder.glob("test_*.go"))
        
        if asn1_files and py_test_files:
            suites.append({
                "name": folder.name,
                "folder": folder,
                "asn1_files": asn1_files,
                "py_test_files": py_test_files,
            })
            
    # Sort suites by name for consistent output
    return sorted(suites, key=lambda x: x["name"])


def main() -> None:
    suites = discover_suites()
    if not suites:
        print("No integration test suites found in", VECTORS_DIR)
        sys.exit(1)

    print(f"Discovered {len(suites)} integration test suite(s).\n")
    
    failures = 0
    for suite in suites:
        print(f"{'=' * 60}")
        print(f"Testing suite: {suite['name']}")
        print(f"  Schemas: {', '.join(f.name for f in suite['asn1_files'])}")
        print(f"  Tests:   {', '.join(f.name for f in suite['py_test_files'])}")
        print(f"{'=' * 60}")

        with tempfile.TemporaryDirectory(prefix="asnvil-test-") as tmpdir:
            output_dir = Path(tmpdir) / "output"
            output_dir.mkdir()

            try:
                compile_asn1_files(suite["asn1_files"], output_dir)
                copy_runtime(output_dir)
                
                ok = True
                if suite["py_test_files"]:
                    ok = run_python_tests(suite["py_test_files"], output_dir)
                
                # Future: if suite["rs_test_files"]: ok = run_rust_tests(...) and ok

                if not ok:
                    failures += 1
                    print(f"FAIL: {suite['name']}")
                else:
                    print(f"PASS: {suite['name']}")
            except Exception as e:
                failures += 1
                print(f"FAIL: {suite['name']} - {e}")

    print(f"\n{'=' * 60}")
    if failures:
        print(f"{failures} integration test suite(s) failed")
        sys.exit(1)
    else:
        print(f"All {len(suites)} integration test suites passed!")


if __name__ == "__main__":
    main()
