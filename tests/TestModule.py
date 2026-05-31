"""Generated from TestModule.asn1 by asn1c."""
from __future__ import annotations
from asn1c_runtime import AsnType, Tag, TagClass, BerEncoder, BerDecoder, DerEncoder, DerDecoder, BitString, ObjectIdentifier, AsnError, InvalidLengthError
from dataclasses import dataclass, field
from typing import Optional
from enum import Enum, IntEnum
from datetime import datetime


@dataclass
class Person(AsnType):


    name: str

    age: int

    active: Optional[bool] = True




    def encode_ber(self) -> bytes:
        content = bytearray()




        _se = BerEncoder()
        _sb = self.name.encode('utf-8')
        _se.write_tag(TagClass.UNIVERSAL, 12, False)
        _se.write_length(len(_sb))
        _se.write_bytes(_sb)
        content.extend(_se.finish())







        _iv = BerEncoder()
        _iv.write_integer(self.age)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        if self.active is not None and self.active != True:


            _be = BerEncoder()
            _be.write_tag(TagClass.UNIVERSAL, 1, False)
            _be.write_length(1)
            _be.write_bytes(b'\xff' if self.active else b'\x00')
            content.extend(_be.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _se = BerEncoder()
        _sb = self.name.encode('utf-8')
        _se.write_tag(TagClass.UNIVERSAL, 12, False)
        _se.write_length(len(_sb))
        _se.write_bytes(_sb)
        content.extend(_se.finish())







        _iv = BerEncoder()
        _iv.write_integer(self.age)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        if self.active is not None and self.active != True:


            _be = BerEncoder()
            _be.write_tag(TagClass.UNIVERSAL, 1, False)
            _be.write_length(1)
            _be.write_bytes(b'\xff' if self.active else b'\x00')
            content.extend(_be.finish())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        _se = DerEncoder()
        _sb = self.name.encode('utf-8')
        _se.write_tag(TagClass.UNIVERSAL, 12, False)
        _se.write_length(len(_sb))
        _se.write_bytes(_sb)
        content.extend(_se.finish())







        _iv = DerEncoder()
        _iv.write_integer(self.age)
        _ib = _iv.finish()
        _ie = DerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        if self.active is not None and self.active != True:


            _be = DerEncoder()
            _be.write_tag(TagClass.UNIVERSAL, 1, False)
            _be.write_length(1)
            _be.write_bytes(b'\xff' if self.active else b'\x00')
            content.extend(_be.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "Person":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _name = _fd.decode('utf-8')







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _age = int.from_bytes(_fd, byteorder='big', signed=True)







        _active = True

        if decoder._pos < _end:
            _active_save = decoder._pos
            _ft = decoder.read_tag()
            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == 1 and not _ft[2]:
                _fl = decoder.read_length()
                _fd = decoder.read_bytes(_fl)

                _active = _fd[0] != 0

            else:
                decoder._pos = _active_save

                _active = True




        return cls(

            name=_name,

            age=_age,

            active=_active

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "Person":
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
        _name = _fd.decode('utf-8')







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _age = int.from_bytes(_fd, byteorder='big', signed=True)







        _active = True

        if not decoder2.at_end():
            _active_save = decoder2._pos
            _ft = decoder2.read_tag()
            if _ft[0] == TagClass.UNIVERSAL and _ft[1] == 1 and not _ft[2]:
                _fl = decoder2.read_length()
                _fd = decoder2.read_bytes(_fl)

                _active = _fd[0] != 0

            else:
                decoder2._pos = _active_save

                _active = True




        return cls(

            name=_name,

            age=_age,

            active=_active

        )


@dataclass
class Department(AsnType):


    deptName: str

    code: int

    location: str




    def encode_ber(self) -> bytes:
        content = bytearray()




        _se = BerEncoder()
        _sb = self.deptName.encode('utf-8')
        _se.write_tag(TagClass.UNIVERSAL, 12, False)
        _se.write_length(len(_sb))
        _se.write_bytes(_sb)
        content.extend(_se.finish())







        _iv = BerEncoder()
        _iv.write_integer(self.code)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _se = BerEncoder()
        _sb = self.location.encode('utf-8')
        _se.write_tag(TagClass.UNIVERSAL, 12, False)
        _se.write_length(len(_sb))
        _se.write_bytes(_sb)
        content.extend(_se.finish())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _se = BerEncoder()
        _sb = self.deptName.encode('utf-8')
        _se.write_tag(TagClass.UNIVERSAL, 12, False)
        _se.write_length(len(_sb))
        _se.write_bytes(_sb)
        content.extend(_se.finish())







        _iv = BerEncoder()
        _iv.write_integer(self.code)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _se = BerEncoder()
        _sb = self.location.encode('utf-8')
        _se.write_tag(TagClass.UNIVERSAL, 12, False)
        _se.write_length(len(_sb))
        _se.write_bytes(_sb)
        content.extend(_se.finish())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        _se = DerEncoder()
        _sb = self.deptName.encode('utf-8')
        _se.write_tag(TagClass.UNIVERSAL, 12, False)
        _se.write_length(len(_sb))
        _se.write_bytes(_sb)
        content.extend(_se.finish())







        _iv = DerEncoder()
        _iv.write_integer(self.code)
        _ib = _iv.finish()
        _ie = DerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        _se = DerEncoder()
        _sb = self.location.encode('utf-8')
        _se.write_tag(TagClass.UNIVERSAL, 12, False)
        _se.write_length(len(_sb))
        _se.write_bytes(_sb)
        content.extend(_se.finish())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "Department":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _deptName = _fd.decode('utf-8')







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _code = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _location = _fd.decode('utf-8')




        return cls(

            deptName=_deptName,

            code=_code,

            location=_location

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "Department":
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
        _deptName = _fd.decode('utf-8')







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _code = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _location = _fd.decode('utf-8')




        return cls(

            deptName=_deptName,

            code=_code,

            location=_location

        )


@dataclass
class Entity(AsnType):


    person: Optional[Person] = None

    department: Optional[Department] = None

    flag: Optional[bool] = None


    def encode_ber(self) -> bytes:




        if self.person is not None:
            _inner = self.person.encode_ber()
            _e = BerEncoder()
            _e.write_tag(TagClass.CONTEXT, 0, True)
            _e.write_length(len(_inner))
            _e.write_bytes(_inner)
            return _e.finish()







        if self.department is not None:
            _inner = self.department.encode_ber()
            _e = BerEncoder()
            _e.write_tag(TagClass.CONTEXT, 1, True)
            _e.write_length(len(_inner))
            _e.write_bytes(_inner)
            return _e.finish()







        if self.flag is not None:
            _be = BerEncoder()
            _be.write_tag(TagClass.UNIVERSAL, 1, False)
            _be.write_length(1)
            _be.write_bytes(b'\xff' if self.flag else b'\x00')
            _inner = _be.finish()
            _e = BerEncoder()
            _e.write_tag(TagClass.CONTEXT, 2, True)
            _e.write_length(len(_inner))
            _e.write_bytes(_inner)
            return _e.finish()




        raise ValueError("No choice alternative set")

    @classmethod
    def decode_ber(cls, data: bytes) -> "Entity":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)




        if _tag[0] == TagClass.CONTEXT and _tag[1] == 0:
            return cls(person=Person.decode_ber(_fd))







        if _tag[0] == TagClass.CONTEXT and _tag[1] == 1:
            return cls(department=Department.decode_ber(_fd))







        if _tag[0] == TagClass.CONTEXT and _tag[1] == 2:
            return cls(flag=_fd[0] != 0)











        raise ValueError(f"Unknown choice tag: {_tag}")

    def encode_ber_indefinite(self) -> bytes:




        if self.person is not None:
            _inner = self.person.encode_ber()
            _e = BerEncoder()
            _e.write_tag(TagClass.CONTEXT, 0, True)
            _e.write_length(0, indefinite=True)
            _e.write_bytes(_inner)
            _e.write_eoc()
            return _e.finish()







        if self.department is not None:
            _inner = self.department.encode_ber()
            _e = BerEncoder()
            _e.write_tag(TagClass.CONTEXT, 1, True)
            _e.write_length(0, indefinite=True)
            _e.write_bytes(_inner)
            _e.write_eoc()
            return _e.finish()







        if self.flag is not None:
            _iv = BerEncoder()

            _iv.write_tag(TagClass.UNIVERSAL, 1, False)
            _iv.write_length(1)
            _iv.write_bytes(b'\xff' if self.flag else b'\x00')

            _inner = _iv.finish()
            _e = BerEncoder()
            _e.write_tag(TagClass.CONTEXT, 2, True)
            _e.write_length(0, indefinite=True)
            _e.write_bytes(_inner)
            _e.write_eoc()
            return _e.finish()




        raise ValueError("No choice alternative set")

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "Entity":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is None:
            _content = decoder.read_constructed_indefinite()
        else:
            _content = decoder.read_bytes(_length)



        if _tag[0] == TagClass.CONTEXT and _tag[1] == 0:
            return cls(person=Person.decode_der(_content))





        if _tag[0] == TagClass.CONTEXT and _tag[1] == 1:
            return cls(department=Department.decode_der(_content))





        if _tag[0] == TagClass.CONTEXT and _tag[1] == 2:
            return cls(flag=_content[0] != 0)










        raise ValueError(f"Unknown choice tag: {_tag}")

    def encode_der(self) -> bytes:




        if self.person is not None:
            _inner = self.person.encode_der()
            _e = DerEncoder()
            _e.write_tag(TagClass.CONTEXT, 0, True)
            _e.write_length(len(_inner))
            _e.write_bytes(_inner)
            return _e.finish()







        if self.department is not None:
            _inner = self.department.encode_der()
            _e = DerEncoder()
            _e.write_tag(TagClass.CONTEXT, 1, True)
            _e.write_length(len(_inner))
            _e.write_bytes(_inner)
            return _e.finish()







        if self.flag is not None:
            _be = DerEncoder()
            _be.write_tag(TagClass.UNIVERSAL, 1, False)
            _be.write_length(1)
            _be.write_bytes(b'\xff' if self.flag else b'\x00')
            _inner = _be.finish()
            _e = DerEncoder()
            _e.write_tag(TagClass.CONTEXT, 2, True)
            _e.write_length(len(_inner))
            _e.write_bytes(_inner)
            return _e.finish()




        raise ValueError("No choice alternative set")

    @classmethod
    def decode_der(cls, data: bytes) -> "Entity":
        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)




        if _tag[0] == TagClass.CONTEXT and _tag[1] == 0:
            return cls(person=Person.decode_der(_fd))







        if _tag[0] == TagClass.CONTEXT and _tag[1] == 1:
            return cls(department=Department.decode_der(_fd))







        if _tag[0] == TagClass.CONTEXT and _tag[1] == 2:
            return cls(flag=_fd[0] != 0)











        raise ValueError(f"Unknown choice tag: {_tag}")

@dataclass
class Container(AsnType):


    id: int

    entity: Entity




    def encode_ber(self) -> bytes:
        content = bytearray()




        _iv = BerEncoder()
        _iv.write_integer(self.id)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        content.extend(self.entity.encode_ber())




        return bytes(content)

    def encode_ber_indefinite(self) -> bytes:
        content = bytearray()




        _iv = BerEncoder()
        _iv.write_integer(self.id)
        _ib = _iv.finish()
        _ie = BerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        content.extend(self.entity.encode_ber())




        _outer = BerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(0, indefinite=True)
        _outer.write_bytes(content)
        _outer.write_eoc()
        return _outer.finish()

    def encode_der(self) -> bytes:

        content = bytearray()




        _iv = DerEncoder()
        _iv.write_integer(self.id)
        _ib = _iv.finish()
        _ie = DerEncoder()
        _ie.write_tag(TagClass.UNIVERSAL, 2, False)
        _ie.write_length(len(_ib))
        _ie.write_bytes(_ib)
        content.extend(_ie.finish())







        content.extend(self.entity.encode_der())




        _outer = DerEncoder()
        _outer.write_tag(TagClass.UNIVERSAL, 16, True)
        _outer.write_length(len(content))
        _outer.write_bytes(content)
        return _outer.finish()

    @classmethod
    def decode_der(cls, data: bytes) -> "Container":


        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        _end = decoder._pos + _length




        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _id = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)
        _entity_re = BerEncoder()
        _entity_re.write_tag(_ft[0], _ft[1], _ft[2])
        _entity_re.write_length(_fl)
        _entity_re.write_bytes(_fd)
        _entity = Entity.decode_der(_entity_re.finish())




        return cls(

            id=_id,

            entity=_entity

        )

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "Container":
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
        _id = int.from_bytes(_fd, byteorder='big', signed=True)







        _ft = decoder2.read_tag()
        _fl = decoder2.read_length()
        _fd = decoder2.read_bytes(_fl)
        _entity_re = BerEncoder()
        _entity_re.write_tag(_ft[0], _ft[1], _ft[2])
        _entity_re.write_length(_fl)
        _entity_re.write_bytes(_fd)
        _entity = Entity.decode_ber(_entity_re.finish())




        return cls(

            id=_id,

            entity=_entity

        )


@dataclass
class MixedChoice(AsnType):


    item: Optional[Person] = None

    count: Optional[int] = None

    label: Optional[str] = None


    def encode_ber(self) -> bytes:




        if self.item is not None:
            _inner = self.item.encode_ber()
            _e = BerEncoder()
            _e.write_tag(TagClass.CONTEXT, 0, True)
            _e.write_length(len(_inner))
            _e.write_bytes(_inner)
            return _e.finish()







        if self.count is not None:
            _iv = BerEncoder()
            _iv.write_integer(self.count)
            _ib = _iv.finish()
            _ie = BerEncoder()
            _ie.write_tag(TagClass.UNIVERSAL, 2, False)
            _ie.write_length(len(_ib))
            _ie.write_bytes(_ib)
            return _ie.finish()







        if self.label is not None:
            _se = BerEncoder()
            _sb = self.label.encode('utf-8')
            _se.write_tag(TagClass.UNIVERSAL, 12, False)
            _se.write_length(len(_sb))
            _se.write_bytes(_sb)
            return _se.finish()




        raise ValueError("No choice alternative set")

    @classmethod
    def decode_ber(cls, data: bytes) -> "MixedChoice":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)




        if _tag[0] == TagClass.CONTEXT and _tag[1] == 0:
            return cls(item=Person.decode_ber(_fd))







        if _tag[0] == TagClass.UNIVERSAL and _tag[1] == 2 and not _tag[2]:
            return cls(count=int.from_bytes(_fd, byteorder='big', signed=True))







        if _tag[0] == TagClass.UNIVERSAL and _tag[1] == 12 and not _tag[2]:
            return cls(label=_fd.decode('utf-8'))











        raise ValueError(f"Unknown choice tag: {_tag}")

    def encode_ber_indefinite(self) -> bytes:




        if self.item is not None:
            _inner = self.item.encode_ber()
            _e = BerEncoder()
            _e.write_tag(TagClass.CONTEXT, 0, True)
            _e.write_length(0, indefinite=True)
            _e.write_bytes(_inner)
            _e.write_eoc()
            return _e.finish()







        if self.count is not None:
            _e = BerEncoder()

            _iv = BerEncoder()
            _iv.write_integer(self.count)
            _ib = _iv.finish()
            _e.write_tag(TagClass.UNIVERSAL, 2, False)
            _e.write_length(0, indefinite=True)
            _e.write_bytes(_ib)
            _e.write_eoc()

            return _e.finish()







        if self.label is not None:
            _e = BerEncoder()

            _sb = self.label.encode('utf-8')
            _e.write_tag(TagClass.UNIVERSAL, 12, False)
            _e.write_length(0, indefinite=True)
            _e.write_bytes(_sb)
            _e.write_eoc()

            return _e.finish()




        raise ValueError("No choice alternative set")

    @classmethod
    def decode_ber_indefinite(cls, data: bytes) -> "MixedChoice":
        decoder = BerDecoder(data)
        _tag = decoder.read_tag()
        _length = decoder.read_length()
        if _length is None:
            _content = decoder.read_constructed_indefinite()
        else:
            _content = decoder.read_bytes(_length)



        if _tag[0] == TagClass.CONTEXT and _tag[1] == 0:
            return cls(item=Person.decode_der(_content))





        if _tag[0] == TagClass.UNIVERSAL and _tag[1] == 2:
            return cls(count=int.from_bytes(_content, byteorder='big', signed=True))





        if _tag[0] == TagClass.UNIVERSAL and _tag[1] == 12:
            return cls(label=_content.decode('utf-8'))










        raise ValueError(f"Unknown choice tag: {_tag}")

    def encode_der(self) -> bytes:




        if self.item is not None:
            _inner = self.item.encode_der()
            _e = DerEncoder()
            _e.write_tag(TagClass.CONTEXT, 0, True)
            _e.write_length(len(_inner))
            _e.write_bytes(_inner)
            return _e.finish()







        if self.count is not None:
            _iv = DerEncoder()
            _iv.write_integer(self.count)
            _ib = _iv.finish()
            _ie = DerEncoder()
            _ie.write_tag(TagClass.UNIVERSAL, 2, False)
            _ie.write_length(len(_ib))
            _ie.write_bytes(_ib)
            return _ie.finish()







        if self.label is not None:
            _se = DerEncoder()
            _sb = self.label.encode('utf-8')
            _se.write_tag(TagClass.UNIVERSAL, 12, False)
            _se.write_length(len(_sb))
            _se.write_bytes(_sb)
            return _se.finish()




        raise ValueError("No choice alternative set")

    @classmethod
    def decode_der(cls, data: bytes) -> "MixedChoice":
        decoder = DerDecoder(data)
        _tag = decoder.read_tag()
        _fl = decoder.read_length()
        _fd = decoder.read_bytes(_fl)




        if _tag[0] == TagClass.CONTEXT and _tag[1] == 0:
            return cls(item=Person.decode_der(_fd))







        if _tag[0] == TagClass.UNIVERSAL and _tag[1] == 2 and not _tag[2]:
            return cls(count=int.from_bytes(_fd, byteorder='big', signed=True))







        if _tag[0] == TagClass.UNIVERSAL and _tag[1] == 12 and not _tag[2]:
            return cls(label=_fd.decode('utf-8'))











        raise ValueError(f"Unknown choice tag: {_tag}")