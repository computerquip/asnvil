"""Generated from LDAPv3.asn1 by asnvil."""
from __future__ import annotations
from asnvil_runtime import AsnType, Tag, TagClass, BerEncoder, BerDecoder, DerEncoder, DerDecoder, BitString, ObjectIdentifier, AsnError, InvalidLengthError
from dataclasses import dataclass, field
from typing import Optional
from enum import Enum, IntEnum
from datetime import datetime


MessageID = int

LDAPString = bytes

LDAPDN = bytes

AttributeName = bytes

AttributeValue = bytes

@dataclass
class AttributeValueAssertion(AsnType):


    attributeDesc: bytes

    assertionValue: bytes




    def encode_ber(self) -> bytes:
        content = bytearray()




        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.attributeDesc)
        content.extend(_be.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.assertionValue)
        content.extend(_be.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.attributeDesc)
        content.extend(_be.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.assertionValue)
        content.extend(_be.finish())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        _be = DerEncoder()
        _be.write_tlv_der(TagClass.UNIVERSAL, 4, self.attributeDesc)
        content.extend(_be.finish())







        _be = DerEncoder()
        _be.write_tlv_der(TagClass.UNIVERSAL, 4, self.assertionValue)
        content.extend(_be.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "AttributeValueAssertion":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _attributeDesc = decoder.read_bytes(_fl)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _assertionValue = decoder.read_bytes(_fl)




        return cls(

            attributeDesc=_attributeDesc,

            assertionValue=_assertionValue

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "AttributeValueAssertion":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is not None:
            raise InvalidLengthError("Expected indefinite length")
        _content = decoder.read_constructed_indefinite()
        decoder2 = BerDecoder(_content)




        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _attributeDesc = decoder2.read_bytes(_fl)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _assertionValue = decoder2.read_bytes(_fl)




        return cls(

            attributeDesc=_attributeDesc,

            assertionValue=_assertionValue

        )


@dataclass
class PartialAttribute(AsnType):


    typeAttr: bytes

    vals: list[AttributeValue]




    def encode_ber(self) -> bytes:
        content = bytearray()




        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.typeAttr)
        content.extend(_be.finish())







        _le = BerEncoder()
        _lc = bytearray()
        for _li in self.vals:


            _li_e = BerEncoder()
            _li_e.write_tlv(TagClass.UNIVERSAL, 4, _li)
            _lc.extend(_li_e.finish())


        _le.write_tag(TagClass.UNIVERSAL, 17, True)
        _le.write_length(len(_lc))
        _le.write_bytes(_lc)
        content.extend(_le.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.typeAttr)
        content.extend(_be.finish())







        _le = BerEncoder()
        _lc = bytearray()
        for _li in self.vals:


            _li_e = BerEncoder()
            _li_e.write_tlv(TagClass.UNIVERSAL, 4, _li)
            _lc.extend(_li_e.finish())


        _le.write_tag(TagClass.UNIVERSAL, 17, True)
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




        _be = DerEncoder()
        _be.write_tlv_der(TagClass.UNIVERSAL, 4, self.typeAttr)
        content.extend(_be.finish())







        _le = DerEncoder()
        _lc = bytearray()
        for _li in self.vals:


            _li_e = DerEncoder()
            _li_e.write_tlv_der(TagClass.UNIVERSAL, 4, _li)
            _lc.extend(_li_e.finish())


        _le.write_tag(TagClass.UNIVERSAL, 17, True)
        _le.write_length(len(_lc))
        _le.write_bytes(_lc)
        content.extend(_le.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "PartialAttribute":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _typeAttr = decoder.read_bytes(_fl)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _ld = DerDecoder(_fd)
        _vals = []
        while not _ld.at_end():


            _lt = _ld.read_tag()
            _ll = _ld.read_length()
            _vals.append(_ld.read_bytes(_ll))






        return cls(

            typeAttr=_typeAttr,

            vals=_vals

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "PartialAttribute":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is not None:
            raise InvalidLengthError("Expected indefinite length")
        _content = decoder.read_constructed_indefinite()
        decoder2 = BerDecoder(_content)




        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _typeAttr = decoder2.read_bytes(_fl)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _ld = BerDecoder(_fd)
        _vals = []
        while not _ld.at_end():


            _lt = _ld.read_tag()
            _ll = _ld.read_length()
            _vals.append(_ld.read_bytes(_ll))






        return cls(

            typeAttr=_typeAttr,

            vals=_vals

        )


@dataclass
class LDAPResult(AsnType):


    resultCode: int

    matchedDN: bytes

    diagnosticMessage: bytes




    def encode_ber(self) -> bytes:
        content = bytearray()




        _iv = BerEncoder()
        _iv.write_integer(self.resultCode.value if hasattr(self.resultCode, 'value') else self.resultCode)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 10, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.matchedDN)
        content.extend(_be.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.diagnosticMessage)
        content.extend(_be.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _iv = BerEncoder()
        _iv.write_integer(self.resultCode.value if hasattr(self.resultCode, 'value') else self.resultCode)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 10, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.matchedDN)
        content.extend(_be.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.diagnosticMessage)
        content.extend(_be.finish())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        _iv = DerEncoder()
        _iv.write_integer(self.resultCode.value if hasattr(self.resultCode, 'value') else self.resultCode)
        _ib = _iv.finish()
        _ie = DerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 10, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _be = DerEncoder()
        _be.write_tlv_der(TagClass.UNIVERSAL, 4, self.matchedDN)
        content.extend(_be.finish())







        _be = DerEncoder()
        _be.write_tlv_der(TagClass.UNIVERSAL, 4, self.diagnosticMessage)
        content.extend(_be.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "LDAPResult":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _resultCode = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _matchedDN = decoder.read_bytes(_fl)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _diagnosticMessage = decoder.read_bytes(_fl)




        return cls(

            resultCode=_resultCode,

            matchedDN=_matchedDN,

            diagnosticMessage=_diagnosticMessage

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "LDAPResult":
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
        _resultCode = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _matchedDN = decoder2.read_bytes(_fl)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _diagnosticMessage = decoder2.read_bytes(_fl)




        return cls(

            resultCode=_resultCode,

            matchedDN=_matchedDN,

            diagnosticMessage=_diagnosticMessage

        )


class PartialAttributeList(list, AsnType):

    def __init__(self, items: list[PartialAttribute] | None = None):
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
    def decode_ber(cls, data: bytes) -> "PartialAttributeList":
        return cls.decode_der(data)

    def encode_der(self) -> bytes:
        return self.encode_ber()

    @classmethod
    def decode_der(cls, data: bytes) -> "PartialAttributeList":
        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length
        items: list[PartialAttribute] = []
        while decoder._pos < _end:


            _item_start = decoder._pos
            _lt = decoder.read_tag()
            _ll = decoder.read_length()
            _ = decoder.read_bytes(_ll)
            items.append(PartialAttribute.decode_der(decoder._data[_item_start:decoder._pos]))


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
    def decode_ber_indefinite(cls, data: bytes) -> "PartialAttributeList":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is None:
            _content = decoder.read_constructed_indefinite()
        else:
            _content = decoder.read_bytes(_length)
        items: list[PartialAttribute] = []
        _ld = BerDecoder(_content)
        while not _ld.at_end():


            _item_start = _ld._pos
            _lt = _ld.read_tag()
            _ll = _ld.read_length()
            _ = _ld.read_bytes(_ll)
            items.append(PartialAttribute.decode_der(_ld._data[_item_start:_ld._pos]))


        return cls(items)

@dataclass
class SearchResultEntry(AsnType):


    objectName: bytes

    attributes: list[PartialAttribute]




    def encode_ber(self) -> bytes:
        content = bytearray()




        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.objectName)
        content.extend(_be.finish())







        _le = BerEncoder()
        _lc = bytearray()
        for _li in self.attributes:


            _lc.extend(_li.encode_ber())


        _le.write_tag(TagClass.UNIVERSAL, 16, True)
        _le.write_length(len(_lc))
        _le.write_bytes(_lc)
        content.extend(_le.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.objectName)
        content.extend(_be.finish())







        _le = BerEncoder()
        _lc = bytearray()
        for _li in self.attributes:


            _lc.extend(_li.encode_ber())


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




        _be = DerEncoder()
        _be.write_tlv_der(TagClass.UNIVERSAL, 4, self.objectName)
        content.extend(_be.finish())







        _le = DerEncoder()
        _lc = bytearray()
        for _li in self.attributes:


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
    def decode_der(cls, data: bytes) -> "SearchResultEntry":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _objectName = decoder.read_bytes(_fl)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _ld = DerDecoder(_fd)
        _attributes = []
        while not _ld.at_end():


            _attributes_start = _ld._pos
            _lt = _ld.read_tag()
            _ll = _ld.read_length()
            _lv = _ld.read_bytes(_ll)
            _attributes.append(PartialAttribute.decode_der(_ld._data[_attributes_start:_ld._pos]))






        return cls(

            objectName=_objectName,

            attributes=_attributes

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "SearchResultEntry":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is not None:
            raise InvalidLengthError("Expected indefinite length")
        _content = decoder.read_constructed_indefinite()
        decoder2 = BerDecoder(_content)




        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _objectName = decoder2.read_bytes(_fl)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _ld = BerDecoder(_fd)
        _attributes = []
        while not _ld.at_end():


            _attributes_start = _ld._pos
            _lt = _ld.read_tag()
            _ll = _ld.read_length()
            _lv = _ld.read_bytes(_ll)
            _attributes.append(PartialAttribute.decode_der(_ld._data[_attributes_start:_ld._pos]))






        return cls(

            objectName=_objectName,

            attributes=_attributes

        )


@dataclass
class BindRequest(AsnType):


    version: int

    name: bytes

    authentication: bytes




    def encode_ber(self) -> bytes:
        content = bytearray()




        _iv = BerEncoder()
        _iv.write_integer(self.version)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.name)
        content.extend(_be.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.authentication)
        content.extend(_be.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _iv = BerEncoder()
        _iv.write_integer(self.version)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.name)
        content.extend(_be.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.authentication)
        content.extend(_be.finish())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        _iv = DerEncoder()
        _iv.write_integer(self.version)
        _ib = _iv.finish()
        _ie = DerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _be = DerEncoder()
        _be.write_tlv_der(TagClass.UNIVERSAL, 4, self.name)
        content.extend(_be.finish())







        _be = DerEncoder()
        _be.write_tlv_der(TagClass.UNIVERSAL, 4, self.authentication)
        content.extend(_be.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "BindRequest":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _version = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _name = decoder.read_bytes(_fl)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _authentication = decoder.read_bytes(_fl)




        return cls(

            version=_version,

            name=_name,

            authentication=_authentication

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "BindRequest":
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
        _version = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _name = decoder2.read_bytes(_fl)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _authentication = decoder2.read_bytes(_fl)




        return cls(

            version=_version,

            name=_name,

            authentication=_authentication

        )


@dataclass
class BindResponse(AsnType):


    resultCode: int

    matchedDN: bytes

    diagnosticMessage: bytes




    def encode_ber(self) -> bytes:
        content = bytearray()




        _iv = BerEncoder()
        _iv.write_integer(self.resultCode.value if hasattr(self.resultCode, 'value') else self.resultCode)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 10, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.matchedDN)
        content.extend(_be.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.diagnosticMessage)
        content.extend(_be.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _iv = BerEncoder()
        _iv.write_integer(self.resultCode.value if hasattr(self.resultCode, 'value') else self.resultCode)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 10, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.matchedDN)
        content.extend(_be.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.diagnosticMessage)
        content.extend(_be.finish())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        _iv = DerEncoder()
        _iv.write_integer(self.resultCode.value if hasattr(self.resultCode, 'value') else self.resultCode)
        _ib = _iv.finish()
        _ie = DerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 10, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _be = DerEncoder()
        _be.write_tlv_der(TagClass.UNIVERSAL, 4, self.matchedDN)
        content.extend(_be.finish())







        _be = DerEncoder()
        _be.write_tlv_der(TagClass.UNIVERSAL, 4, self.diagnosticMessage)
        content.extend(_be.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "BindResponse":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _resultCode = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _matchedDN = decoder.read_bytes(_fl)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _diagnosticMessage = decoder.read_bytes(_fl)




        return cls(

            resultCode=_resultCode,

            matchedDN=_matchedDN,

            diagnosticMessage=_diagnosticMessage

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "BindResponse":
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
        _resultCode = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _matchedDN = decoder2.read_bytes(_fl)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _diagnosticMessage = decoder2.read_bytes(_fl)




        return cls(

            resultCode=_resultCode,

            matchedDN=_matchedDN,

            diagnosticMessage=_diagnosticMessage

        )


@dataclass
class SearchRequest(AsnType):


    baseObject: bytes

    scope: int

    derefAliases: int

    sizeLimit: int

    timeLimit: int

    typesOnly: bool

    filter: bytes

    attributes: list[AttributeName]




    def encode_ber(self) -> bytes:
        content = bytearray()




        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.baseObject)
        content.extend(_be.finish())







        _iv = BerEncoder()
        _iv.write_integer(self.scope.value if hasattr(self.scope, 'value') else self.scope)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 10, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _iv = BerEncoder()
        _iv.write_integer(self.derefAliases.value if hasattr(self.derefAliases, 'value') else self.derefAliases)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 10, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _iv = BerEncoder()
        _iv.write_integer(self.sizeLimit)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _iv = BerEncoder()
        _iv.write_integer(self.timeLimit)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _be = BerEncoder()
        _be.write_tag(TagClass.UNIVERSAL, 1, False)
        _be.write_length(1)
        _be.write_bytes(b'\xff' if self.typesOnly else b'\x00')
        content.extend(_be.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.filter)
        content.extend(_be.finish())







        _le = BerEncoder()
        _lc = bytearray()
        for _li in self.attributes:


            _li_e = BerEncoder()
            _li_e.write_tlv(TagClass.UNIVERSAL, 4, _li)
            _lc.extend(_li_e.finish())


        _le.write_tag(TagClass.UNIVERSAL, 16, True)
        _le.write_length(len(_lc))
        _le.write_bytes(_lc)
        content.extend(_le.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.baseObject)
        content.extend(_be.finish())







        _iv = BerEncoder()
        _iv.write_integer(self.scope.value if hasattr(self.scope, 'value') else self.scope)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 10, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _iv = BerEncoder()
        _iv.write_integer(self.derefAliases.value if hasattr(self.derefAliases, 'value') else self.derefAliases)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 10, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _iv = BerEncoder()
        _iv.write_integer(self.sizeLimit)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _iv = BerEncoder()
        _iv.write_integer(self.timeLimit)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _be = BerEncoder()
        _be.write_tag(TagClass.UNIVERSAL, 1, False)
        _be.write_length(1)
        _be.write_bytes(b'\xff' if self.typesOnly else b'\x00')
        content.extend(_be.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.filter)
        content.extend(_be.finish())







        _le = BerEncoder()
        _lc = bytearray()
        for _li in self.attributes:


            _li_e = BerEncoder()
            _li_e.write_tlv(TagClass.UNIVERSAL, 4, _li)
            _lc.extend(_li_e.finish())


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




        _be = DerEncoder()
        _be.write_tlv_der(TagClass.UNIVERSAL, 4, self.baseObject)
        content.extend(_be.finish())







        _iv = DerEncoder()
        _iv.write_integer(self.scope.value if hasattr(self.scope, 'value') else self.scope)
        _ib = _iv.finish()
        _ie = DerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 10, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _iv = DerEncoder()
        _iv.write_integer(self.derefAliases.value if hasattr(self.derefAliases, 'value') else self.derefAliases)
        _ib = _iv.finish()
        _ie = DerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 10, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _iv = DerEncoder()
        _iv.write_integer(self.sizeLimit)
        _ib = _iv.finish()
        _ie = DerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _iv = DerEncoder()
        _iv.write_integer(self.timeLimit)
        _ib = _iv.finish()
        _ie = DerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _be = DerEncoder()
        _be.write_tag(TagClass.UNIVERSAL, 1, False)
        _be.write_length(1)
        _be.write_bytes(b'\xff' if self.typesOnly else b'\x00')
        content.extend(_be.finish())







        _be = DerEncoder()
        _be.write_tlv_der(TagClass.UNIVERSAL, 4, self.filter)
        content.extend(_be.finish())







        _le = DerEncoder()
        _lc = bytearray()
        for _li in self.attributes:


            _li_e = DerEncoder()
            _li_e.write_tlv_der(TagClass.UNIVERSAL, 4, _li)
            _lc.extend(_li_e.finish())


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
    def decode_der(cls, data: bytes) -> "SearchRequest":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _baseObject = decoder.read_bytes(_fl)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _scope = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _derefAliases = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _sizeLimit = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _timeLimit = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _typesOnly = _fd[0] != 0







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _filter = decoder.read_bytes(_fl)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _ld = DerDecoder(_fd)
        _attributes = []
        while not _ld.at_end():


            _lt = _ld.read_tag()
            _ll = _ld.read_length()
            _attributes.append(_ld.read_bytes(_ll))






        return cls(

            baseObject=_baseObject,

            scope=_scope,

            derefAliases=_derefAliases,

            sizeLimit=_sizeLimit,

            timeLimit=_timeLimit,

            typesOnly=_typesOnly,

            filter=_filter,

            attributes=_attributes

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "SearchRequest":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is not None:
            raise InvalidLengthError("Expected indefinite length")
        _content = decoder.read_constructed_indefinite()
        decoder2 = BerDecoder(_content)




        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _baseObject = decoder2.read_bytes(_fl)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _scope = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _derefAliases = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _sizeLimit = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _timeLimit = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _typesOnly = _fd[0] != 0







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _filter = decoder2.read_bytes(_fl)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _ld = BerDecoder(_fd)
        _attributes = []
        while not _ld.at_end():


            _lt = _ld.read_tag()
            _ll = _ld.read_length()
            _attributes.append(_ld.read_bytes(_ll))






        return cls(

            baseObject=_baseObject,

            scope=_scope,

            derefAliases=_derefAliases,

            sizeLimit=_sizeLimit,

            timeLimit=_timeLimit,

            typesOnly=_typesOnly,

            filter=_filter,

            attributes=_attributes

        )


@dataclass
class LDAPMessage(AsnType):


    messageID: int

    protocolOp: bytes

    controls: Optional[list[bytes]] = None




    def encode_ber(self) -> bytes:
        content = bytearray()




        _iv = BerEncoder()
        _iv.write_integer(self.messageID)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.protocolOp)
        content.extend(_be.finish())







        if self.controls is not None:


            _le = BerEncoder()
            _lc = bytearray()
            for _li in self.controls:


                _li_e = BerEncoder()
                _li_e.write_tlv(TagClass.UNIVERSAL, 4, _li)
                _lc.extend(_li_e.finish())


            _le.write_tag(TagClass.UNIVERSAL, 16, True)
            _le.write_length(len(_lc))
            _le.write_bytes(_lc)
            content.extend(_le.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _iv = BerEncoder()
        _iv.write_integer(self.messageID)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _be = BerEncoder()
        _be.write_tlv(TagClass.UNIVERSAL, 4, self.protocolOp)
        content.extend(_be.finish())







        if self.controls is not None:


            _le = BerEncoder()
            _lc = bytearray()
            for _li in self.controls:


                _li_e = BerEncoder()
                _li_e.write_tlv(TagClass.UNIVERSAL, 4, _li)
                _lc.extend(_li_e.finish())


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




        _iv = DerEncoder()
        _iv.write_integer(self.messageID)
        _ib = _iv.finish()
        _ie = DerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _be = DerEncoder()
        _be.write_tlv_der(TagClass.UNIVERSAL, 4, self.protocolOp)
        content.extend(_be.finish())







        if self.controls is not None:


            _le = DerEncoder()
            _lc = bytearray()
            for _li in self.controls:


                _li_e = DerEncoder()
                _li_e.write_tlv_der(TagClass.UNIVERSAL, 4, _li)
                _lc.extend(_li_e.finish())


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
    def decode_der(cls, data: bytes) -> "LDAPMessage":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _messageID = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _protocolOp = decoder.read_bytes(_fl)







        _controls = None

        if decoder._pos < _end:
            _controls_save = decoder._pos
            _ft = decoder.read_tag()
            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == 16 and not _ft[2]:
                _fl = decoder.read_length()
                _fd = decoder.read_bytes(_fl)

                _ld = DerDecoder(_fd)
                _controls = []
                while not _ld.at_end():


                    _lt = _ld.read_tag()
                    _ll = _ld.read_length()
                    _controls.append(_ld.read_bytes(_ll))



            else:
                decoder._pos = _controls_save




        return cls(

            messageID=_messageID,

            protocolOp=_protocolOp,

            controls=_controls

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "LDAPMessage":
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
        _messageID = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _protocolOp = decoder2.read_bytes(_fl)







        _controls = None

        if not decoder2.at_end():
            _controls_save = decoder2._pos
            _ft = decoder2.read_tag()
            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == 16 and not _ft[2]:
                _fl = decoder2.read_length()
                _fd = decoder2.read_bytes(_fl)

                _ld = BerDecoder(_fd)
                _controls = []
                while not _ld.at_end():


                    _lt = _ld.read_tag()
                    _ll = _ld.read_length()
                    _controls.append(_ld.read_bytes(_ll))



            else:
                decoder2._pos = _controls_save




        return cls(

            messageID=_messageID,

            protocolOp=_protocolOp,

            controls=_controls

        )
