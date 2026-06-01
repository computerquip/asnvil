"""Tests for indefinite length BER encoding/decoding roundtrip."""
from TestModule import Person, Department, Company, Config, Certificate, Status


def test_person_ber_indefinite_roundtrip():
    p = Person(name="Alice", age=30, active=True, status=0)
    encoded = p.encode_ber_indefinite()
    assert encoded[:2] == bytes([0x30, 0x80])
    assert encoded[-2:] == bytes([0x00, 0x00])
    decoded = Person.decode_ber_indefinite(encoded)
    assert decoded.name == "Alice"
    assert decoded.age == 30
    assert decoded.active is True
    assert decoded.status == 0


def test_department_ber_indefinite_roundtrip():
    d = Department(name="Engineering", people=[])
    encoded = d.encode_ber_indefinite()
    assert encoded[:2] == bytes([0x30, 0x80])
    assert encoded[-2:] == bytes([0x00, 0x00])
    decoded = Department.decode_ber_indefinite(encoded)
    assert decoded.name == "Engineering"


def test_company_ber_indefinite_roundtrip():
    c = Company(name="Acme Corp", departments=[])
    encoded = c.encode_ber_indefinite()
    assert encoded[:2] == bytes([0x30, 0x80])
    assert encoded[-2:] == bytes([0x00, 0x00])
    decoded = Company.decode_ber_indefinite(encoded)
    assert decoded.name == "Acme Corp"


def test_config_ber_indefinite_defaults():
    c = Config()
    encoded = c.encode_ber_indefinite()
    assert encoded[:2] == bytes([0x30, 0x80])
    assert encoded[-2:] == bytes([0x00, 0x00])
    decoded = Config.decode_ber_indefinite(encoded)
    assert decoded.enabled is True
    assert decoded.count == 42
    assert decoded.label == "default"


def test_config_ber_indefinite_with_values():
    c = Config(enabled=False, count=100, label="custom")
    encoded = c.encode_ber_indefinite()
    assert encoded[:2] == bytes([0x30, 0x80])
    assert encoded[-2:] == bytes([0x00, 0x00])
    decoded = Config.decode_ber_indefinite(encoded)
    assert decoded.enabled is False
    assert decoded.count == 100
    assert decoded.label == "custom"


def test_ber_indefinite_inner_matches_der_inner():
    """Verify indefinite inner content matches DER inner content."""
    p = Person(name="Bob", age=25, active=False, status=1)
    der = p.encode_der()
    indefinite = p.encode_ber_indefinite()
    der_inner = der[2:]
    indefinite_inner = indefinite[2:-2]
    assert indefinite_inner == der_inner


def test_ber_indefinite_decode_matches_der_decode():
    """Verify decoding indefinite produces same result as DER decode."""
    p = Person(name="Charlie", age=40, active=True, status=0)
    der = p.encode_der()
    indefinite = p.encode_ber_indefinite()
    decoded_der = Person.decode_der(der)
    decoded_indefinite = Person.decode_ber_indefinite(indefinite)
    assert decoded_der.name == decoded_indefinite.name
    assert decoded_der.age == decoded_indefinite.age
    assert decoded_der.active == decoded_indefinite.active
    assert decoded_der.status == decoded_indefinite.status


def test_ber_indefinite_nested():
    """Test indefinite encoding with SEQUENCE/SET types."""
    c = Company(name="Nested Corp", departments=[])
    encoded = c.encode_ber_indefinite()
    assert encoded[:2] == bytes([0x30, 0x80])
    assert encoded[-2:] == bytes([0x00, 0x00])
    decoded = Company.decode_ber_indefinite(encoded)
    assert decoded.name == "Nested Corp"
    assert len(decoded.departments) == 0


def test_ber_indefinite_department_with_people():
    """Test indefinite encoding with list of primitive types."""
    p = Person(name="Alice", age=30, active=True, status=0)
    encoded = p.encode_ber_indefinite()
    assert encoded[:2] == bytes([0x30, 0x80])
    assert encoded[-2:] == bytes([0x00, 0x00])
    decoded = Person.decode_ber_indefinite(encoded)
    assert decoded.name == "Alice"
    assert decoded.age == 30
    assert decoded.active is True
    assert decoded.status == 0
