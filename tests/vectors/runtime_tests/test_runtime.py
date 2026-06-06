"""Unit tests for asnvil Python runtime (ber.py, der.py, types.py, errors.py)."""
from __future__ import annotations

import pytest

from asnvil_runtime.ber import BerEncoder, BerDecoder
from asnvil_runtime.der import DerEncoder, DerDecoder
from asnvil_runtime.errors import (
    AsnError,
    UnexpectedTagError,
    InvalidLengthError,
    TruncatedInputError,
    IndefiniteLengthNotAllowedError,
    InvalidIntegerEncodingError,
    SetNotCanonicalError,
)
from asnvil_runtime.types import BitString, ObjectIdentifier, AsnAny
from asnvil_runtime import Tag, TagClass


# ─── BerEncoder tests ───────────────────────────────────────────────

def test_encode_tag_simple():
    e = BerEncoder()
    e.write_tag(TagClass.UNIVERSAL, 2, constructed=False)
    assert e.finish() == bytes([0x02])

    e = BerEncoder()
    e.write_tag(TagClass.UNIVERSAL, 16, constructed=True)
    assert e.finish() == bytes([0x30])

    e = BerEncoder()
    e.write_tag(TagClass.CONTEXT, 0, constructed=False)
    assert e.finish() == bytes([0x80])


def test_encode_tag_long():
    e = BerEncoder()
    e.write_tag(TagClass.UNIVERSAL, 31, constructed=False)
    assert e.finish() == bytes([0x1F, 0x1F])

    e = BerEncoder()
    e.write_tag(TagClass.UNIVERSAL, 128, constructed=False)
    assert e.finish() == bytes([0x1F, 0x81, 0x00])


def test_encode_length_short():
    e = BerEncoder()
    e.write_length(0)
    assert e.finish() == bytes([0x00])

    e = BerEncoder()
    e.write_length(127)
    assert e.finish() == bytes([0x7F])


def test_encode_length_long():
    e = BerEncoder()
    e.write_length(128)
    assert e.finish() == bytes([0x81, 0x80])

    e = BerEncoder()
    e.write_length(256)
    assert e.finish() == bytes([0x82, 0x01, 0x00])


def test_encode_length_indefinite():
    e = BerEncoder()
    e.write_length(0, indefinite=True)
    assert e.finish() == bytes([0x80])


def test_encode_integer_zero():
    e = BerEncoder()
    e.write_integer(0)
    assert e.finish() == bytes([0x00])


def test_encode_integer_positive():
    for value, expected in [
        (1, bytes([0x01])),
        (127, bytes([0x7F])),
        (128, bytes([0x00, 0x80])),
        (255, bytes([0x00, 0xFF])),
        (256, bytes([0x01, 0x00])),
        (32767, bytes([0x7F, 0xFF])),
    ]:
        e = BerEncoder()
        e.write_integer(value)
        assert e.finish() == expected, f"encode({value}) failed"


def test_encode_integer_negative():
    for value, expected in [
        (-1, bytes([0xFF])),
        (-128, bytes([0x80])),
        (-129, bytes([0xFF, 0x7F])),
        (-32768, bytes([0x80, 0x00])),
    ]:
        e = BerEncoder()
        e.write_integer(value)
        assert e.finish() == expected, f"encode({value}) failed"


def test_encode_integer_negative_roundtrip():
    for value in [-1, -128, -129, -32768, -65536, -1000]:
        int_e = BerEncoder()
        int_e.write_integer(value)
        int_bytes = int_e.finish()
        e = BerEncoder()
        e.write_tag(0, 2)
        e.write_length(len(int_bytes))
        e.write_bytes(int_bytes)
        d = BerDecoder(e.finish())
        d.read_tag()
        decoded = d.read_integer()
        assert decoded == value, f"roundtrip({value}) failed"


