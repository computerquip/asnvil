"""BER encode/decode primitives."""
from __future__ import annotations

from .errors import (
    AsnError,
    UnexpectedTagError,
    InvalidLengthError,
    TruncatedInputError,
)


class BerEncoder:
    def __init__(self):
        self._buf = bytearray()

    def write_tag(self, tag_class: int, number: int, constructed: bool = False) -> None:
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

    def write_length(self, length: int, indefinite: bool = False) -> None:
        if indefinite:
            self._buf.append(0x80)
        elif length < 0:
            raise AsnError("Negative length")
        elif length <= 127:
            self._buf.append(length)
        else:
            num_bytes = []
            while length > 0:
                num_bytes.insert(0, length & 0xFF)
                length >>= 8
            self._buf.append(0x80 | len(num_bytes))
            self._buf.extend(num_bytes)

    def write_integer(self, value: int) -> None:
        if value == 0:
            self._buf.append(0x00)
            return
        if value < 0:
            num_bytes = []
            temp = value
            while temp < -0x80 or temp > 0x7F:
                num_bytes.insert(0, temp & 0xFF)
                temp >>= 8
            if len(num_bytes) == 0 or (num_bytes[0] & 0x80) == 0:
                num_bytes.insert(0, 0xFF)
            self._buf.extend(num_bytes)
        else:
            num_bytes = []
            temp = value
            while temp > 0:
                num_bytes.insert(0, temp & 0xFF)
                temp >>= 8
            if num_bytes[0] & 0x80:
                num_bytes.insert(0, 0x00)
            self._buf.extend(num_bytes)

    def write_bytes(self, data: bytes) -> None:
        self._buf.extend(data)

    def write_eoc(self) -> None:
        """Write end-of-content marker (two zero bytes)."""
        self._buf.append(0x00)
        self._buf.append(0x00)

    def write_tlv(self, tag_class: int, number: int, content: bytes, constructed: bool = False) -> None:
        self.write_tag(tag_class, number, constructed)
        self.write_length(len(content))
        self.write_bytes(content)

    def write_tlv_indefinite(self, tag_class: int, number: int, content: bytes, constructed: bool = False) -> None:
        """Write TLV with indefinite length encoding."""
        self.write_tag(tag_class, number, constructed)
        self.write_length(0, indefinite=True)
        self.write_bytes(content)
        self.write_eoc()

    def finish(self) -> bytes:
        return bytes(self._buf)


class BerDecoder:
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

    def read_length(self) -> int | None:
        if self._pos >= len(self._data):
            raise TruncatedInputError("Not enough data to decode length")
        first = self._data[self._pos]
        self._pos += 1

        if first == 0x80:
            return None  # Indefinite length
        elif first <= 0x7F:
            return first
        else:
            num_bytes = first & 0x7F
            if self._pos + num_bytes > len(self._data):
                raise TruncatedInputError("Truncated length")
            length = 0
            for _ in range(num_bytes):
                length = (length << 8) | self._data[self._pos]
                self._pos += 1
            return length

    def read_integer(self) -> int:
        length = self.read_length()
        if length is None:
            raise InvalidLengthError("Indefinite length for INTEGER")
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
        if length is None:
            return (tag, None, self._pos)  # Indefinite length
        if self._pos + length > len(self._data):
            raise TruncatedInputError("Truncated TLV")
        content = self._data[self._pos : self._pos + length]
        self._pos += length
        return (tag, content, self._pos)

    def remaining(self) -> int:
        return len(self._data) - self._pos

    def is_eoc(self) -> bool:
        """Check if the next two bytes are an end-of-content marker."""
        return self._pos + 1 < len(self._data) and self._data[self._pos] == 0x00 and self._data[self._pos + 1] == 0x00

    def read_eoc(self) -> None:
        """Consume an end-of-content marker (two zero bytes)."""
        if not self.is_eoc():
            raise TruncatedInputError("Expected end-of-content marker")
        self._pos += 2

    def read_constructed_indefinite(self) -> bytes:
        """Read a constructed type with indefinite length until EOC marker.

        Returns the concatenated content bytes of all nested TLVs.
        """
        content = bytearray()
        while not self.is_eoc():
            tag = self.read_tag()
            length = self.read_length()
            if length is None:
                inner = self.read_constructed_indefinite()
                inner_tag = tag
                inner_e = BerEncoder()
                inner_e.write_tag(inner_tag[0], inner_tag[1], inner_tag[2])
                inner_e.write_length(0, indefinite=True)
                inner_e.write_bytes(inner)
                inner_e.write_eoc()
                content.extend(inner_e.finish())
            else:
                value = self.read_bytes(length)
                elem_e = BerEncoder()
                elem_e.write_tag(tag[0], tag[1], tag[2])
                elem_e.write_length(len(value))
                elem_e.write_bytes(value)
                content.extend(elem_e.finish())
        self.read_eoc()
        return bytes(content)

    def at_end(self) -> bool:
        return self._pos >= len(self._data)
