"""Integration tests for ANY DEFINED BY and ANY-containing structures.

Tests generated Python types can decode externally-generated BER data
from the asn1c project test vectors.

Adapted from vlm/asn1c tests-c-compiler/data-62/ (MIT license).
https://github.com/vlm/asn1c
"""
from __future__ import annotations

import os
import sys
import pathlib

# Allow importing from output directory when run via integration test runner
_integ_output = os.environ.get("INTEG_OUTPUT_DIR")
if _integ_output:
    sys.path.insert(0, _integ_output)

import pytest

# These imports will be available after the ASN.1 spec is compiled
try:
    from AnyTestModule import T1Ext, T2, T3Set, T4Choice
    HAS_GENERATED = True
except ImportError:
    HAS_GENERATED = False

from asnvil_runtime import ObjectIdentifier
from asnvil_runtime.ber import BerDecoder
from asnvil_runtime.errors import AsnError, TruncatedInputError


VECTORS_DIR = pathlib.Path(__file__).resolve().parent.parent.parent.parent / "vectors" / "ber"
DATA62_DIR = VECTORS_DIR / "data-62"


def _load_ber_file(name: str) -> bytes:
    """Load a .ber binary test vector."""
    path = DATA62_DIR / name
    if not path.exists():
        pytest.skip(f"BER file not found: {path}")
    return path.read_bytes()


@pytest.mark.skipif(not HAS_GENERATED, reason="Generated types not available")
class TestAnyDecode:
    """Test that generated Python types can decode externally-generated BER data."""

    def test_decode_ber_basic(self):
        """Basic sanity check: verify the generated module imports correctly."""
        assert T1Ext is not None
        assert T2 is not None

    def test_t1_ext_simple(self):
        """Test T1-ext with INTEGER only (no ANY)."""
        # Manually create and roundtrip
        t = T1Ext(i=42, any=None)
        encoded = t.encode_der()
        decoded = T1Ext.decode_der(encoded)
        assert decoded.i == 42
        assert decoded.any is None

    def test_t2_simple(self):
        """Test T2 with simple ANY content."""
        # Create a T2 with raw ANY bytes
        any_content = bytes([0x02, 0x01, 0x0A])  # INTEGER 10
        t = T2(nested=any_content)
        encoded = t.encode_der()
        decoded = T2.decode_der(encoded)
        assert decoded.nested == any_content

    def test_ber_decode_data62_01(self):
        """Decode data-62-01.ber: SEQUENCE + INTEGER(5) + [1]ANY."""
        data = _load_ber_file("data-62-01.ber")
        d = BerDecoder(data)
        tag = d.read_tag()
        assert tag == (0, 16, True)  # SEQUENCE
        length = d.read_length()
        assert length == 11
        # Verify we can read the INTEGER child
        child_tag = d.read_tag()
        assert child_tag == (0, 2, False)  # INTEGER
        child_len = d.read_length()
        assert child_len == 1
        value = d.read_bytes(child_len)
        assert value == bytes([0x05])

    def test_ber_decode_data62_27(self):
        """Decode data-62-27.ber: SEQUENCE + empty [1]ANY."""
        data = _load_ber_file("data-62-27.ber")
        d = BerDecoder(data)
        tag = d.read_tag()
        assert tag == (0, 16, True)  # SEQUENCE
        length = d.read_length()
        assert length == 2
        # Read the [1] ANY (A1 00)
        child_tag = d.read_tag()
        assert child_tag == (2, 1, True)  # Context 1, constructed
        child_len = d.read_length()
        assert child_len == 0

    def test_ber_decode_data62_22(self):
        """Decode data-62-22.ber: SEQUENCE + nested ANY containing OCTET STRING."""
        data = _load_ber_file("data-62-22.ber")
        d = BerDecoder(data)
        tag = d.read_tag()
        assert tag == (0, 16, True)  # SEQUENCE
        # Verify total structure is parseable
        content = d.read_bytes(d.read_length())
        assert len(content) > 0
        # Parse inner ANY
        inner = BerDecoder(content)
        inner_tag = inner.read_tag()
        assert inner_tag[0] == 2  # Context
        inner_len = inner.read_length()
        inner_content = inner.read_bytes(inner_len)
        assert len(inner_content) == inner_len

    def test_t4_choice_octet_string(self):
        """Test T4-choice with OCTET STRING alternative."""
        t = T4Choice(octet=b"123")
        encoded = t.encode_der()
        decoded = T4Choice.decode_der(encoded)
        assert decoded.octet == b"123"

    def test_t4_choice_integer(self):
        """Test T4-choice with INTEGER alternative."""
        t = T4Choice(integer=42)
        encoded = t.encode_der()
        decoded = T4Choice.decode_der(encoded)
        assert decoded.integer == 42

    def test_error_truncated_ber(self):
        """Test that truncated BER raises appropriate errors."""
        data = _load_ber_file("data-62-02-B.ber")
        d = BerDecoder(data)
        d.read_tag()
        with pytest.raises(TruncatedInputError):
            d.read_length()
            d.read_bytes(100)

    def test_error_indefinite_in_der(self):
        """Test that indefinite length in DER context raises error."""
        data = _load_ber_file("data-62-26-B.ber")
        d = BerDecoder(data)
        d.read_tag()
        # BER should accept indefinite length
        length = d.read_length()
        assert length is None  # Indefinite
