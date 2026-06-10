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
VECTORS_DIR = TESTS_DIR / "vectors"
RUNTIME_SRC = REPO_ROOT / "asnvil-runtime-python"


def compile_asn1_files(asn1_files: list[Path], output_dir: Path, lang: str = "python") -> None:
    """Run `cargo run` to compile all provided .asn1 files into the output directory."""
    for asn1_path in asn1_files:
        result = subprocess.run(
            [
                "cargo", "run", "--quiet", "--",
                "-o", str(output_dir),
                "--lang", lang,
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
        ["uv", "run", "--with", "pytest", "--with", "pyyaml", "pytest"] + test_files_str + ["-v"],
        cwd=REPO_ROOT,
        env=env,
        capture_output=True,
        text=True,
    )
    print(result.stdout)
    if result.stderr:
        print(result.stderr, file=sys.stderr)
    return result.returncode == 0


def run_rust_tests(test_files: list[Path], output_dir: Path) -> bool:
    """Run rust-script on the provided .rs test files with generated code available."""
    all_passed = True
    for test_file in test_files:
        with tempfile.TemporaryDirectory() as tmpdir:
            tmp_test_file = Path(tmpdir) / test_file.name
            
            # Read the test file and substitute __REPO_ROOT__ with the actual absolute path
            content = test_file.read_text()
            content = content.replace("__REPO_ROOT__", str(REPO_ROOT))
            tmp_test_file.write_text(content)
            
            # Copy all generated .rs files to the temp dir so they can be included via #[path]
            for rs_file in output_dir.glob("*.rs"):
                shutil.copy(rs_file, Path(tmpdir) / rs_file.name)
            
            env = os.environ.copy()
            env["INTEG_OUTPUT_DIR"] = str(output_dir)
            result = subprocess.run(
                ["rust-script", "--test", str(tmp_test_file)],
                cwd=REPO_ROOT,
                env=env,
                capture_output=True,
                text=True,
            )
            print(result.stdout)
            if result.stderr:
                print(result.stderr, file=sys.stderr)
            if result.returncode != 0:
                all_passed = False
                print(f"FAIL: rust-script execution failed for {test_file.name}")
            else:
                print(f"PASS: {test_file.name}")
    return all_passed


def discover_suites() -> list[dict]:
    """Dynamically discover all test suites based on file extensions."""
    suites = []
    for folder in VECTORS_DIR.iterdir():
        if not folder.is_dir() or folder.name.startswith("."):
            continue
        
        asn1_files = sorted(folder.glob("*.asn1"))
        py_test_files = sorted(folder.glob("test_*.py"))
        rs_test_files = sorted(folder.glob("test_*.rs"))
        
        # A suite is valid if it has test files (asn1 files are optional for pure runtime tests)
        if py_test_files or rs_test_files:
            suites.append({
                "name": folder.name,
                "folder": folder,
                "asn1_files": asn1_files,
                "py_test_files": py_test_files,
                "rs_test_files": rs_test_files,
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
                ok = True
                
                # Compile for Python if there are Python tests
                if suite["asn1_files"] and suite["py_test_files"]:
                    compile_asn1_files(suite["asn1_files"], output_dir, lang="python")
                    copy_runtime(output_dir)
                    ok = run_python_tests(suite["py_test_files"], output_dir) and ok
                
                # Compile for Rust if there are Rust tests
                if suite["asn1_files"] and suite["rs_test_files"]:
                    # Use a separate output dir for Rust to avoid mixing .py and .rs
                    rust_output_dir = output_dir / "rust"
                    rust_output_dir.mkdir()
                    compile_asn1_files(suite["asn1_files"], rust_output_dir, lang="rust")
                    ok = run_rust_tests(suite["rs_test_files"], rust_output_dir) and ok

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