def test_write_tlv():
    e = BerEncoder()
    e.write_tlv(TagClass.UNIVERSAL, 2, bytes([0x01]))
    assert e.finish() == bytes([0x02, 0x01, 0x01])


def test_write_eoc():
    e = BerEncoder()
    e.write_eoc()
    assert e.finish() == bytes([0x00, 0x00])


def test_write_tlv_indefinite():
    e = BerEncoder()
    e.write_tlv_indefinite(TagClass.UNIVERSAL, 16, bytes([0x02, 0x01, 0x01]), constructed=True)
    result = e.finish()
    assert result[0] == 0x30
    assert result[1] == 0x80
    assert result[-2:] == bytes([0x00, 0x00])


# ─── BerDecoder tests ───────────────────────────────────────────────

def test_decode_tag_simple():
    d = BerDecoder(bytes([0x02]))
    assert d.read_tag() == (0, 2, False)

    d = BerDecoder(bytes([0x30]))
    assert d.read_tag() == (0, 16, True)


def test_decode_tag_long():
    d = BerDecoder(bytes([0x1F, 0x1F]))
    assert d.read_tag() == (0, 31, False)

    d = BerDecoder(bytes([0x1F, 0x81, 0x00]))
    assert d.read_tag() == (0, 128, False)


def test_decode_length_short():
    d = BerDecoder(bytes([0x00]))
    assert d.read_length() == 0

    d = BerDecoder(bytes([0x7F]))
    assert d.read_length() == 127


def test_decode_length_long():
    d = BerDecoder(bytes([0x81, 0x80]))
    assert d.read_length() == 128

    d = BerDecoder(bytes([0x82, 0x01, 0x00]))
    assert d.read_length() == 256


def test_decode_length_indefinite():
    d = BerDecoder(bytes([0x80]))
    assert d.read_length() is None


def test_decode_integer_various():
    for data, expected in [
        (bytes([0x02, 0x01, 0x00]), 0),
        (bytes([0x02, 0x01, 0x01]), 1),
        (bytes([0x02, 0x02, 0x00, 0x80]), 128),
        (bytes([0x02, 0x01, 0xFF]), -1),
        (bytes([0x02, 0x01, 0x80]), -128),
    ]:
        d = BerDecoder(data)
        d.read_tag()
        assert d.read_integer() == expected, f"decode({data.hex()}) failed"


def test_read_bytes():
    d = BerDecoder(bytes([0x01, 0x02, 0x03]))
    assert d.read_bytes(2) == bytes([0x01, 0x02])
    assert d.read_bytes(1) == bytes([0x03])


def test_read_tlv():
    d = BerDecoder(bytes([0x02, 0x01, 0x05]))
    tag, content, pos = d.read_tlv()
    assert tag == (0, 2, False)
    assert content == bytes([0x05])
    assert pos == 3


def test_truncated_tag():
    d = BerDecoder(bytes([]))
    with pytest.raises(TruncatedInputError):
        d.read_tag()


def test_truncated_length():
    d = BerDecoder(bytes([0x02]))
    d.read_tag()
    with pytest.raises(TruncatedInputError):
        d.read_length()


def test_truncated_data():
    d = BerDecoder(bytes([0x01, 0x02]))
    with pytest.raises(TruncatedInputError):
        d.read_bytes(3)


# ─── DerEncoder tests ───────────────────────────────────────────────

def test_der_no_indefinite():
    e = DerEncoder()
    with pytest.raises(IndefiniteLengthNotAllowedError):
        e.write_length(0, indefinite=True)


def test_der_integer_minimal():
    e = DerEncoder()
    e.write_integer(0)
    assert e.finish() == bytes([0x00])

    e = DerEncoder()
    e.write_integer(127)
    assert e.finish() == bytes([0x7F])

    e = DerEncoder()
    e.write_integer(-128)
    assert e.finish() == bytes([0x80])


