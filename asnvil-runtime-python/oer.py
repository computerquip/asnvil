"""OER (Octet Encoding Rules) encode/decode primitives (Basic/Unaligned).

Basic OER (X.691) is a more compact encoding than BER/DER, primarily differing
in length encoding and some type-specific optimizations. This implementation
focuses on the unaligned (basic) variant.

OER canonicalization rules:
1. Definite length only (no indefinite length).
2. Length < 128: 1 octet containing the length.
3. Length >= 128: 2 octets (first = 128 + (length // 256), second = length % 256).
"""
from __future__ import annotations

from .errors import AsnError, IndefiniteLengthNotAllowedError, TruncatedInputError


class OerEncoder:
    """OER encoder that enforces Basic OER rules."""

    def __init__(self):
        self._buf = bytearray()

    def write_tag(self, tag_class: int, number: int, constructed: bool = False) -> None:
        # OER tag encoding is identical to BER
        if number < 31:
            tag_byte = (tag_class << 6) | (number & 0x1F)
            if constructed:
                tag_byte |= 0x20
            self._buf.append(tag_byte)
        else:
            tag_byte = (tag_class << 6) | 0x1F
            if constructed:
                tag_byte |= 0x20
            self._buf.append(tag_byte)
            num_bytes = []
            num = number
            while num > 0:
                num_bytes.insert(0, num & 0x7F)
                num >>= 7
            for i in range(len(num_bytes)):
                if i < len(num_bytes) - 1:
                    num_bytes[i] |= 0x80
            self._buf.extend(num_bytes)

    def write_length(self, length: int) -> None:
        """Write length in Basic OER format."""
        if length < 0:
            raise AsnError("Negative length")
        if length < 128:
            self._buf.append(length)
        else:
            first_byte = 128 + (length // 256)
            second_byte = length % 256
            self._buf.append(first_byte)
            self._buf.append(second_byte)

    def write_integer(self, value: int) -> None:
        # OER integer encoding is minimal two's complement, same as DER
        if value == 0:
            self._buf.append(0x00)
            return
        
        num_bytes = []
        temp = value
        if value < 0:
            while temp < -0x80 or temp > 0x7F:
                num_bytes.insert(0, temp & 0xFF)
                temp >>= 8
            num_bytes.insert(0, temp & 0xFF)
            if (num_bytes[0] & 0x80) == 0:
                num_bytes.insert(0, 0xFF)
        else:
            while temp > 0:
                num_bytes.insert(0, temp & 0xFF)
                temp >>= 8
            if num_bytes[0] & 0x80:
                num_bytes.insert(0, 0x00)
        self._buf.extend(num_bytes)

    def write_bytes(self, data: bytes) -> None:
        self._buf.extend(data)

    def write_tlv(self, tag_class: int, number: int, content: bytes, constructed: bool = False) -> None:
        self.write_tag(tag_class, number, constructed)
        self.write_length(len(content))
        self.write_bytes(content)

    def finish(self) -> bytes:
        return bytes(self._buf)


class OerDecoder:
    """OER decoder that validates Basic OER rules."""

    def __init__(self, data: bytes):
        self._data = data
        self._pos = 0

    def read_tag(self) -> tuple:
        if self._pos >= len(self._data):
            raise TruncatedInputError("Not enough data to decode tag")
        tag_byte = self._data[self._pos]
        self._pos += 1
        tag_class = (tag_byte >> 6) & 0x03
        constructed = bool(tag_byte & 0x20)
        number = tag_byte & 0x1F

        if number == 0x1F:
            number = 0
            while True:
                if self._pos >= len(self._data):
                    raise TruncatedInputError("Truncated long tag")
                byte = self._data[self._pos]
                self._pos += 1
                number = (number << 7) | (byte & 0x7F)
                if not (byte & 0x80):
                    break

        return (tag_class, number, constructed)

    def read_length(self) -> int:
        if self._pos >= len(self._data):
            raise TruncatedInputError("Not enough data to decode length")
        first = self._data[self._pos]
        self._pos += 1

        if first < 128:
            return first
        else:
            if self._pos >= len(self._data):
                raise TruncatedInputError("Truncated length")
            second = self._data[self._pos]
            self._pos += 1
            return ((first - 128) << 8) | second

    def read_integer(self) -> int:
        length = self.read_length()
        if length == 0:
            return 0
        if self._pos + length > len(self._data):
            raise TruncatedInputError("Truncated integer")

        value = 0
        for _ in range(length):
            value = (value << 8) | self._data[self._pos]
            self._pos += 1
        if value & (1 << (length * 8 - 1)):
            value -= 1 << (length * 8)
        return value

    def read_bytes(self, length: int) -> bytes:
        if self._pos + length > len(self._data):
            raise TruncatedInputError("Truncated data")
        result = self._data[self._pos : self._pos + length]
        self._pos += length
        return result

    def read_tlv(self) -> tuple:
        tag = self.read_tag()
        length = self.read_length()
        if self._pos + length > len(self._data):
            raise TruncatedInputError("Truncated TLV")
        content = self._data[self._pos : self._pos + length]
        self._pos += length
        return (tag, content, self._pos)

    def remaining(self) -> int:
        return len(self._data) - self._pos

    def at_end(self) -> bool:
        return self._pos >= len(self._data)
