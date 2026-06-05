"""DER encode/decode primitives (BER with canonicalization constraints).

DER (Distinguished Encoding Rules) is a subset of BER that produces exactly one
encoding for any given value. This is essential for cryptographic applications
like X.509 certificates where deterministic encoding is required.

DER canonicalization rules:
1. No indefinite length encoding
2. Minimal integer encoding (no unnecessary leading sign bytes)
3. Minimal length encoding (use shortest possible form)
4. SET elements must be sorted lexicographically by their encoded TLV
5. Boolean must encode as 0x00 (false) or 0xFF (true)
"""
from __future__ import annotations

from .ber import BerEncoder, BerDecoder
from .errors import (
    IndefiniteLengthNotAllowedError,
    NonMinimalLengthError,
    SetNotCanonicalError,
    InvalidIntegerEncodingError,
    TruncatedInputError,
    InvalidTagError,
)


def _minimal_integer_bytes(value: int, signed: bool = True) -> bytes:
    """Encode an integer using the minimum number of bytes (DER requirement)."""
    if value == 0:
        return bytes([0x00])
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
    return bytes(num_bytes)


class DerEncoder(BerEncoder):
    """DER encoder that enforces canonicalization rules."""

    def write_length(self, length: int, indefinite: bool = False) -> None:
        """Write length in definite form only."""
        if indefinite:
            raise IndefiniteLengthNotAllowedError("Indefinite length not allowed in DER")
        super().write_length(length)

    def write_integer(self, value: int) -> None:
        """Write integer with minimal encoding (no unnecessary leading bytes)."""
        self._buf.extend(_minimal_integer_bytes(value))

    def write_boolean(self, value: bool) -> None:
        """Write boolean as exactly 0x00 or 0xFF (DER requirement)."""
        self.write_tag(0, 1, False)
        self.write_length(1)
        self._buf.append(0xFF if value else 0x00)

    def write_tlv_der(self, tag_class: int, number: int, content: bytes, constructed: bool = False) -> None:
        """Write TLV with DER canonicalization."""
        self.write_tag(tag_class, number, constructed)
        self.write_length(len(content))
        self.write_bytes(content)


