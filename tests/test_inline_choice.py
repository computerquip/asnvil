from TestModule import Person, PersonContact
from asn1c_runtime import BerDecoder, DerEncoder, DerDecoder


def test_person_with_email():
    p = Person(name="Alice", age=30, contact=PersonContact(email="alice@example.com"))
    encoded = p.encode_der()
    decoded = Person.decode_der(encoded)
    assert decoded.name == "Alice"
    assert decoded.age == 30
    assert decoded.contact is not None
    assert decoded.contact.email == "alice@example.com"
    assert decoded.contact.phone is None


def test_person_with_phone():
    p = Person(name="Bob", age=25, contact=PersonContact(phone="+1234567890"))
    encoded = p.encode_der()
    decoded = Person.decode_der(encoded)
    assert decoded.name == "Bob"
    assert decoded.age == 25
    assert decoded.contact is not None
    assert decoded.contact.phone == "+1234567890"
    assert decoded.contact.email is None


def test_person_no_contact():
    p = Person(name="Charlie", age=40, contact=None)
    encoded = p.encode_der()
    decoded = Person.decode_der(encoded)
    assert decoded.name == "Charlie"
    assert decoded.age == 40
    assert decoded.contact is None


def test_der_roundtrip_email():
    p = Person(name="Dave", age=50, contact=PersonContact(email="dave@test.com"))
    encoded = p.encode_der()
    decoded = Person.decode_der(encoded)
    assert decoded.contact is not None
    assert decoded.contact.email == "dave@test.com"


def test_ber_indefinite_roundtrip():
    p = Person(name="Eve", age=35, contact=PersonContact(phone="911"))
    encoded = p.encode_ber_indefinite()
    decoded = Person.decode_ber_indefinite(encoded)
    assert decoded.name == "Eve"
    assert decoded.age == 35
    assert decoded.contact is not None
    assert decoded.contact.phone == "911"


def test_choice_encode_ber():
    c = PersonContact(email="test@test.com")
    encoded = c.encode_ber()
    assert len(encoded) > 0


def test_choice_decode_der():
    c = PersonContact(phone="555-1234")
    encoded = c.encode_der()
    decoded = PersonContact.decode_der(encoded)
    assert decoded.phone == "555-1234"
    assert decoded.email is None


def test_choice_encode_der():
    c = PersonContact(email="der@test.com")
    encoded = c.encode_der()
    decoded = PersonContact.decode_der(encoded)
    assert decoded.email == "der@test.com"


def test_choice_ber_indefinite():
    c = PersonContact(phone="800-555-1234")
    encoded = c.encode_ber_indefinite()
    decoded = PersonContact.decode_ber_indefinite(encoded)
    assert decoded.phone == "800-555-1234"