def test_der_boolean():
    e = DerEncoder()
    e.write_boolean(True)
    assert e.finish() == bytes([0x01, 0x01, 0xFF])

    e = DerEncoder()
    e.write_boolean(False)
    assert e.finish() == bytes([0x01, 0x01, 0x00])


def test_write_tlv_der():
    e = DerEncoder()
    e.write_tlv_der(TagClass.UNIVERSAL, 2, bytes([0x01]))
    assert e.finish() == bytes([0x02, 0x01, 0x01])


# ─── DerDecoder tests ───────────────────────────────────────────────

def test_der_rejects_indefinite():
    d = DerDecoder(bytes([0x80]))
    with pytest.raises(IndefiniteLengthNotAllowedError):
        d.read_length()


def test_der_rejects_non_minimal_int():
    data = bytes([0x02, 0x02, 0x00, 0x01])
    d = DerDecoder(data)
    d.read_tag()
    with pytest.raises(InvalidIntegerEncodingError):
        d.read_integer()


def test_der_rejects_non_minimal_negative_int():
    data = bytes([0x02, 0x02, 0xFF, 0xFF])
    d = DerDecoder(data)
    d.read_tag()
    with pytest.raises(InvalidIntegerEncodingError):
        d.read_integer()


def test_der_boolean_validation():
    data = bytes([0x01, 0x01, 0x01])
    d = DerDecoder(data)
    d.read_tag()
    with pytest.raises(InvalidIntegerEncodingError):
        d.read_boolean()


def test_der_length_validation():
    e = DerEncoder()
    with pytest.raises(IndefiniteLengthNotAllowedError):
        e.write_length(0, indefinite=True)


def test_read_set_elements_canonical():
    encoded = bytes([
        0x02, 0x01, 0x01,
        0x02, 0x01, 0x02,
    ])
    tlv_list = DerDecoder(encoded).read_set_elements(0, 17, 6)
    assert len(tlv_list) == 2


def test_read_set_elements_non_canonical():
    encoded = bytes([
        0x02, 0x01, 0x02,
        0x02, 0x01, 0x01,
    ])
    d = DerDecoder(encoded)
    with pytest.raises(SetNotCanonicalError):
        d.read_set_elements(0, 17, 6)


# ─── BitString/ObjectIdentifier tests ───────────────────────────────

def test_bitstring_roundtrip():
    bs = BitString(bytes([0xAB, 0xC0]), unused_bits=4)
    assert bs.data == bytes([0xAB, 0xC0])
    assert bs.unused_bits == 4


def test_objectidentifier_roundtrip():
    oid = ObjectIdentifier([1, 2, 840, 113549])
    encoded = oid.encode()
    decoded, consumed = ObjectIdentifier.decode(encoded)
    assert decoded.components == [1, 2, 840, 113549]
    assert consumed == len(encoded)


def test_objectidentifier_str():
    oid = ObjectIdentifier([1, 2, 840])
    assert repr(oid) == "ObjectIdentifier([1, 2, 840])"


# ─── Error hierarchy tests ─────────────────────────────────────────

def test_error_inheritance():
    assert issubclass(UnexpectedTagError, AsnError)
    assert issubclass(InvalidLengthError, AsnError)
    assert issubclass(TruncatedInputError, AsnError)
    assert issubclass(IndefiniteLengthNotAllowedError, AsnError)
    assert issubclass(InvalidIntegerEncodingError, AsnError)
    assert issubclass(SetNotCanonicalError, AsnError)


# ─── Tag class tests ────────────────────────────────────────────────

def test_tag_encode_simple():
    t = Tag(tag_class=TagClass.UNIVERSAL, number=2)
    assert t.encode() == bytes([0x02])


def test_tag_decode_simple():
    tag, pos = Tag.decode(bytes([0x02]))
    assert tag.tag_class == TagClass.UNIVERSAL
    assert tag.number == 2
    assert pos == 1


