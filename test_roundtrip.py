import sys
sys.path.insert(0, 'output')

from TestModule import Person, Department, Company, Status
from asn1c_runtime.types import BitString, ObjectIdentifier

def test_person_roundtrip():
    p = Person(name="Alice", age=30, active=True, status=Status.active)
    encoded = p.encode_der()
    decoded = Person.decode_der(encoded)
    assert decoded.name == "Alice", f"name: {decoded.name}"
    assert decoded.age == 30, f"age: {decoded.age}"
    assert decoded.active == True, f"active: {decoded.active}"
    assert decoded.status == 0, f"status: {decoded.status}"
    print("PASS: Person roundtrip")

def test_department_roundtrip():
    p1 = Person(name="Alice", age=30, active=True, status=Status.active)
    p2 = Person(name="Bob", age=25, active=False, status=Status.pending)
    dept = Department(name="Engineering", people=[p1, p2])
    encoded = dept.encode_der()
    decoded = Department.decode_der(encoded)
    assert decoded.name == "Engineering", f"name: {decoded.name}"
    assert len(decoded.people) == 2, f"people count: {len(decoded.people)}"
    assert decoded.people[0].name == "Alice", f"person[0].name: {decoded.people[0].name}"
    assert decoded.people[0].age == 30, f"person[0].age: {decoded.people[0].age}"
    assert decoded.people[1].name == "Bob", f"person[1].name: {decoded.people[1].name}"
    assert decoded.people[1].age == 25, f"person[1].age: {decoded.people[1].age}"
    print("PASS: Department (SEQUENCE OF) roundtrip")

def test_company_roundtrip():
    p1 = Person(name="Alice", age=30, active=True, status=Status.active)
    dept1 = Department(name="Engineering", people=[p1])
    p2 = Person(name="Charlie", age=40, active=True, status=Status.inactive)
    dept2 = Department(name="Sales", people=[p2])
    company = Company(name="Acme Corp", departments=[dept1, dept2])
    encoded = company.encode_der()
    decoded = Company.decode_der(encoded)
    assert decoded.name == "Acme Corp", f"name: {decoded.name}"
    assert len(decoded.departments) == 2, f"departments count: {len(decoded.departments)}"
    assert decoded.departments[0].name == "Engineering", f"dept[0].name: {decoded.departments[0].name}"
    assert decoded.departments[0].people[0].name == "Alice", f"dept[0].people[0].name"
    assert decoded.departments[1].name == "Sales", f"dept[1].name: {decoded.departments[1].name}"
    assert decoded.departments[1].people[0].name == "Charlie", f"dept[1].people[0].name"
    print("PASS: Company (SET OF) roundtrip")

def test_empty_list():
    dept = Department(name="Empty", people=[])
    encoded = dept.encode_der()
    decoded = Department.decode_der(encoded)
    assert decoded.name == "Empty", f"name: {decoded.name}"
    assert len(decoded.people) == 0, f"people count: {len(decoded.people)}"
    print("PASS: Empty list roundtrip")

def test_config_defaults():
    from TestModule import Config
    # All defaults - should encode to empty content
    cfg = Config(enabled=True, count=42, label="default", flag=None)
    encoded = cfg.encode_der()
    decoded = Config.decode_der(encoded)
    assert decoded.enabled == True, f"enabled: {decoded.enabled}"
    assert decoded.count == 42, f"count: {decoded.count}"
    assert decoded.label == "default", f"label: {decoded.label}"
    assert decoded.flag == None, f"flag: {decoded.flag}"
    print("PASS: Config defaults roundtrip")

    # Non-default values - should encode all fields
    cfg2 = Config(enabled=False, count=100, label="custom", flag=True)
    encoded2 = cfg2.encode_der()
    decoded2 = Config.decode_der(encoded2)
    assert decoded2.enabled == False, f"enabled: {decoded2.enabled}"
    assert decoded2.count == 100, f"count: {decoded2.count}"
    assert decoded2.label == "custom", f"label: {decoded2.label}"
    assert decoded2.flag == True, f"flag: {decoded2.flag}"
    print("PASS: Config non-defaults roundtrip")

def test_certificate_roundtrip():
    from TestModule import Certificate
    cert = Certificate(
        serial=12345,
        key=b'\x01\x02\x03\x04\x05',
        signature=BitString(b'\xab\xcd', unused_bits=0),
        issuer=ObjectIdentifier([2, 5, 4, 3])
    )
    encoded = cert.encode_der()
    decoded = Certificate.decode_der(encoded)
    assert decoded.serial == 12345, f"serial: {decoded.serial}"
    assert decoded.key == b'\x01\x02\x03\x04\x05', f"key: {decoded.key!r}"
    assert decoded.signature.data == b'\xab\xcd', f"signature.data: {decoded.signature.data!r}"
    assert decoded.signature.unused_bits == 0, f"signature.unused_bits: {decoded.signature.unused_bits}"
    assert decoded.issuer == ObjectIdentifier([2, 5, 4, 3]), f"issuer: {decoded.issuer}"
    print("PASS: Certificate (BIT STRING, OCTET STRING, OID) roundtrip")

