"""ASN.1 runtime types."""
from __future__ import annotations

from .errors import AsnError


class BitString:
    def __init__(self, data: bytes, unused_bits: int = 0):
        if unused_bits < 0 or unused_bits > 7:
            raise AsnError("unused_bits must be 0-7")
        if unused_bits > 0 and len(data) > 0:
            if data[-1] & ((1 << unused_bits) - 1):
                raise AsnError("Unused bits must be zero")
        self._data = data
        self._unused_bits = unused_bits

    @property
    def data(self) -> bytes:
        return self._data

    @property
    def unused_bits(self) -> int:
        return self._unused_bits

    def __eq__(self, other):
        if isinstance(other, BitString):
            return self._data == other._data and self._unused_bits == other._unused_bits
        return False

    def __repr__(self):
        return f"BitString({self._data!r}, unused_bits={self._unused_bits})"


class ObjectIdentifier:
    def __init__(self, components: list[int | str]):
        self._components = components

    @property
    def components(self) -> list[int | str]:
        return self._components

    def encode(self) -> bytes:
        if len(self._components) < 2:
            raise AsnError("OID must have at least 2 components")
        result = bytearray()
        first = self._components[0]
        second = self._components[1]
        if isinstance(first, int) and isinstance(second, int):
            result.append(first * 40 + second)
        else:
            raise AsnError("First two OID components must be integers")
        for component in self._components[2:]:
            if isinstance(component, int):
                if component < 0:
                    raise AsnError("OID component cannot be negative")
                num_bytes = []
                while component > 0:
                    num_bytes.insert(0, component & 0x7F)
                    component >>= 7
                if not num_bytes:
                    num_bytes = [0]
                for i in range(len(num_bytes) - 1):
                    num_bytes[i] |= 0x80
                result.extend(num_bytes)
            else:
                raise AsnError("OID components after first two must be integers")
        return bytes(result)

    @classmethod
    def decode(cls, data: bytes) -> tuple["ObjectIdentifier", int]:
        if len(data) < 1:
            raise AsnError("Not enough data to decode OID")
        first = data[0]
        components: list[int | str] = [first // 40, first % 40]
        pos = 1
        while pos < len(data):
            component = 0
            while True:
                if pos >= len(data):
                    raise AsnError("Truncated OID")
                byte = data[pos]
                pos += 1
                component = (component << 7) | (byte & 0x7F)
                if not (byte & 0x80):
                    break
            components.append(component)
        return cls(components), pos

    def __eq__(self, other):
        if isinstance(other, ObjectIdentifier):
            return self._components == other._components
        return False

    def __repr__(self):
        return f"ObjectIdentifier({self._components!r})"


class AsnAny:
    def __init__(self, tag_class: int, number: int, content: bytes):
        self.tag_class = tag_class
        self.number = number
        self.content = content
