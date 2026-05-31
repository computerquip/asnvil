"""ASN.1 Compiler Python Runtime - Pure stdlib implementation."""
from __future__ import annotations

from .errors import AsnError, UnexpectedTagError, InvalidLengthError, TruncatedInputError
from .ber import BerEncoder, BerDecoder
from .der import DerEncoder, DerDecoder
from .types import BitString, ObjectIdentifier

__all__ = [
    "AsnError",
    "UnexpectedTagError",
    "InvalidLengthError",
    "TruncatedInputError",
    "BerEncoder",
    "BerDecoder",
    "DerEncoder",
    "DerDecoder",
    "BitString",
    "ObjectIdentifier",
    "AsnType",
    "Tag",
    "TagClass",
]

from dataclasses import dataclass


class TagClass:
    UNIVERSAL = 0
    APPLICATION = 1
    CONTEXT = 2
    PRIVATE = 3


@dataclass
class Tag:
    tag_class: int
    number: int
    constructed: bool = False

    def encode(self) -> bytes:
        if self.number < 31:
            tag_byte = (self.tag_class << 6) | (self.number & 0x1F)
            if self.constructed:
                tag_byte |= 0x20
            return bytes([tag_byte])
        else:
            tag_byte = (self.tag_class << 6) | 0x1F
            if self.constructed:
                tag_byte |= 0x20
            num_bytes = []
            num = self.number
            while num > 0:
                num_bytes.insert(0, num & 0x7F)
                num >>= 7
            for i, byte in enumerate(num_bytes):
                if i < len(num_bytes) - 1:
                    byte |= 0x80
                num_bytes[i] = byte
            return bytes([tag_byte]) + bytes(num_bytes)

    @classmethod
    def decode(cls, data: bytes) -> tuple[Tag, int]:
        if len(data) < 1:
            raise TruncatedInputError("Not enough data to decode tag")
        tag_byte = data[0]
        tag_class = (tag_byte >> 6) & 0x03
        constructed = bool(tag_byte & 0x20)
        number = tag_byte & 0x1F
        pos = 1

        if number == 0x1F:
            number = 0
            while True:
                if pos >= len(data):
                    raise TruncatedInputError("Truncated long tag")
                byte = data[pos]
                pos += 1
                number = (number << 7) | (byte & 0x7F)
                if not (byte & 0x80):
                    break

        return cls(tag_class=tag_class, number=number, constructed=constructed), pos


class AsnType:
    TAG: Tag = Tag(tag_class=TagClass.UNIVERSAL, number=0)
