"""Integration tests for constrained ASN.1 types."""
import pytest
import sys
import importlib.util
from pathlib import Path

@pytest.fixture
def constrained_types_module(compile_asn1):
    """Compile the constrained types vector and load the generated Python module."""
    output_dir = compile_asn1("tests/vectors/asn1/2000_constrained_types.asn1")
    spec = importlib.util.spec_from_file_location(
        "ConstrainedTypes", output_dir / "ConstrainedTypes.py"
    )
    mod = importlib.util.module_from_spec(spec)
    sys.modules["ConstrainedTypes"] = mod
    spec.loader.exec_module(mod)
    return mod


def test_valid_user_record(constrained_types_module):
    user = constrained_types_module.UserRecord(id=42, name="Alice", age=30)
    data = user.encode_der()
    decoded = constrained_types_module.UserRecord.decode_der(data)
    assert decoded.id == 42
    assert decoded.name == "Alice"
    assert decoded.age == 30


def test_valid_user_with_optional(constrained_types_module):
    user = constrained_types_module.UserRecord(id=100, name="Bob", age=25, status=3, notes="Test notes")
    data = user.encode_der()
    decoded = constrained_types_module.UserRecord.decode_der(data)
    assert decoded.id == 100
    assert decoded.name == "Bob"
    assert decoded.status == 3
    assert decoded.notes == "Test notes"


def test_valid_boundary_id_zero(constrained_types_module):
    user = constrained_types_module.UserRecord(id=0, name="X", age=0)
    data = user.encode_der()
    decoded = constrained_types_module.UserRecord.decode_der(data)
    assert decoded.id == 0


def test_valid_boundary_id_max(constrained_types_module):
    user = constrained_types_module.UserRecord(id=1000, name="Y", age=150)
    data = user.encode_der()
    decoded = constrained_types_module.UserRecord.decode_der(data)
    assert decoded.id == 1000
    assert decoded.age == 150


def test_valid_name_min_length(constrained_types_module):
    user = constrained_types_module.UserRecord(id=1, name="A", age=1)
    data = user.encode_der()
    decoded = constrained_types_module.UserRecord.decode_der(data)
    assert decoded.name == "A"


def test_valid_name_max_length(constrained_types_module):
    user = constrained_types_module.UserRecord(id=1, name="A" * 50, age=1)
    data = user.encode_der()
    decoded = constrained_types_module.UserRecord.decode_der(data)
    assert len(decoded.name) == 50


def test_invalid_id_exceeds_max(constrained_types_module):
    from asnvil_runtime import ConstraintViolationError
    user = constrained_types_module.UserRecord(id=1001, name="Test", age=25)
    with pytest.raises(ConstraintViolationError):
        user.encode_der()


def test_invalid_age_negative(constrained_types_module):
    from asnvil_runtime import ConstraintViolationError
    user = constrained_types_module.UserRecord(id=1, name="Test", age=-1)
    with pytest.raises(ConstraintViolationError):
        user.encode_der()


def test_invalid_name_empty(constrained_types_module):
    from asnvil_runtime import ConstraintViolationError
    user = constrained_types_module.UserRecord(id=1, name="", age=25)
    with pytest.raises(ConstraintViolationError):
        user.encode_der()
