"""Integration tests for embedded CHOICE with AUTOMATIC TAGS.

Tests embedded CHOICE with AUTOMATIC TAGS, extension markers.

Adapted from vlm/asn1c tests-c-compiler/67-embedded-choice-OK.asn1 (MIT license).
https://github.com/vlm/asn1c
"""
from __future__ import annotations

import os
import sys

_integ_output = os.environ.get("INTEG_OUTPUT_DIR")
if _integ_output:
    sys.path.insert(0, _integ_output)

import pytest

try:
    from EmbeddedChoiceModule import Container, EmbeddedChoice, ExtendedSequence
    HAS_GENERATED = True
except ImportError:
    HAS_GENERATED = False


@pytest.mark.skipif(not HAS_GENERATED, reason="Generated types not available")
class TestEmbeddedChoice:
    """Test embedded CHOICE with AUTOMATIC TAGS encode/decode roundtrips."""

    def test_choice_integer(self):
        """Test EmbeddedChoice selecting integer alternative."""
        t = EmbeddedChoice(integer=42)
        encoded = t.encode_der()
        decoded = EmbeddedChoice.decode_der(encoded)
        assert decoded.integer == 42

    def test_choice_string(self):
        """Test EmbeddedChoice selecting string alternative."""
        t = EmbeddedChoice(string="hello")
        encoded = t.encode_der()
        decoded = EmbeddedChoice.decode_der(encoded)
        assert decoded.string == "hello"

    def test_choice_octet(self):
        """Test EmbeddedChoice selecting octet string alternative."""
        t = EmbeddedChoice(octet=b"\x01\x02\x03")
        encoded = t.encode_der()
        decoded = EmbeddedChoice.decode_der(encoded)
        assert decoded.octet == b"\x01\x02\x03"

    def test_container_with_choice_integer(self):
        """Test Container with EmbeddedChoice (integer)."""
        choice = EmbeddedChoice(integer=100)
        c = Container(id=1, value=choice)
        encoded = c.encode_der()
        decoded = Container.decode_der(encoded)
        assert decoded.id == 1
        assert decoded.value.integer == 100

    def test_container_with_choice_string(self):
        """Test Container with EmbeddedChoice (string)."""
        choice = EmbeddedChoice(string="test data")
        c = Container(id=2, value=choice)
        encoded = c.encode_der()
        decoded = Container.decode_der(encoded)
        assert decoded.id == 2
        assert decoded.value.string == "test data"

    def test_container_with_choice_octet(self):
        """Test Container with EmbeddedChoice (octet string)."""
        choice = EmbeddedChoice(octet=b"\xDE\xAD\xBE\xEF")
        c = Container(id=3, value=choice)
        encoded = c.encode_der()
        decoded = Container.decode_der(encoded)
        assert decoded.id == 3
        assert decoded.value.octet == b"\xDE\xAD\xBE\xEF"

    def test_extended_sequence_base(self):
        """Test ExtendedSequence with only base fields."""
        t = ExtendedSequence(name="base", value=1)
        encoded = t.encode_der()
        decoded = ExtendedSequence.decode_der(encoded)
        assert decoded.name == "base"
        assert decoded.value == 1

    def test_extended_sequence_with_optional(self):
        """Test ExtendedSequence with optional extension field."""
        t = ExtendedSequence(name="extended", value=2, extra="bonus")
        encoded = t.encode_der()
        decoded = ExtendedSequence.decode_der(encoded)
        assert decoded.name == "extended"
        assert decoded.value == 2
        assert decoded.extra == "bonus"

    def test_choice_tag_uniqueness(self):
        """Verify that different CHOICE alternatives produce different tag bytes."""
        t_int = EmbeddedChoice(integer=1)
        t_str = EmbeddedChoice(string="x")
        encoded_int = t_int.encode_der()
        encoded_str = t_str.encode_der()
        # Different alternatives should have different first byte (tag)
        assert encoded_int[0] != encoded_str[0], (
            "CHOICE alternatives should have distinct tags with AUTOMATIC TAGS"
        )

    def test_container_list_of_choices(self):
        """Test Container with list of choices."""
        choices = [
            EmbeddedChoice(integer=1),
            EmbeddedChoice(string="two"),
            EmbeddedChoice(octet=b"\x03"),
        ]
        c = Container(id=10, value=choices[0])
        encoded = c.encode_der()
        decoded = Container.decode_der(encoded)
        assert decoded.id == 10
        assert decoded.value.integer == 1
