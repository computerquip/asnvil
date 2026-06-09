use crate::errors::AsnError;
use crate::types::TagClass;
use num_bigint::BigInt;

/// OER (Octet Encoding Rules) encoder (Basic/Unaligned).
pub struct OerEncoder {
    buf: Vec<u8>,
}

impl OerEncoder {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn write_tag(&mut self, tag_class: TagClass, number: u32, constructed: bool) -> Result<(), AsnError> {
        if number < 31 {
            let mut tag_byte = ((tag_class as u8) << 6) | (number as u8 & 0x1F);
            if constructed {
                tag_byte |= 0x20;
            }
            self.buf.push(tag_byte);
        } else {
            let mut tag_byte = ((tag_class as u8) << 6) | 0x1F;
            if constructed {
                tag_byte |= 0x20;
            }
            self.buf.push(tag_byte);
            let mut num_bytes = Vec::new();
            let mut num = number;
            while num > 0 {
                num_bytes.insert(0, (num & 0x7F) as u8);
                num >>= 7;
            }
            for i in 0..num_bytes.len() {
                if i < num_bytes.len() - 1 {
                    num_bytes[i] |= 0x80;
                }
            }
            self.buf.extend(num_bytes);
        }
        Ok(())
    }

    pub fn write_length(&mut self, length: usize) -> Result<(), AsnError> {
        if length < 128 {
            self.buf.push(length as u8);
        } else {
            let first_byte = 128 + ((length >> 8) as u8);
            let second_byte = (length & 0xFF) as u8;
            self.buf.push(first_byte);
            self.buf.push(second_byte);
        }
        Ok(())
    }

    pub fn write_integer(&mut self, value: &BigInt) -> Result<(), AsnError> {
        if value == &BigInt::from(0) {
            self.buf.push(0x00);
            return Ok(());
        }

        let (sign, mut bytes) = value.to_bytes_be();
        if sign == num_bigint::Sign::Minus {
            for b in &mut bytes {
                *b = !*b;
            }
            let mut carry = 1;
            for b in bytes.iter_mut().rev() {
                let (new_b, c) = b.overflowing_add(carry);
                *b = new_b;
                carry = if c { 1 } else { 0 };
                if carry == 0 {
                    break;
                }
            }
            if bytes.first().map_or(true, |&b| (b & 0x80) == 0) {
                bytes.insert(0, 0xFF);
            }
        } else {
            if bytes.first().map_or(true, |&b| (b & 0x80) != 0) {
                bytes.insert(0, 0x00);
            }
        }

        if bytes.len() > 1 {
            let first = bytes[0];
            let second = bytes[1];
            if first == 0x00 && (second & 0x80) == 0 {
                return Err(AsnError::InvalidIntegerEncoding);
            }
            if first == 0xFF && (second & 0x80) != 0 {
                return Err(AsnError::InvalidIntegerEncoding);
            }
        }

        self.buf.extend(bytes);
        Ok(())
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        self.buf.extend(data);
    }

    pub fn write_boolean(&mut self, value: bool) {
        self.buf.push(if value { 0xFF } else { 0x00 });
    }

    pub fn write_tlv(&mut self, tag_class: TagClass, number: u32, content: &[u8], constructed: bool) -> Result<(), AsnError> {
        self.write_tag(tag_class, number, constructed)?;
        self.write_length(content.len())?;
        self.write_bytes(content);
        Ok(())
    }

    pub fn finish(self) -> Vec<u8> {
        self.buf
    }
}

/// OER (Octet Encoding Rules) decoder (Basic/Unaligned).
pub struct OerDecoder<'a> {
    pub data: &'a [u8],
    pub pos: usize,
}

impl<'a> OerDecoder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn read_tag(&mut self) -> Result<(TagClass, u32, bool), AsnError> {
        if self.pos >= self.data.len() {
            return Err(AsnError::TruncatedInput);
        }
        let tag_byte = self.data[self.pos];
        self.pos += 1;
        let tag_class = TagClass::from((tag_byte >> 6) & 0x03);
        let constructed = (tag_byte & 0x20) != 0;
        let mut number = (tag_byte & 0x1F) as u32;

        if number == 0x1F {
            number = 0;
            loop {
                if self.pos >= self.data.len() {
                    return Err(AsnError::TruncatedInput);
                }
                let byte = self.data[self.pos];
                self.pos += 1;
                number = (number << 7) | ((byte & 0x7F) as u32);
                if (byte & 0x80) == 0 {
                    break;
                }
            }
        }

        Ok((tag_class, number, constructed))
    }

    pub fn read_length(&mut self) -> Result<usize, AsnError> {
        if self.pos >= self.data.len() {
            return Err(AsnError::TruncatedInput);
        }
        let first = self.data[self.pos];
        self.pos += 1;

        if first < 128 {
            Ok(first as usize)
        } else {
            if self.pos >= self.data.len() {
                return Err(AsnError::TruncatedInput);
            }
            let second = self.data[self.pos];
            self.pos += 1;
            Ok((((first - 128) as usize) << 8) | (second as usize))
        }
    }

    pub fn read_integer(&mut self) -> Result<BigInt, AsnError> {
        let length = self.read_length()?;
        if length == 0 {
            return Ok(BigInt::from(0));
        }
        if self.pos + length > self.data.len() {
            return Err(AsnError::TruncatedInput);
        }

        if length > 1 {
            let first = self.data[self.pos];
            let second = self.data[self.pos + 1];
            if first == 0x00 && (second & 0x80) == 0 {
                return Err(AsnError::InvalidIntegerEncoding);
            }
            if first == 0xFF && (second & 0x80) != 0 {
                return Err(AsnError::InvalidIntegerEncoding);
            }
        }

        let bytes = &self.data[self.pos..self.pos + length];
        self.pos += length;

        if bytes[0] & 0x80 != 0 {
            let mut val = BigInt::from(0);
            for &b in bytes {
                val = (val << 8) | BigInt::from(b);
            }
            let shift = BigInt::from(1) << (length * 8);
            Ok(val - shift)
        } else {
            Ok(BigInt::from_bytes_be(num_bigint::Sign::Plus, bytes))
        }
    }

    pub fn read_bytes(&mut self, length: usize) -> Result<&'a [u8], AsnError> {
        if self.pos + length > self.data.len() {
            return Err(AsnError::TruncatedInput);
        }
        let result = &self.data[self.pos..self.pos + length];
        self.pos += length;
        Ok(result)
    }

    pub fn read_boolean(&mut self) -> Result<bool, AsnError> {
        let length = self.read_length()?;
        if length != 1 {
            return Err(AsnError::InvalidIntegerEncoding);
        }
        let byte = self.data[self.pos];
        self.pos += 1;
        if byte != 0x00 && byte != 0xFF {
            return Err(AsnError::InvalidIntegerEncoding);
        }
        Ok(byte != 0x00)
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    pub fn rewind_to(&mut self, pos: usize) {
        self.pos = pos;
    }
}
