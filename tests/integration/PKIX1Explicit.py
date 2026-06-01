"""Generated from PKIX1Explicit.asn1 by asnvil."""
from __future__ import annotations
from asnvil_runtime import AsnType, Tag, TagClass, BerEncoder, BerDecoder, DerEncoder, DerDecoder, BitString, ObjectIdentifier, AsnError, InvalidLengthError
from dataclasses import dataclass, field
from typing import Optional
from enum import Enum, IntEnum
from datetime import datetime


Version = int

CertificateSerialNumber = int

@dataclass
class AlgorithmIdentifier(AsnType):


    algorithm: ObjectIdentifier

    parameters: Optional[bytes] = None




    def encode_ber(self) -> bytes:
        content = bytearray()




        _oe = BerEncoder()
        _ob = self.algorithm.encode()
        _oe.write_tag(TagClass.UNIVERSAL, 6, False)
        _oe.write_length(len(_ob))
        _oe.write_bytes(_ob)
        content.extend(_oe.finish())







        if self.parameters is not None:


            _be = BerEncoder()
            _be.write_tlv(TagClass.UNIVERSAL, 4, self.parameters)
            content.extend(_be.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _oe = BerEncoder()
        _ob = self.algorithm.encode()
        _oe.write_tag(TagClass.UNIVERSAL, 6, False)
        _oe.write_length(len(_ob))
        _oe.write_bytes(_ob)
        content.extend(_oe.finish())







        if self.parameters is not None:


            _be = BerEncoder()
            _be.write_tlv(TagClass.UNIVERSAL, 4, self.parameters)
            content.extend(_be.finish())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        _oe = DerEncoder()
        _ob = self.algorithm.encode()
        _oe.write_tag(TagClass.UNIVERSAL, 6, False)
        _oe.write_length(len(_ob))
        _oe.write_bytes(_ob)
        content.extend(_oe.finish())







        if self.parameters is not None:


            _be = DerEncoder()
            _be.write_tlv_der(TagClass.UNIVERSAL, 4, self.parameters)
            content.extend(_be.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "AlgorithmIdentifier":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _algorithm, _ = ObjectIdentifier.decode(_fd)







        _parameters = None

        if decoder._pos < _end:
            _parameters_save = decoder._pos
            _ft = decoder.read_tag()
            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == 4 and not _ft[2]:
                _fl = decoder.read_length()
                _fd = decoder.read_bytes(_fl)

                _parameters = _fd

            else:
                decoder._pos = _parameters_save




        return cls(

            algorithm=_algorithm,

            parameters=_parameters

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "AlgorithmIdentifier":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is not None:
            raise InvalidLengthError("Expected indefinite length")
        _content = decoder.read_constructed_indefinite()
        decoder2 = BerDecoder(_content)




        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _algorithm, _ = ObjectIdentifier.decode(_fd)







        _parameters = None

        if not decoder2.at_end():
            _parameters_save = decoder2._pos
            _ft = decoder2.read_tag()
            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == 4 and not _ft[2]:
                _fl = decoder2.read_length()
                _fd = decoder2.read_bytes(_fl)

                _parameters = _fd

            else:
                decoder2._pos = _parameters_save




        return cls(

            algorithm=_algorithm,

            parameters=_parameters

        )


@dataclass
class Validity(AsnType):


    notBefore: datetime

    notAfter: datetime




    def encode_ber(self) -> bytes:
        content = bytearray()




        _te = BerEncoder()
        _tb = self.notBefore.strftime('%Y%m%d%H%M%SZ').encode('ascii')
        _te.write_tag(TagClass.UNIVERSAL, 23, False)
        _te.write_length(len(_tb))
        _te.write_bytes(_tb)
        content.extend(_te.finish())







        _te = BerEncoder()
        _tb = self.notAfter.strftime('%Y%m%d%H%M%SZ').encode('ascii')
        _te.write_tag(TagClass.UNIVERSAL, 23, False)
        _te.write_length(len(_tb))
        _te.write_bytes(_tb)
        content.extend(_te.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _te = BerEncoder()
        _tb = self.notBefore.strftime('%Y%m%d%H%M%SZ').encode('ascii')
        _te.write_tag(TagClass.UNIVERSAL, 23, False)
        _te.write_length(len(_tb))
        _te.write_bytes(_tb)
        content.extend(_te.finish())







        _te = BerEncoder()
        _tb = self.notAfter.strftime('%Y%m%d%H%M%SZ').encode('ascii')
        _te.write_tag(TagClass.UNIVERSAL, 23, False)
        _te.write_length(len(_tb))
        _te.write_bytes(_tb)
        content.extend(_te.finish())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        _te = DerEncoder()
        _tb = self.notBefore.strftime('%Y%m%d%H%M%SZ').encode('ascii')
        _te.write_tag(TagClass.UNIVERSAL, 23, False)
        _te.write_length(len(_tb))
        _te.write_bytes(_tb)
        content.extend(_te.finish())







        _te = DerEncoder()
        _tb = self.notAfter.strftime('%Y%m%d%H%M%SZ').encode('ascii')
        _te.write_tag(TagClass.UNIVERSAL, 23, False)
        _te.write_length(len(_tb))
        _te.write_bytes(_tb)
        content.extend(_te.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "Validity":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        from datetime import datetime
        _notBefore = datetime.strptime(_fd.decode('ascii'), '%Y%m%d%H%M%SZ')







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        from datetime import datetime
        _notAfter = datetime.strptime(_fd.decode('ascii'), '%Y%m%d%H%M%SZ')




        return cls(

            notBefore=_notBefore,

            notAfter=_notAfter

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "Validity":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is not None:
            raise InvalidLengthError("Expected indefinite length")
        _content = decoder.read_constructed_indefinite()
        decoder2 = BerDecoder(_content)




        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        from datetime import datetime
        _notBefore = datetime.strptime(_fd.decode('ascii'), '%Y%m%d%H%M%SZ')







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        from datetime import datetime
        _notAfter = datetime.strptime(_fd.decode('ascii'), '%Y%m%d%H%M%SZ')




        return cls(

            notBefore=_notBefore,

            notAfter=_notAfter

        )


@dataclass
class SubjectPublicKeyInfo(AsnType):


    algorithm: AlgorithmIdentifier

    subjectPublicKey: BitString




    def encode_ber(self) -> bytes:
        content = bytearray()




        content.extend(self.algorithm.encode_ber())







        _be = BerEncoder()
        _be.write_tag(TagClass.UNIVERSAL, 3, False)
        _be.write_length(len(self.subjectPublicKey.data) + 1)
        _be.write_bytes(bytes([self.subjectPublicKey.unused_bits]))
        _be.write_bytes(self.subjectPublicKey.data)
        content.extend(_be.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        content.extend(self.algorithm.encode_ber())







        _be = BerEncoder()
        _be.write_tag(TagClass.UNIVERSAL, 3, False)
        _be.write_length(len(self.subjectPublicKey.data) + 1)
        _be.write_bytes(bytes([self.subjectPublicKey.unused_bits]))
        _be.write_bytes(self.subjectPublicKey.data)
        content.extend(_be.finish())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        content.extend(self.algorithm.encode_der())







        _be = DerEncoder()
        _be.write_tag(TagClass.UNIVERSAL, 3, False)
        _be.write_length(len(self.subjectPublicKey.data) + 1)
        _be.write_bytes(bytes([self.subjectPublicKey.unused_bits]))
        _be.write_bytes(self.subjectPublicKey.data)
        content.extend(_be.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "SubjectPublicKeyInfo":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _algorithm_re = BerEncoder()
        _algorithm_re.write_tag(_ft[0], _ft[1], _ft[2])
        _algorithm_re.write_length(_fl)
        _algorithm_re.write_bytes(_fd)
        _algorithm = AlgorithmIdentifier.decode_der(_algorithm_re.finish())







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _subjectPublicKey = BitString(_fd[1:], _fd[0])




        return cls(

            algorithm=_algorithm,

            subjectPublicKey=_subjectPublicKey

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "SubjectPublicKeyInfo":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is not None:
            raise InvalidLengthError("Expected indefinite length")
        _content = decoder.read_constructed_indefinite()
        decoder2 = BerDecoder(_content)




        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _algorithm_re = BerEncoder()
        _algorithm_re.write_tag(_ft[0], _ft[1], _ft[2])
        _algorithm_re.write_length(_fl)
        _algorithm_re.write_bytes(_fd)
        _algorithm = AlgorithmIdentifier.decode_ber(_algorithm_re.finish())







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _subjectPublicKey = BitString(_fd[1:], _fd[0])




        return cls(

            algorithm=_algorithm,

            subjectPublicKey=_subjectPublicKey

        )


AttributeValue = bytes

@dataclass
class AttributeTypeAndValue(AsnType):


    type: ObjectIdentifier

    value: bytes




    def encode_ber(self) -> bytes:
        content = bytearray()




        _oe = BerEncoder()
        _ob = self.type.encode()
        _oe.write_tag(TagClass.UNIVERSAL, 6, False)
        _oe.write_length(len(_ob))
        _oe.write_bytes(_ob)
        content.extend(_oe.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.value)
        content.extend(_be.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _oe = BerEncoder()
        _ob = self.type.encode()
        _oe.write_tag(TagClass.UNIVERSAL, 6, False)
        _oe.write_length(len(_ob))
        _oe.write_bytes(_ob)
        content.extend(_oe.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.value)
        content.extend(_be.finish())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        _oe = DerEncoder()
        _ob = self.type.encode()
        _oe.write_tag(TagClass.UNIVERSAL, 6, False)
        _oe.write_length(len(_ob))
        _oe.write_bytes(_ob)
        content.extend(_oe.finish())







        _be = DerEncoder()
        _be.write_tlv_der(TagClass.UNIVERSAL, 4, self.value)
        content.extend(_be.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "AttributeTypeAndValue":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _type, _ = ObjectIdentifier.decode(_fd)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _value = decoder.read_bytes(_fl)




        return cls(

            type=_type,

            value=_value

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "AttributeTypeAndValue":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is not None:
            raise InvalidLengthError("Expected indefinite length")
        _content = decoder.read_constructed_indefinite()
        decoder2 = BerDecoder(_content)




        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _type, _ = ObjectIdentifier.decode(_fd)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _value = decoder2.read_bytes(_fl)




        return cls(

            type=_type,

            value=_value

        )


class RelativeDistinguishedName(list, AsnType):

    def __init__(self, items: list[AttributeTypeAndValue] | None = None):
        super().__init__(items or [])

    def encode_ber(self) -> bytes:

        _outer = BerEncoder()

        _content = bytearray()
        for _item in self:


            _content.extend(_item.encode_der())



        _outer.write_tag(TagClass.UNIVERSAL, 17, True)

        _outer.write_length(len(_content))
        _outer.write_bytes(_content)
        return _outer.finish()

    @classmethod
    def decode_ber(cls, data: bytes) -> "RelativeDistinguishedName":
        return cls.decode_der(data)

    def encode_der(self) -> bytes:
        return self.encode_ber()

    @classmethod
    def decode_der(cls, data: bytes) -> "RelativeDistinguishedName":
        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length
        items: list[AttributeTypeAndValue] = []
        while decoder._pos < _end:


            _item_start = decoder._pos
            _lt = decoder.read_tag()
            _ll = decoder.read_length()
            _ = decoder.read_bytes(_ll)
            items.append(AttributeTypeAndValue.decode_der(decoder._data[_item_start:decoder._pos]))


        return cls(items)

    def encode_ber_indefinite(self) -> bytes:
        _outer = BerEncoder()
        _content = bytearray()
        for _item in self:


            _content.extend(_item.encode_der())



        _outer.write_tag(TagClass.UNIVERSAL, 17, True)

        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(_content)
        _outer.write_eoc()
        return _outer.finish()

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "RelativeDistinguishedName":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is None:
            _content = decoder.read_constructed_indefinite()
        else:
            _content = decoder.read_bytes(_length)
        items: list[AttributeTypeAndValue] = []
        _ld = BerDecoder(_content)
        while not _ld.at_end():


            _item_start = _ld._pos
            _lt = _ld.read_tag()
            _ll = _ld.read_length()
            _ = _ld.read_bytes(_ll)
            items.append(AttributeTypeAndValue.decode_der(_ld._data[_item_start:_ld._pos]))


        return cls(items)

class RDNSequence(list, AsnType):

    def __init__(self, items: list[RelativeDistinguishedName] | None = None):
        super().__init__(items or [])

    def encode_ber(self) -> bytes:

        _outer = BerEncoder()

        _content = bytearray()
        for _item in self:


            _content.extend(_item.encode_ber())



        _outer.write_tag(TagClass.UNIVERSAL, 16, True)

        _outer.write_length(len(_content))
        _outer.write_bytes(_content)
        return _outer.finish()

    @classmethod
    def decode_ber(cls, data: bytes) -> "RDNSequence":
        return cls.decode_der(data)

    def encode_der(self) -> bytes:
        return self.encode_ber()

    @classmethod
    def decode_der(cls, data: bytes) -> "RDNSequence":
        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length
        items: list[RelativeDistinguishedName] = []
        while decoder._pos < _end:


            _item_start = decoder._pos
            _lt = decoder.read_tag()
            _ll = decoder.read_length()
            _ = decoder.read_bytes(_ll)
            items.append(RelativeDistinguishedName.decode_der(decoder._data[_item_start:decoder._pos]))


        return cls(items)

    def encode_ber_indefinite(self) -> bytes:
        _outer = BerEncoder()
        _content = bytearray()
        for _item in self:


            _content.extend(_item.encode_ber())



        _outer.write_tag(TagClass.UNIVERSAL, 16, True)

        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(_content)
        _outer.write_eoc()
        return _outer.finish()

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "RDNSequence":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is None:
            _content = decoder.read_constructed_indefinite()
        else:
            _content = decoder.read_bytes(_length)
        items: list[RelativeDistinguishedName] = []
        _ld = BerDecoder(_content)
        while not _ld.at_end():


            _item_start = _ld._pos
            _lt = _ld.read_tag()
            _ll = _ld.read_length()
            _ = _ld.read_bytes(_ll)
            items.append(RelativeDistinguishedName.decode_der(_ld._data[_item_start:_ld._pos]))


        return cls(items)

@dataclass
class Name(AsnType):


    rdnSequence: list[RelativeDistinguishedName]




    def encode_ber(self) -> bytes:
        content = bytearray()




        _le = BerEncoder()
        _lc = bytearray()
        for _li in self.rdnSequence:


            _lc.extend(_li.encode_der())


        _le.write_tag(TagClass.UNIVERSAL, 16, True)
        _le.write_length(len(_lc))
        _le.write_bytes(_lc)
        content.extend(_le.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _le = BerEncoder()
        _lc = bytearray()
        for _li in self.rdnSequence:


            _lc.extend(_li.encode_der())


        _le.write_tag(TagClass.UNIVERSAL, 16, True)
        _le.write_length(len(_lc))
        _le.write_bytes(_lc)
        content.extend(_le.finish())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        _le = DerEncoder()
        _lc = bytearray()
        for _li in self.rdnSequence:


             _lc.extend(_li.encode_der())


        _le.write_tag(TagClass.UNIVERSAL, 16, True)
        _le.write_length(len(_lc))
        _le.write_bytes(_lc)
        content.extend(_le.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "Name":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _ld = DerDecoder(_fd)
        _rdnSequence = []
        while not _ld.at_end():


            _rdnSequence_start = _ld._pos
            _lt = _ld.read_tag()
            _ll = _ld.read_length()
            _lv = _ld.read_bytes(_ll)
            _rdnSequence.append(RelativeDistinguishedName.decode_der(_ld._data[_rdnSequence_start:_ld._pos]))






        return cls(

            rdnSequence=_rdnSequence

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "Name":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is not None:
            raise InvalidLengthError("Expected indefinite length")
        _content = decoder.read_constructed_indefinite()
        decoder2 = BerDecoder(_content)




        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _ld = BerDecoder(_fd)
        _rdnSequence = []
        while not _ld.at_end():


            _rdnSequence_start = _ld._pos
            _lt = _ld.read_tag()
            _ll = _ld.read_length()
            _lv = _ld.read_bytes(_ll)
            _rdnSequence.append(RelativeDistinguishedName.decode_der(_ld._data[_rdnSequence_start:_ld._pos]))






        return cls(

            rdnSequence=_rdnSequence

        )


@dataclass
class Extension(AsnType):


    extnID: ObjectIdentifier

    extnValue: bytes

    critical: Optional[bool] = None




    def encode_ber(self) -> bytes:
        content = bytearray()




        _oe = BerEncoder()
        _ob = self.extnID.encode()
        _oe.write_tag(TagClass.UNIVERSAL, 6, False)
        _oe.write_length(len(_ob))
        _oe.write_bytes(_ob)
        content.extend(_oe.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.extnValue)
        content.extend(_be.finish())







        if self.critical is not None:


            _be = BerEncoder()
            _be.write_tag(TagClass.UNIVERSAL, 1, False)
            _be.write_length(1)
            _be.write_bytes(b'\xff' if self.critical else b'\x00')
            content.extend(_be.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _oe = BerEncoder()
        _ob = self.extnID.encode()
        _oe.write_tag(TagClass.UNIVERSAL, 6, False)
        _oe.write_length(len(_ob))
        _oe.write_bytes(_ob)
        content.extend(_oe.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.extnValue)
        content.extend(_be.finish())







        if self.critical is not None:


            _be = BerEncoder()
            _be.write_tag(TagClass.UNIVERSAL, 1, False)
            _be.write_length(1)
            _be.write_bytes(b'\xff' if self.critical else b'\x00')
            content.extend(_be.finish())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        _oe = DerEncoder()
        _ob = self.extnID.encode()
        _oe.write_tag(TagClass.UNIVERSAL, 6, False)
        _oe.write_length(len(_ob))
        _oe.write_bytes(_ob)
        content.extend(_oe.finish())







        _be = DerEncoder()
        _be.write_tlv_der(TagClass.UNIVERSAL, 4, self.extnValue)
        content.extend(_be.finish())







        if self.critical is not None:


            _be = DerEncoder()
            _be.write_tag(TagClass.UNIVERSAL, 1, False)
            _be.write_length(1)
            _be.write_bytes(b'\xff' if self.critical else b'\x00')
            content.extend(_be.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "Extension":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _extnID, _ = ObjectIdentifier.decode(_fd)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _extnValue = decoder.read_bytes(_fl)







        _critical = None

        if decoder._pos < _end:
            _critical_save = decoder._pos
            _ft = decoder.read_tag()
            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == 1 and not _ft[2]:
                _fl = decoder.read_length()
                _fd = decoder.read_bytes(_fl)

                _critical = _fd[0] != 0

            else:
                decoder._pos = _critical_save




        return cls(

            extnID=_extnID,

            extnValue=_extnValue,

            critical=_critical

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "Extension":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is not None:
            raise InvalidLengthError("Expected indefinite length")
        _content = decoder.read_constructed_indefinite()
        decoder2 = BerDecoder(_content)




        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _extnID, _ = ObjectIdentifier.decode(_fd)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _extnValue = decoder2.read_bytes(_fl)







        _critical = None

        if not decoder2.at_end():
            _critical_save = decoder2._pos
            _ft = decoder2.read_tag()
            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == 1 and not _ft[2]:
                _fl = decoder2.read_length()
                _fd = decoder2.read_bytes(_fl)

                _critical = _fd[0] != 0

            else:
                decoder2._pos = _critical_save




        return cls(

            extnID=_extnID,

            extnValue=_extnValue,

            critical=_critical

        )


class Extensions(list, AsnType):

    def __init__(self, items: list[Extension] | None = None):
        super().__init__(items or [])

    def encode_ber(self) -> bytes:

        _outer = BerEncoder()

        _content = bytearray()
        for _item in self:


            _content.extend(_item.encode_der())



        _outer.write_tag(TagClass.UNIVERSAL, 16, True)

        _outer.write_length(len(_content))
        _outer.write_bytes(_content)
        return _outer.finish()

    @classmethod
    def decode_ber(cls, data: bytes) -> "Extensions":
        return cls.decode_der(data)

    def encode_der(self) -> bytes:
        return self.encode_ber()

    @classmethod
    def decode_der(cls, data: bytes) -> "Extensions":
        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length
        items: list[Extension] = []
        while decoder._pos < _end:


            _item_start = decoder._pos
            _lt = decoder.read_tag()
            _ll = decoder.read_length()
            _ = decoder.read_bytes(_ll)
            items.append(Extension.decode_der(decoder._data[_item_start:decoder._pos]))


        return cls(items)

    def encode_ber_indefinite(self) -> bytes:
        _outer = BerEncoder()
        _content = bytearray()
        for _item in self:


            _content.extend(_item.encode_der())



        _outer.write_tag(TagClass.UNIVERSAL, 16, True)

        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(_content)
        _outer.write_eoc()
        return _outer.finish()

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "Extensions":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is None:
            _content = decoder.read_constructed_indefinite()
        else:
            _content = decoder.read_bytes(_length)
        items: list[Extension] = []
        _ld = BerDecoder(_content)
        while not _ld.at_end():


            _item_start = _ld._pos
            _lt = _ld.read_tag()
            _ll = _ld.read_length()
            _ = _ld.read_bytes(_ll)
            items.append(Extension.decode_der(_ld._data[_item_start:_ld._pos]))


        return cls(items)

@dataclass
class TBSCertificate(AsnType):


    serialNumber: int

    signature: AlgorithmIdentifier

    issuer: Name

    validity: Validity

    subject: Name

    subjectPublicKeyInfo: SubjectPublicKeyInfo

    version: Optional[int] = None




    def encode_ber(self) -> bytes:
        content = bytearray()




        _iv = BerEncoder()
        _iv.write_integer(self.serialNumber)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        content.extend(self.signature.encode_ber())







        content.extend(self.issuer.encode_ber())







        content.extend(self.validity.encode_ber())







        content.extend(self.subject.encode_ber())







        content.extend(self.subjectPublicKeyInfo.encode_ber())







        if self.version is not None:


            _iv = BerEncoder()
            _iv.write_integer(self.version)
            _ib = _iv.finish()
            _ie = BerEncoder()
            _ie.write_tag(TagClass.UNIVERSAL, 2, False)
            _ie.write_length(len(_ib))
            _ie.write_bytes(_ib)
            content.extend(_ie.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _iv = BerEncoder()
        _iv.write_integer(self.serialNumber)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        content.extend(self.signature.encode_ber())







        content.extend(self.issuer.encode_ber())







        content.extend(self.validity.encode_ber())







        content.extend(self.subject.encode_ber())







        content.extend(self.subjectPublicKeyInfo.encode_ber())







        if self.version is not None:


            _iv = BerEncoder()
            _iv.write_integer(self.version)
            _ib = _iv.finish()
            _ie = BerEncoder()
            _ie.write_tag(TagClass.UNIVERSAL, 2, False)
            _ie.write_length(len(_ib))
            _ie.write_bytes(_ib)
            content.extend(_ie.finish())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        _iv = DerEncoder()
        _iv.write_integer(self.serialNumber)
        _ib = _iv.finish()
        _ie = DerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        content.extend(self.signature.encode_der())







        content.extend(self.issuer.encode_der())







        content.extend(self.validity.encode_der())







        content.extend(self.subject.encode_der())







        content.extend(self.subjectPublicKeyInfo.encode_der())







        if self.version is not None:


            _iv = DerEncoder()
            _iv.write_integer(self.version)
            _ib = _iv.finish()
            _ie = DerEncoder()
            _ie.write_tag(TagClass.UNIVERSAL, 2, False)
            _ie.write_length(len(_ib))
            _ie.write_bytes(_ib)
            content.extend(_ie.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "TBSCertificate":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _serialNumber = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _signature_re = BerEncoder()
        _signature_re.write_tag(_ft[0], _ft[1], _ft[2])
        _signature_re.write_length(_fl)
        _signature_re.write_bytes(_fd)
        _signature = AlgorithmIdentifier.decode_der(_signature_re.finish())







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _issuer_re = BerEncoder()
        _issuer_re.write_tag(_ft[0], _ft[1], _ft[2])
        _issuer_re.write_length(_fl)
        _issuer_re.write_bytes(_fd)
        _issuer = Name.decode_der(_issuer_re.finish())







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _validity_re = BerEncoder()
        _validity_re.write_tag(_ft[0], _ft[1], _ft[2])
        _validity_re.write_length(_fl)
        _validity_re.write_bytes(_fd)
        _validity = Validity.decode_der(_validity_re.finish())







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _subject_re = BerEncoder()
        _subject_re.write_tag(_ft[0], _ft[1], _ft[2])
        _subject_re.write_length(_fl)
        _subject_re.write_bytes(_fd)
        _subject = Name.decode_der(_subject_re.finish())







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _subjectPublicKeyInfo_re = BerEncoder()
        _subjectPublicKeyInfo_re.write_tag(_ft[0], _ft[1], _ft[2])
        _subjectPublicKeyInfo_re.write_length(_fl)
        _subjectPublicKeyInfo_re.write_bytes(_fd)
        _subjectPublicKeyInfo = SubjectPublicKeyInfo.decode_der(_subjectPublicKeyInfo_re.finish())







        _version = None

        if decoder._pos < _end:
            _version_save = decoder._pos
            _ft = decoder.read_tag()
            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == 2 and not _ft[2]:
                _fl = decoder.read_length()
                _fd = decoder.read_bytes(_fl)

                _version = int.from_bytes(_fd, byteorder='big', signed=True)

            else:
                decoder._pos = _version_save




        return cls(

            serialNumber=_serialNumber,

            signature=_signature,

            issuer=_issuer,

            validity=_validity,

            subject=_subject,

            subjectPublicKeyInfo=_subjectPublicKeyInfo,

            version=_version

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "TBSCertificate":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is not None:
            raise InvalidLengthError("Expected indefinite length")
        _content = decoder.read_constructed_indefinite()
        decoder2 = BerDecoder(_content)




        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _serialNumber = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _signature_re = BerEncoder()
        _signature_re.write_tag(_ft[0], _ft[1], _ft[2])
        _signature_re.write_length(_fl)
        _signature_re.write_bytes(_fd)
        _signature = AlgorithmIdentifier.decode_ber(_signature_re.finish())







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _issuer_re = BerEncoder()
        _issuer_re.write_tag(_ft[0], _ft[1], _ft[2])
        _issuer_re.write_length(_fl)
        _issuer_re.write_bytes(_fd)
        _issuer = Name.decode_ber(_issuer_re.finish())







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _validity_re = BerEncoder()
        _validity_re.write_tag(_ft[0], _ft[1], _ft[2])
        _validity_re.write_length(_fl)
        _validity_re.write_bytes(_fd)
        _validity = Validity.decode_ber(_validity_re.finish())







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _subject_re = BerEncoder()
        _subject_re.write_tag(_ft[0], _ft[1], _ft[2])
        _subject_re.write_length(_fl)
        _subject_re.write_bytes(_fd)
        _subject = Name.decode_ber(_subject_re.finish())







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _subjectPublicKeyInfo_re = BerEncoder()
        _subjectPublicKeyInfo_re.write_tag(_ft[0], _ft[1], _ft[2])
        _subjectPublicKeyInfo_re.write_length(_fl)
        _subjectPublicKeyInfo_re.write_bytes(_fd)
        _subjectPublicKeyInfo = SubjectPublicKeyInfo.decode_ber(_subjectPublicKeyInfo_re.finish())







        _version = None

        if not decoder2.at_end():
            _version_save = decoder2._pos
            _ft = decoder2.read_tag()
            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == 2 and not _ft[2]:
                _fl = decoder2.read_length()
                _fd = decoder2.read_bytes(_fl)

                _version = int.from_bytes(_fd, byteorder='big', signed=True)

            else:
                decoder2._pos = _version_save




        return cls(

            serialNumber=_serialNumber,

            signature=_signature,

            issuer=_issuer,

            validity=_validity,

            subject=_subject,

            subjectPublicKeyInfo=_subjectPublicKeyInfo,

            version=_version

        )


@dataclass
class Certificate(AsnType):


    tbsCertificate: TBSCertificate

    signatureAlgorithm: AlgorithmIdentifier

    signatureValue: BitString




    def encode_ber(self) -> bytes:
        content = bytearray()




        content.extend(self.tbsCertificate.encode_ber())







        content.extend(self.signatureAlgorithm.encode_ber())







        _be = BerEncoder()
        _be.write_tag(TagClass.UNIVERSAL, 3, False)
        _be.write_length(len(self.signatureValue.data) + 1)
        _be.write_bytes(bytes([self.signatureValue.unused_bits]))
        _be.write_bytes(self.signatureValue.data)
        content.extend(_be.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        content.extend(self.tbsCertificate.encode_ber())







        content.extend(self.signatureAlgorithm.encode_ber())







        _be = BerEncoder()
        _be.write_tag(TagClass.UNIVERSAL, 3, False)
        _be.write_length(len(self.signatureValue.data) + 1)
        _be.write_bytes(bytes([self.signatureValue.unused_bits]))
        _be.write_bytes(self.signatureValue.data)
        content.extend(_be.finish())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        content.extend(self.tbsCertificate.encode_der())







        content.extend(self.signatureAlgorithm.encode_der())







        _be = DerEncoder()
        _be.write_tag(TagClass.UNIVERSAL, 3, False)
        _be.write_length(len(self.signatureValue.data) + 1)
        _be.write_bytes(bytes([self.signatureValue.unused_bits]))
        _be.write_bytes(self.signatureValue.data)
        content.extend(_be.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "Certificate":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _tbsCertificate_re = BerEncoder()
        _tbsCertificate_re.write_tag(_ft[0], _ft[1], _ft[2])
        _tbsCertificate_re.write_length(_fl)
        _tbsCertificate_re.write_bytes(_fd)
        _tbsCertificate = TBSCertificate.decode_der(_tbsCertificate_re.finish())







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _signatureAlgorithm_re = BerEncoder()
        _signatureAlgorithm_re.write_tag(_ft[0], _ft[1], _ft[2])
        _signatureAlgorithm_re.write_length(_fl)
        _signatureAlgorithm_re.write_bytes(_fd)
        _signatureAlgorithm = AlgorithmIdentifier.decode_der(_signatureAlgorithm_re.finish())







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _signatureValue = BitString(_fd[1:], _fd[0])




        return cls(

            tbsCertificate=_tbsCertificate,

            signatureAlgorithm=_signatureAlgorithm,

            signatureValue=_signatureValue

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "Certificate":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is not None:
            raise InvalidLengthError("Expected indefinite length")
        _content = decoder.read_constructed_indefinite()
        decoder2 = BerDecoder(_content)




        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _tbsCertificate_re = BerEncoder()
        _tbsCertificate_re.write_tag(_ft[0], _ft[1], _ft[2])
        _tbsCertificate_re.write_length(_fl)
        _tbsCertificate_re.write_bytes(_fd)
        _tbsCertificate = TBSCertificate.decode_ber(_tbsCertificate_re.finish())







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _signatureAlgorithm_re = BerEncoder()
        _signatureAlgorithm_re.write_tag(_ft[0], _ft[1], _ft[2])
        _signatureAlgorithm_re.write_length(_fl)
        _signatureAlgorithm_re.write_bytes(_fd)
        _signatureAlgorithm = AlgorithmIdentifier.decode_ber(_signatureAlgorithm_re.finish())







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _signatureValue = BitString(_fd[1:], _fd[0])




        return cls(

            tbsCertificate=_tbsCertificate,

            signatureAlgorithm=_signatureAlgorithm,

            signatureValue=_signatureValue

        )


@dataclass
class CertificateList(AsnType):


    tbsCertList: TBSCertList

    signatureAlgorithm: AlgorithmIdentifier

    signature: BitString




    def encode_ber(self) -> bytes:
        content = bytearray()




        content.extend(self.tbsCertList.encode_ber())







        content.extend(self.signatureAlgorithm.encode_ber())







        _be = BerEncoder()
        _be.write_tag(TagClass.UNIVERSAL, 3, False)
        _be.write_length(len(self.signature.data) + 1)
        _be.write_bytes(bytes([self.signature.unused_bits]))
        _be.write_bytes(self.signature.data)
        content.extend(_be.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        content.extend(self.tbsCertList.encode_ber())







        content.extend(self.signatureAlgorithm.encode_ber())







        _be = BerEncoder()
        _be.write_tag(TagClass.UNIVERSAL, 3, False)
        _be.write_length(len(self.signature.data) + 1)
        _be.write_bytes(bytes([self.signature.unused_bits]))
        _be.write_bytes(self.signature.data)
        content.extend(_be.finish())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        content.extend(self.tbsCertList.encode_der())







        content.extend(self.signatureAlgorithm.encode_der())







        _be = DerEncoder()
        _be.write_tag(TagClass.UNIVERSAL, 3, False)
        _be.write_length(len(self.signature.data) + 1)
        _be.write_bytes(bytes([self.signature.unused_bits]))
        _be.write_bytes(self.signature.data)
        content.extend(_be.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "CertificateList":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _tbsCertList_re = BerEncoder()
        _tbsCertList_re.write_tag(_ft[0], _ft[1], _ft[2])
        _tbsCertList_re.write_length(_fl)
        _tbsCertList_re.write_bytes(_fd)
        _tbsCertList = TBSCertList.decode_der(_tbsCertList_re.finish())







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _signatureAlgorithm_re = BerEncoder()
        _signatureAlgorithm_re.write_tag(_ft[0], _ft[1], _ft[2])
        _signatureAlgorithm_re.write_length(_fl)
        _signatureAlgorithm_re.write_bytes(_fd)
        _signatureAlgorithm = AlgorithmIdentifier.decode_der(_signatureAlgorithm_re.finish())







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _signature = BitString(_fd[1:], _fd[0])




        return cls(

            tbsCertList=_tbsCertList,

            signatureAlgorithm=_signatureAlgorithm,

            signature=_signature

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "CertificateList":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is not None:
            raise InvalidLengthError("Expected indefinite length")
        _content = decoder.read_constructed_indefinite()
        decoder2 = BerDecoder(_content)




        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _tbsCertList_re = BerEncoder()
        _tbsCertList_re.write_tag(_ft[0], _ft[1], _ft[2])
        _tbsCertList_re.write_length(_fl)
        _tbsCertList_re.write_bytes(_fd)
        _tbsCertList = TBSCertList.decode_ber(_tbsCertList_re.finish())







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _signatureAlgorithm_re = BerEncoder()
        _signatureAlgorithm_re.write_tag(_ft[0], _ft[1], _ft[2])
        _signatureAlgorithm_re.write_length(_fl)
        _signatureAlgorithm_re.write_bytes(_fd)
        _signatureAlgorithm = AlgorithmIdentifier.decode_ber(_signatureAlgorithm_re.finish())







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _signature = BitString(_fd[1:], _fd[0])




        return cls(

            tbsCertList=_tbsCertList,

            signatureAlgorithm=_signatureAlgorithm,

            signature=_signature

        )


@dataclass
class TBSCertList(AsnType):


    signature: AlgorithmIdentifier

    issuer: Name

    thisUpdate: datetime

    nextUpdate: Optional[datetime] = None

    version: Optional[int] = None




    def encode_ber(self) -> bytes:
        content = bytearray()




        content.extend(self.signature.encode_ber())







        content.extend(self.issuer.encode_ber())







        _te = BerEncoder()
        _tb = self.thisUpdate.strftime('%Y%m%d%H%M%SZ').encode('ascii')
        _te.write_tag(TagClass.UNIVERSAL, 23, False)
        _te.write_length(len(_tb))
        _te.write_bytes(_tb)
        content.extend(_te.finish())







        if self.nextUpdate is not None:


            _te = BerEncoder()
            _tb = self.nextUpdate.strftime('%Y%m%d%H%M%SZ').encode('ascii')
            _te.write_tag(TagClass.UNIVERSAL, 23, False)
            _te.write_length(len(_tb))
            _te.write_bytes(_tb)
            content.extend(_te.finish())







        if self.version is not None:


            _iv = BerEncoder()
            _iv.write_integer(self.version)
            _ib = _iv.finish()
            _ie = BerEncoder()
            _ie.write_tag(TagClass.UNIVERSAL, 2, False)
            _ie.write_length(len(_ib))
            _ie.write_bytes(_ib)
            content.extend(_ie.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        content.extend(self.signature.encode_ber())







        content.extend(self.issuer.encode_ber())







        _te = BerEncoder()
        _tb = self.thisUpdate.strftime('%Y%m%d%H%M%SZ').encode('ascii')
        _te.write_tag(TagClass.UNIVERSAL, 23, False)
        _te.write_length(len(_tb))
        _te.write_bytes(_tb)
        content.extend(_te.finish())







        if self.nextUpdate is not None:


            _te = BerEncoder()
            _tb = self.nextUpdate.strftime('%Y%m%d%H%M%SZ').encode('ascii')
            _te.write_tag(TagClass.UNIVERSAL, 23, False)
            _te.write_length(len(_tb))
            _te.write_bytes(_tb)
            content.extend(_te.finish())







        if self.version is not None:


            _iv = BerEncoder()
            _iv.write_integer(self.version)
            _ib = _iv.finish()
            _ie = BerEncoder()
            _ie.write_tag(TagClass.UNIVERSAL, 2, False)
            _ie.write_length(len(_ib))
            _ie.write_bytes(_ib)
            content.extend(_ie.finish())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        content.extend(self.signature.encode_der())







        content.extend(self.issuer.encode_der())







        _te = DerEncoder()
        _tb = self.thisUpdate.strftime('%Y%m%d%H%M%SZ').encode('ascii')
        _te.write_tag(TagClass.UNIVERSAL, 23, False)
        _te.write_length(len(_tb))
        _te.write_bytes(_tb)
        content.extend(_te.finish())







        if self.nextUpdate is not None:


            _te = BerEncoder()
            _tb = self.nextUpdate.strftime('%Y%m%d%H%M%SZ').encode('ascii')
            _te.write_tag(TagClass.UNIVERSAL, 23, False)
            _te.write_length(len(_tb))
            _te.write_bytes(_tb)
            content.extend(_te.finish())







        if self.version is not None:


            _iv = DerEncoder()
            _iv.write_integer(self.version)
            _ib = _iv.finish()
            _ie = DerEncoder()
            _ie.write_tag(TagClass.UNIVERSAL, 2, False)
            _ie.write_length(len(_ib))
            _ie.write_bytes(_ib)
            content.extend(_ie.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "TBSCertList":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _signature_re = BerEncoder()
        _signature_re.write_tag(_ft[0], _ft[1], _ft[2])
        _signature_re.write_length(_fl)
        _signature_re.write_bytes(_fd)
        _signature = AlgorithmIdentifier.decode_der(_signature_re.finish())







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _issuer_re = BerEncoder()
        _issuer_re.write_tag(_ft[0], _ft[1], _ft[2])
        _issuer_re.write_length(_fl)
        _issuer_re.write_bytes(_fd)
        _issuer = Name.decode_der(_issuer_re.finish())







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        from datetime import datetime
        _thisUpdate = datetime.strptime(_fd.decode('ascii'), '%Y%m%d%H%M%SZ')







        _nextUpdate = None

        if decoder._pos < _end:
            _nextUpdate_save = decoder._pos
            _ft = decoder.read_tag()
            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == 23 and not _ft[2]:
                _fl = decoder.read_length()
                _fd = decoder.read_bytes(_fl)

                from datetime import datetime
                _nextUpdate = datetime.strptime(_fd.decode('ascii'), '%Y%m%d%H%M%SZ')

            else:
                decoder._pos = _nextUpdate_save







        _version = None

        if decoder._pos < _end:
            _version_save = decoder._pos
            _ft = decoder.read_tag()
            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == 2 and not _ft[2]:
                _fl = decoder.read_length()
                _fd = decoder.read_bytes(_fl)

                _version = int.from_bytes(_fd, byteorder='big', signed=True)

            else:
                decoder._pos = _version_save




        return cls(

            signature=_signature,

            issuer=_issuer,

            thisUpdate=_thisUpdate,

            nextUpdate=_nextUpdate,

            version=_version

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "TBSCertList":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is not None:
            raise InvalidLengthError("Expected indefinite length")
        _content = decoder.read_constructed_indefinite()
        decoder2 = BerDecoder(_content)




        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _signature_re = BerEncoder()
        _signature_re.write_tag(_ft[0], _ft[1], _ft[2])
        _signature_re.write_length(_fl)
        _signature_re.write_bytes(_fd)
        _signature = AlgorithmIdentifier.decode_ber(_signature_re.finish())







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _issuer_re = BerEncoder()
        _issuer_re.write_tag(_ft[0], _ft[1], _ft[2])
        _issuer_re.write_length(_fl)
        _issuer_re.write_bytes(_fd)
        _issuer = Name.decode_ber(_issuer_re.finish())







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        from datetime import datetime
        _thisUpdate = datetime.strptime(_fd.decode('ascii'), '%Y%m%d%H%M%SZ')







        _nextUpdate = None

        if not decoder2.at_end():
            _nextUpdate_save = decoder2._pos
            _ft = decoder2.read_tag()
            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == 23 and not _ft[2]:
                _fl = decoder2.read_length()
                _fd = decoder2.read_bytes(_fl)

                from datetime import datetime
                _nextUpdate = datetime.strptime(_fd.decode('ascii'), '%Y%m%d%H%M%SZ')

            else:
                decoder2._pos = _nextUpdate_save







        _version = None

        if not decoder2.at_end():
            _version_save = decoder2._pos
            _ft = decoder2.read_tag()
            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == 2 and not _ft[2]:
                _fl = decoder2.read_length()
                _fd = decoder2.read_bytes(_fl)

                _version = int.from_bytes(_fd, byteorder='big', signed=True)

            else:
                decoder2._pos = _version_save




        return cls(

            signature=_signature,

            issuer=_issuer,

            thisUpdate=_thisUpdate,

            nextUpdate=_nextUpdate,

            version=_version

        )