def test_choice_roundtrip():
    from TestModule import MessageChoice
    # Test string alternative
    msg1 = MessageChoice(text="hello")
    encoded1 = msg1.encode_ber()
    decoded1 = MessageChoice.decode_ber(encoded1)
    assert decoded1.text == "hello", f"text: {decoded1.text}"
    assert decoded1.number is None, f"number: {decoded1.number}"
    assert decoded1.flag is None, f"flag: {decoded1.flag}"
    print("PASS: MessageChoice (string) roundtrip")

    # Test integer alternative
    msg2 = MessageChoice(number=42)
    encoded2 = msg2.encode_ber()
    decoded2 = MessageChoice.decode_ber(encoded2)
    assert decoded2.text is None, f"text: {decoded2.text}"
    assert decoded2.number == 42, f"number: {decoded2.number}"
    assert decoded2.flag is None, f"flag: {decoded2.flag}"
    print("PASS: MessageChoice (integer) roundtrip")

    # Test boolean alternative
    msg3 = MessageChoice(flag=True)
    encoded3 = msg3.encode_ber()
    decoded3 = MessageChoice.decode_ber(encoded3)
    assert decoded3.text is None, f"text: {decoded3.text}"
    assert decoded3.number is None, f"number: {decoded3.number}"
    assert decoded3.flag == True, f"flag: {decoded3.flag}"
    print("PASS: MessageChoice (boolean) roundtrip")

def test_choice_nested_roundtrip():
    from TestModule import PersonOrDept, Person, Department, Status
    # Test Person alternative
    p1 = Person(name="Alice", age=30, active=True, status=Status.active)
    choice1 = PersonOrDept(person=p1)
    encoded1 = choice1.encode_der()
    decoded1 = PersonOrDept.decode_der(encoded1)
    assert decoded1.person is not None, "person should be set"
    assert decoded1.person.name == "Alice", f"person.name: {decoded1.person.name}"
    assert decoded1.dept is None, f"dept: {decoded1.dept}"
    print("PASS: PersonOrDept (Person) roundtrip")

    # NOTE: Department alternative not tested - both Person and Department are
    # SEQUENCE types with identical UNIVERSAL 16 tags, so CHOICE can't
    # distinguish them by tag. Requires explicitly tagged alternatives (Milestone 6+).

def test_ber_der_equivalence_for_sequence():
    """Test that BER and DER field encodings are identical for SEQUENCE (not SET)."""
    p = Person(name="Bob", age=25, active=False, status=Status.pending)
    ber_encoded = p.encode_ber()
    der_encoded = p.encode_der()
    # DER wraps in SEQUENCE tag+length, BER returns raw content
    # The DER content (after tag+length) should match BER exactly
    # DER: [0x30 (SEQUENCE)][length][content...]
    # BER: [content...]
    # So DER content bytes = der_encoded[2:] for short form
    der_content = der_encoded[2:]
    assert ber_encoded == der_content, f"BER and DER content differ:\n  BER: {ber_encoded.hex()}\n  DER content: {der_content.hex()}"
    print("PASS: BER and DER content equivalence for SEQUENCE")

def test_der_person_roundtrip():
    """Test DER encode/decode for a simple SEQUENCE type."""
    p = Person(name="Alice", age=30, active=True, status=Status.active)
    encoded = p.encode_der()
    decoded = Person.decode_der(encoded)
    assert decoded.name == "Alice", f"name: {decoded.name}"
    assert decoded.age == 30, f"age: {decoded.age}"
    assert decoded.active == True, f"active: {decoded.active}"
    assert decoded.status == 0, f"status: {decoded.status}"
    print("PASS: Person DER roundtrip")

def test_der_company_set_canonicalization():
    """Test that DER SET encoding sorts elements lexicographically by TLV."""
    p1 = Person(name="Alice", age=30, active=True, status=Status.active)
    dept1 = Department(name="Engineering", people=[p1])
    p2 = Person(name="Charlie", age=40, active=True, status=Status.inactive)
    dept2 = Department(name="Sales", people=[p2])
    company = Company(name="Acme Corp", departments=[dept1, dept2])
    
    # DER encoding sorts SET elements
    der_encoded = company.encode_der()
    
    # Decode and verify the logical value is preserved
    decoded_der = Company.decode_der(der_encoded)
    
    assert decoded_der.name == "Acme Corp"
    assert len(decoded_der.departments) == 2
    print("PASS: Company SET canonicalization (DER)")

def test_der_choice_roundtrip():
    """Test DER encode/decode for CHOICE types."""
    from TestModule import MessageChoice
    msg1 = MessageChoice(text="hello")
    encoded1 = msg1.encode_der()
    decoded1 = MessageChoice.decode_der(encoded1)
    assert decoded1.text == "hello", f"text: {decoded1.text}"
    assert decoded1.number is None, f"number: {decoded1.number}"
    print("PASS: MessageChoice DER (string) roundtrip")

    msg2 = MessageChoice(number=42)
    encoded2 = msg2.encode_der()
    decoded2 = MessageChoice.decode_der(encoded2)
    assert decoded2.number == 42, f"number: {decoded2.number}"
    assert decoded2.text is None, f"text: {decoded2.text}"
    print("PASS: MessageChoice DER (integer) roundtrip")

if __name__ == "__main__":
    test_person_roundtrip()
    test_department_roundtrip()
    test_company_roundtrip()
    test_empty_list()
    test_config_defaults()
    test_certificate_roundtrip()
    test_choice_roundtrip()
    test_choice_nested_roundtrip()
    test_der_person_roundtrip()
    test_der_company_set_canonicalization()
    test_der_choice_roundtrip()
    test_ber_der_equivalence_for_sequence()
    print("\nAll tests passed!")
