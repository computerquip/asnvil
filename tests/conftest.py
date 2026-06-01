import sys
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

import pytest

# ─── Path setup ──────────────────────────────────────────────────────

# Add project root so `from asn1c_runtime` resolves to the symlink
_project_root = os.path.join(os.path.dirname(__file__), "..")
_project_root = os.path.normpath(_project_root)
if _project_root not in sys.path:
    sys.path.insert(0, _project_root)

# ─── Constants ───────────────────────────────────────────────────────

REPO_ROOT = Path(__file__).resolve().parent.parent
RUNTIME_SRC = REPO_ROOT / "asn1c-runtime-python"

# ─── Fixtures ────────────────────────────────────────────────────────


@pytest.fixture(scope="session")
def runtime_path():
    """Path to the asn1c-runtime-python source directory."""
    return RUNTIME_SRC


@pytest.fixture
def compile_asn1():
    """Helper to compile an ASN.1 file and return the output directory path.

    Usage:
        def test_something(compile_asn1):
            output_dir = compile_asn1("tests/explicit_choice.asn1")
            # output_dir has compiled .py files and asn1c_runtime/
    """
    dirs_to_cleanup = []

    def _compile(asn1_path):
        asn1_path = Path(asn1_path)
        if not asn1_path.is_absolute():
            asn1_path = REPO_ROOT / asn1_path

        tmpdir = tempfile.mkdtemp(prefix="asn1c-test-")
        dirs_to_cleanup.append(tmpdir)
        output_dir = Path(tmpdir) / "output"
        output_dir.mkdir()

        # Compile
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
            raise RuntimeError(
                f"cargo compile failed for {asn1_path.name}:\n{result.stderr}"
            )

        # Copy runtime
        dest = output_dir / "asn1c_runtime"
        shutil.copytree(RUNTIME_SRC, dest)

        return output_dir

    yield _compile

    # Cleanup
    for d in dirs_to_cleanup:
        shutil.rmtree(d, ignore_errors=True)


@pytest.fixture
def generated_module(compile_asn1):
    """Compile the default test module (inline_choice.asn1) and set up imports.

    Usage:
        def test_something(generated_module, compile_asn1):
            # generated_module is the output_dir path
            sys.path.insert(0, str(generated_module))
            from TestModule import Person
    """
    output_dir = compile_asn1("tests/inline_choice.asn1")
    if str(output_dir) not in sys.path:
        sys.path.insert(0, str(output_dir))
    return output_dir