def test_tag_roundtrip():
    t = Tag(tag_class=TagClass.CONTEXT, number=5, constructed=True)
    encoded = t.encode()
    decoded, _ = Tag.decode(encoded)
    assert decoded.tag_class == TagClass.CONTEXT
    assert decoded.number == 5
    assert decoded.constructed is True


# ─── AsnAny tests ───────────────────────────────────────────────────

def test_asn_any_stores_data():
    a = AsnAny(tag_class=0, number=4, content=bytes([0x01, 0x02]))
    assert a.tag_class == 0
    assert a.number == 4
    assert a.content == bytes([0x01, 0x02])


# ─── Error handling / edge case tests ───────────────────────────────

def test_decode_empty_input():
    d = BerDecoder(bytes([]))
    with pytest.raises(TruncatedInputError):
        d.read_tag()


def test_decode_truncated_length():
    d = BerDecoder(bytes([0x02, 0x82]))
    d.read_tag()
    with pytest.raises(TruncatedInputError):
        d.read_length()


def test_decode_truncated_integer():
    d = BerDecoder(bytes([0x02, 0x02, 0x01]))
    d.read_tag()
    with pytest.raises(TruncatedInputError):
        d.read_integer()


def test_decode_truncated_tlv():
    d = BerDecoder(bytes([0x02, 0x02, 0x01]))
    with pytest.raises(TruncatedInputError):
        d.read_tlv()


def test_ber_decode_negative_integer():
    for data, expected in [
        (bytes([0x02, 0x01, 0xFF]), -1),
        (bytes([0x02, 0x01, 0x80]), -128),
        (bytes([0x02, 0x02, 0xFF, 0x7F]), -129),
        (bytes([0x02, 0x02, 0x80, 0x00]), -32768),
    ]:
        d = BerDecoder(data)
        d.read_tag()
        assert d.read_integer() == expected, f"decode({data.hex()}) failed"


def test_der_decode_negative_integer_minimal():
    """DER decoder should accept minimal negative integer encodings."""
    for data, expected in [
        (bytes([0x02, 0x01, 0xFF]), -1),
        (bytes([0x02, 0x01, 0x80]), -128),
        (bytes([0x02, 0x02, 0xFF, 0x7F]), -129),
    ]:
        d = DerDecoder(data)
        d.read_tag()
        assert d.read_integer() == expected, f"decode({data.hex()}) failed"


def test_der_decode_rejects_non_minimal_negative():
    """DER decoder should reject non-minimal negative integer (leading 0xFF)."""
    data = bytes([0x02, 0x03, 0xFF, 0xFF, 0x7F])
    d = DerDecoder(data)
    d.read_tag()
    with pytest.raises(InvalidIntegerEncodingError):
        d.read_integer()


def test_bitstring_invalid_unused_bits():
    with pytest.raises(AsnError):
        BitString(bytes([0x00]), unused_bits=8)


def test_bitstring_unused_bits_must_be_zero():
    with pytest.raises(AsnError):
        BitString(bytes([0x01]), unused_bits=4)


def test_objectidentifier_too_short():
    with pytest.raises(AsnError):
        ObjectIdentifier.decode(bytes([]))


def test_objectidentifier_truncated():
    # 0x81 starts a multi-byte component but no continuation follows
    with pytest.raises(AsnError):
        ObjectIdentifier.decode(bytes([0x01, 0x81]))


def test_bitstring_eq():
    bs1 = BitString(bytes([0xA0]), unused_bits=4)
    bs2 = BitString(bytes([0xA0]), unused_bits=4)
    bs3 = BitString(bytes([0xA0]), unused_bits=0)
    assert bs1 == bs2
    assert bs1 != bs3


def test_asn_any_repr():
    a = AsnAny(tag_class=0, number=4, content=bytes([0x01, 0x02]))
    assert a.tag_class == 0
    assert a.number == 4
    assert a.content == bytes([0x01, 0x02])