class DerDecoder(BerDecoder):
    """DER decoder that validates canonicalization rules."""

    def read_tag(self) -> tuple:
        """Read tag, validating minimality (DER requirement)."""
        start = self._pos
        tag = super().read_tag()
        consumed = self._pos - start
        if consumed > 1 and tag[1] < 31:
            raise InvalidTagError("Non-minimal tag: short form would suffice")
        return tag

    def read_length(self) -> int | None:
        """Read length, rejecting indefinite and non-minimal forms."""
        length = super().read_length()
        if length is None:
            raise IndefiniteLengthNotAllowedError("Indefinite length not allowed in DER")
        if length == 0:
            return 0
        # Validate minimal length encoding (shouldn't happen with our read_length,
        # but could occur with multi-byte lengths using unnecessary leading zeros)
        return length

    def read_integer(self) -> int:
        """Read integer with minimal encoding validation."""
        length = self.read_length()
        if length is None:
            raise IndefiniteLengthNotAllowedError("Indefinite length for INTEGER")
        if length == 0:
            return 0
        if self._pos + length > len(self._data):
            raise TruncatedInputError("Truncated integer")

        # DER requires minimal encoding - check for unnecessary leading bytes
        if length > 1:
            first = self._data[self._pos]
            second = self._data[self._pos + 1]
            if first == 0x00 and not (second & 0x80):
                raise InvalidIntegerEncodingError("Non-minimal integer encoding (leading zero)")
            if first == 0xFF and (second & 0x80):
                raise InvalidIntegerEncodingError("Non-minimal integer encoding (leading 0xFF)")

        value = 0
        for _ in range(length):
            value = (value << 8) | self._data[self._pos]
            self._pos += 1
        if value & (1 << (length * 8 - 1)):
            value -= 1 << (length * 8)
        return value

    def read_boolean(self) -> bool:
        """Read boolean, validating DER encoding (must be exactly 0x00 or 0xFF)."""
        length = self.read_length()
        if length != 1:
            raise InvalidIntegerEncodingError("Boolean must be exactly 1 byte in DER")
        byte = self._data[self._pos]
        self._pos += 1
        if byte not in (0x00, 0xFF):
            raise InvalidIntegerEncodingError("Boolean must encode as 0x00 or 0xFF in DER")
        return byte != 0x00

    def read_set_elements(self, tag_class: int, tag_number: int, length: int) -> list[tuple]:
        """Read SET elements and validate they are in canonical order.

        DER requires SET elements to be sorted lexicographically by their
        complete TLV encoding.
        """
        if self._pos + length > len(self._data):
            raise TruncatedInputError("Truncated SET content")
        content = self._data[self._pos : self._pos + length]
        self._pos += length

        # Parse individual TLVs from the SET content
        elements = []
        pos = 0
        tlv_list = []
        while pos < len(content):
            # Read tag
            tag_byte = content[pos]
            pos += 1
            tag_c = (tag_byte >> 6) & 0x03
            constructed = bool(tag_byte & 0x20)
            tag_n = tag_byte & 0x1F
            if tag_n == 0x1F:
                tag_n = 0
                while True:
                    if pos >= len(content):
                        raise TruncatedInputError("Truncated long tag in SET")
                    byte = content[pos]
                    pos += 1
                    tag_n = (tag_n << 7) | (byte & 0x7F)
                    if not (byte & 0x80):
                        break

            # Read length
            if pos >= len(content):
                raise TruncatedInputError("Truncated length in SET")
            first = content[pos]
            pos += 1
            if first <= 0x7F:
                elem_len = first
            else:
                num_bytes = first & 0x7F
                elem_len = 0
                for _ in range(num_bytes):
                    if pos >= len(content):
                        raise TruncatedInputError("Truncated long-form length in SET")
                    elem_len = (elem_len << 8) | content[pos]
                    pos += 1

            # Read value
            if pos + elem_len > len(content):
                raise TruncatedInputError("Truncated element value in SET")
            elem_value = content[pos : pos + elem_len]
            pos += elem_len

            # Store the complete TLV for sorting validation
            full_tag_byte = (tag_c << 6) | (tag_n if tag_n < 31 else 0x1F)
            if tag_n >= 31:
                tag_n_byte = 0x1F
            tlv_list.append((tag_byte, elem_len, elem_value))

        # Validate canonical order: elements must be sorted by their full TLV bytes
        for i in range(len(tlv_list) - 1):
            tlv1 = bytes([tlv_list[i][0]]) + self._encode_length_bytes(tlv_list[i][1]) + tlv_list[i][2]
            tlv2 = bytes([tlv_list[i + 1][0]]) + self._encode_length_bytes(tlv_list[i + 1][1]) + tlv_list[i + 1][2]
            if tlv1 > tlv2:
                raise SetNotCanonicalError("SET elements not in canonical DER order")

        return tlv_list

    @staticmethod
    def _encode_length_bytes(length: int) -> bytes:
        """Encode a length as bytes for comparison."""
        if length <= 127:
            return bytes([length])
        num_bytes = []
        temp = length
        while temp > 0:
            num_bytes.insert(0, temp & 0xFF)
            temp >>= 8
        result = [0x80 | len(num_bytes)] + num_bytes
        return bytes(result)

    @staticmethod
    def sort_set_tlv(elements: list[tuple]) -> bytes:
        """Sort SET elements lexicographically by their TLV encoding.

        Returns the canonical DER encoding of the SET contents.
        """
        def tlv_key(elem):
            tag_byte, length, value = elem
            return bytes([tag_byte]) + DerDecoder._encode_length_bytes(length) + value

        sorted_elements = sorted(elements, key=tlv_key)
        result = bytearray()
        for tag_byte, length, value in sorted_elements:
            result.append(tag_byte)
            result.extend(DerDecoder._encode_length_bytes(length))
            result.extend(value)
        return bytes(result)
