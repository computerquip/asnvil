import sys
import os
import shutil
import subprocess
import tempfile
import importlib.util
from pathlib import Path

import pytest

# ─── Path setup ──────────────────────────────────────────────────────

# Load asnvil-runtime-python/ as the `asnvil_runtime` package.
# Python module names cannot contain hyphens, so we register the package
# manually via importlib.
_runtime_dir = Path(__file__).resolve().parent.parent / "asnvil-runtime-python"
_runtime_init = _runtime_dir / "__init__.py"
if "asnvil_runtime" not in sys.modules:
    import types
    _pkg = types.ModuleType("asnvil_runtime")
    _pkg.__path__ = [str(_runtime_dir)]
    _pkg.__file__ = str(_runtime_init)
    sys.modules["asnvil_runtime"] = _pkg
    # Execute __init__.py in the package namespace
    with open(_runtime_init) as _f:
        exec(compile(_f.read(), str(_runtime_init), "exec"), _pkg.__dict__)

# ─── Constants ───────────────────────────────────────────────────────

REPO_ROOT = Path(__file__).resolve().parent.parent
RUNTIME_SRC = REPO_ROOT / "asnvil-runtime-python"

# ─── Fixtures ────────────────────────────────────────────────────────


@pytest.fixture(scope="session")
def runtime_path():
    """Path to the asnvil-runtime-python source directory."""
    return RUNTIME_SRC


@pytest.fixture
def compile_asn1():
    """Helper to compile an ASN.1 file and return the output directory path.

    Usage:
        def test_something(compile_asn1):
            output_dir = compile_asn1("tests/vectors/integration/explicit_choice/schema.asn1")
            # output_dir has compiled .py files and asnvil_runtime/
    """
    dirs_to_cleanup = []

    def _compile(asn1_path):
        asn1_path = Path(asn1_path)
        if not asn1_path.is_absolute():
            asn1_path = REPO_ROOT / asn1_path

        tmpdir = tempfile.mkdtemp(prefix="asnvil-test-")
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
        dest = output_dir / "asnvil_runtime"
        shutil.copytree(RUNTIME_SRC, dest)

        return output_dir

    yield _compile

    # Cleanup
    for d in dirs_to_cleanup:
        shutil.rmtree(d, ignore_errors=True)


@pytest.fixture
def generated_module(compile_asn1, request):
    """Compile the schema.asn1 in the current test directory and set up imports.

    Usage:
        def test_something(generated_module):
            # generated_module is the output_dir path
            sys.path.insert(0, str(generated_module))
            # Import using the ASN.1 MODULE IDENTIFIER name, not the filename
            # from TestModule import Person
    """
    test_dir = Path(request.fspath).parent
    schema_path = test_dir / "schema.asn1"
    
    if not schema_path.exists():
        raise FileNotFoundError(
            f"Could not find schema.asn1 in {test_dir}. "
            "Ensure the test is in a vector folder with a schema.asn1 file."
        )
        
    output_dir = compile_asn1(schema_path)
    if str(output_dir) not in sys.path:
        sys.path.insert(0, str(output_dir))
    return output_dir

