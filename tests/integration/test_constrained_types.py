"""Integration tests for constrained ASN.1 types."""
import pytest

def compile_and_import(tmp_path):
    import subprocess, shutil, sys
    asn_file = tmp_path / "2000_constrained_types.asn1"
    shutil.copy("tests/vectors/asn1/2000_constrained_types.asn1", asn_file)
    out_dir = tmp_path / "out"
    out_dir.mkdir()
    runtime_dir = tmp_path / "asnvil_runtime"
    shutil.copytree("asnvil-runtime-python", runtime_dir)
    result = subprocess.run(
        [sys.executable, "-m", "cargo", "run", "--", "-o", str(out_dir), str(asn_file)],
        cwd=".", capture_output=True, text=True
    )
    if result.returncode != 0:
        # Try direct cargo run
        result = subprocess.run(
            ["cargo", "run", "--", "-o", str(out_dir), str(asn_file)],
            cwd=".", capture_output=True, text=True
        )
        assert result.returncode == 0, f"Compilation failed: {result.stderr}"
    import importlib.util
    spec = importlib.util.spec_from_file_location("ConstrainedTypes", out_dir / "ConstrainedTypes.py")
    mod = importlib.util.module_from_spec(spec)
    sys.modules["ConstrainedTypes"] = mod
    spec.loader.exec_module(mod)
    return mod


def test_valid_user_record(tmp_path):
    mod = compile_and_import(tmp_path)
    user = mod.UserRecord(id=42, name="Alice", age=30)
    data = user.encode_der()
    decoded = mod.UserRecord.decode_der(data)
    assert decoded.id == 42
    assert decoded.name == "Alice"
    assert decoded.age == 30


def test_valid_user_with_optional(tmp_path):
    mod = compile_and_import(tmp_path)
    user = mod.UserRecord(id=100, name="Bob", age=25, status=3, notes="Test notes")
    data = user.encode_der()
    decoded = mod.UserRecord.decode_der(data)
    assert decoded.id == 100
    assert decoded.name == "Bob"
    assert decoded.status == 3
    assert decoded.notes == "Test notes"


def test_valid_boundary_id_zero(tmp_path):
    mod = compile_and_import(tmp_path)
    user = mod.UserRecord(id=0, name="X", age=0)
    data = user.encode_der()
    decoded = mod.UserRecord.decode_der(data)
    assert decoded.id == 0


def test_valid_boundary_id_max(tmp_path):
    mod = compile_and_import(tmp_path)
    user = mod.UserRecord(id=1000, name="Y", age=150)
    data = user.encode_der()
    decoded = mod.UserRecord.decode_der(data)
    assert decoded.id == 1000
    assert decoded.age == 150


def test_valid_name_min_length(tmp_path):
    mod = compile_and_import(tmp_path)
    user = mod.UserRecord(id=1, name="A", age=1)
    data = user.encode_der()
    decoded = mod.UserRecord.decode_der(data)
    assert decoded.name == "A"


def test_valid_name_max_length(tmp_path):
    mod = compile_and_import(tmp_path)
    user = mod.UserRecord(id=1, name="A" * 50, age=1)
    data = user.encode_der()
    decoded = mod.UserRecord.decode_der(data)
    assert len(decoded.name) == 50


def test_invalid_id_exceeds_max(tmp_path):
    mod = compile_and_import(tmp_path)
    from asnvil_runtime import ConstraintViolationError
    user = mod.UserRecord(id=1001, name="Test", age=25)
    with pytest.raises(ConstraintViolationError):
        user.encode_der()


def test_invalid_age_negative(tmp_path):
    mod = compile_and_import(tmp_path)
    from asnvil_runtime import ConstraintViolationError
    user = mod.UserRecord(id=1, name="Test", age=-1)
    with pytest.raises(ConstraintViolationError):
        user.encode_der()


def test_invalid_name_empty(tmp_path):
    mod = compile_and_import(tmp_path)
    from asnvil_runtime import ConstraintViolationError
    user = mod.UserRecord(id=1, name="", age=25)
    with pytest.raises(ConstraintViolationError):
        user.encode_der()
