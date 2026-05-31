"""Roundtrip tests for explicitly tagged CHOICE alternatives."""
import sys
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from asn1c_runtime import DerEncoder, DerDecoder
from TestModule import Person, Department, Entity, Container, MixedChoice

def test_entity_person_roundtrip():
    """Test Entity CHOICE with Person alternative (explicitly tagged [0])."""
    person = Person(name="Alice", age=30, active=True)
    entity = Entity(person=person)
    encoded = entity.encode_der()
    decoded = Entity.decode_der(encoded)
    assert decoded.person is not None
    assert decoded.person.name == "Alice"
    assert decoded.person.age == 30
    assert decoded.person.active == True
    assert decoded.department is None
    assert decoded.flag is None
    # Verify the outer tag is CONTEXT 0
    dec = DerDecoder(encoded)
    tag = dec.read_tag()
    assert tag[0] == 2, f"Expected CONTEXT-SPECIFIC tag class (2), got {tag[0]}"  # Context-specific = 2
    assert tag[1] == 0, f"Expected tag number 0, got {tag[1]}"
    assert tag[2] == True, "Expected constructed bit set for explicit tagging"

def test_entity_department_roundtrip():
    """Test Entity CHOICE with Department alternative (explicitly tagged [1])."""
    dept = Department(deptName="Engineering", code=100, location="Building A")
    entity = Entity(department=dept)
    encoded = entity.encode_der()
    decoded = Entity.decode_der(encoded)
    assert decoded.department is not None
    assert decoded.department.deptName == "Engineering"
    assert decoded.department.code == 100
    assert decoded.department.location == "Building A"
    assert decoded.person is None
    assert decoded.flag is None
    # Verify the outer tag is CONTEXT 1
    dec = DerDecoder(encoded)
    tag = dec.read_tag()
    assert tag[0] == 2
    assert tag[1] == 1

def test_entity_flag_roundtrip():
    """Test Entity CHOICE with BOOLEAN alternative (explicitly tagged [2])."""
    entity = Entity(flag=True)
    encoded = entity.encode_der()
    decoded = Entity.decode_der(encoded)
    assert decoded.flag == True
    assert decoded.person is None
    assert decoded.department is None
    # Verify the outer tag is CONTEXT 2
    dec = DerDecoder(encoded)
    tag = dec.read_tag()
    assert tag[0] == 2
    assert tag[1] == 2

def test_container_with_entity():
    """Test Container SEQUENCE with Entity CHOICE field."""
    dept = Department(deptName="Sales", code=200, location="Building B")
    entity = Entity(department=dept)
    container = Container(id=42, entity=entity)
    encoded = container.encode_der()
    decoded = Container.decode_der(encoded)
    assert decoded.id == 42
    assert decoded.entity is not None
    assert decoded.entity.department is not None
    assert decoded.entity.department.deptName == "Sales"
    assert decoded.entity.department.code == 200

def test_mixed_choice_person():
    """Test MixedChoice with explicitly tagged Person alternative [0]."""
    person = Person(name="Bob", age=25, active=False)
    choice = MixedChoice(item=person)
    encoded = choice.encode_der()
    decoded = MixedChoice.decode_der(encoded)
    assert decoded.item is not None
    assert decoded.item.name == "Bob"
    assert decoded.item.age == 25
    assert decoded.count is None
    assert decoded.label is None

def test_mixed_choice_integer():
    """Test MixedChoice with inherent INTEGER alternative."""
    choice = MixedChoice(count=42)
    encoded = choice.encode_der()
    decoded = MixedChoice.decode_der(encoded)
    assert decoded.count == 42
    assert decoded.item is None
    assert decoded.label is None

def test_mixed_choice_string():
    """Test MixedChoice with inherent UTF8String alternative."""
    choice = MixedChoice(label="test")
    encoded = choice.encode_der()
    decoded = MixedChoice.decode_der(encoded)
    assert decoded.label == "test"
    assert decoded.count is None
    assert decoded.item is None

def test_entity_different_alternatives_distinguishable():
    """Verify that Person and Department alternatives produce different encodings."""
    person = Person(name="Test", age=1, active=True)
    dept = Department(deptName="Test", code=1, location="Test")
    entity_person = Entity(person=person)
    entity_dept = Entity(department=dept)
    encoded_person = entity_person.encode_der()
    encoded_dept = entity_dept.encode_der()
    # The outer tags should be different (CONTEXT 0 vs CONTEXT 1)
    assert encoded_person[0] != encoded_dept[0], "Person and Department should have different outer tags"

def test_entity_deep_nesting():
    """Test Entity with Person containing complex data."""
    person = Person(name="Deep Nested Test", age=999, active=True)
    entity = Entity(person=person)
    container = Container(id=1, entity=entity)
    encoded = container.encode_der()
    decoded = Container.decode_der(encoded)
    assert decoded.id == 1
    assert decoded.entity.person is not None
    assert decoded.entity.person.name == "Deep Nested Test"
    assert decoded.entity.person.age == 999
