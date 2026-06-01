"""Roundtrip tests for ANY DEFINED BY fields."""
from TestModule import Container, MessageType
from asn1c_runtime import DerDecoder, TagClass


def _encode_integer(value: int) -> bytes:
    """Manually encode an INTEGER as TLV for ANY DEFINED BY."""
    if value == 0:
        value_bytes = bytes([0x00])
    elif value > 0:
        num_bytes = []
        temp = value
        while temp > 0:
            num_bytes.insert(0, temp & 0xFF)
            temp >>= 8
        if num_bytes[0] & 0x80:
            num_bytes.insert(0, 0x00)
        value_bytes = bytes(num_bytes)
    else:
        num_bytes = []
        temp = value
        while temp < -0x80 or temp > 0x7F:
            num_bytes.insert(0, temp & 0xFF)
            temp >>= 8
        num_bytes.insert(0, temp & 0xFF)
        if (num_bytes[0] & 0x80) == 0:
            num_bytes.insert(0, 0xFF)
        value_bytes = bytes(num_bytes)
    return bytes([0x02, len(value_bytes)]) + value_bytes


def _encode_boolean(value: bool) -> bytes:
    """Manually encode a BOOLEAN as TLV for ANY DEFINED BY."""
    return bytes([0x01, 0x01, 0xFF if value else 0x00])


def _encode_octet_string(value: bytes) -> bytes:
    """Manually encode an OCTET STRING as TLV for ANY DEFINED BY."""
    return bytes([0x04, len(value)]) + value


def test_any_defined_by_integer():
    """Test Container with ANY DEFINED BY holding an INTEGER (42)."""
    tlv = _encode_integer(42)
    c = Container(msgType=MessageType.integer, msgValue=tlv)
    encoded = c.encode_der()
    decoded = Container.decode_der(encoded)
    assert decoded.msgType == MessageType.integer
    assert decoded.msgValue == tlv


def test_any_defined_by_boolean():
    """Test Container with ANY DEFINED BY holding a BOOLEAN (True)."""
    tlv = _encode_boolean(True)
    c = Container(msgType=MessageType.boolean, msgValue=tlv)
    encoded = c.encode_der()
    decoded = Container.decode_der(encoded)
    assert decoded.msgType == MessageType.boolean
    assert decoded.msgValue == tlv


def test_any_defined_by_negative_integer():
    """Test Container with ANY DEFINED BY holding a negative INTEGER (-128)."""
    tlv = _encode_integer(-128)
    c = Container(msgType=MessageType.integer, msgValue=tlv)
    encoded = c.encode_der()
    decoded = Container.decode_der(encoded)
    assert decoded.msgType == MessageType.integer
    assert decoded.msgValue == tlv


def test_any_defined_by_large_integer():
    """Test Container with ANY DEFINED BY holding a large INTEGER (32767)."""
    tlv = _encode_integer(32767)
    c = Container(msgType=MessageType.integer, msgValue=tlv)
    encoded = c.encode_der()
    decoded = Container.decode_der(encoded)
    assert decoded.msgType == MessageType.integer
    assert decoded.msgValue == tlv


def test_any_defined_by_different_types_distinguishable():
    """Verify that INTEGER and BOOLEAN ANY DEFINED BY produce different encodings."""
    int_tlv = _encode_integer(0)
    bool_tlv = _encode_boolean(True)
    c_int = Container(msgType=MessageType.integer, msgValue=int_tlv)
    c_bool = Container(msgType=MessageType.boolean, msgValue=bool_tlv)
    encoded_int = c_int.encode_der()
    encoded_bool = c_bool.encode_der()
    assert encoded_int != encoded_bool
