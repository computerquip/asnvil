use crate::errors::AsnError;
use crate::types::TagClass;

pub struct BerEncoder {
    buf: Vec<u8>,
}

impl BerEncoder {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn write_tag(&mut self, tag_class: TagClass, number: u32, constructed: bool) {
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
    }

    pub fn write_length(&mut self, length: usize, indefinite: bool) {
        if indefinite {
            self.buf.push(0x80);
        } else if length <= 127 {
            self.buf.push(length as u8);
        } else {
            let mut num_bytes = Vec::new();
            let mut len = length;
            while len > 0 {
                num_bytes.insert(0, (len & 0xFF) as u8);
                len >>= 8;
            }
            self.buf.push(0x80 | (num_bytes.len() as u8));
            self.buf.extend(num_bytes);
        }
    }

    pub fn write_integer(&mut self, value: &num_bigint::BigInt) {
        if value == &num_bigint::BigInt::from(0) {
            self.buf.push(0x00);
            return;
        }

        let (sign, mut bytes) = value.to_bytes_be();
        if sign == num_bigint::Sign::Minus {
            // Two's complement for negative numbers
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
        self.buf.extend(bytes);
    }

    pub fn write_bytes(&mut self, data: &[u8]) {
        self.buf.extend(data);
    }

    pub fn write_eoc(&mut self) {
        self.buf.push(0x00);
        self.buf.push(0x00);
    }

    pub fn write_tlv(&mut self, tag_class: TagClass, number: u32, content: &[u8], constructed: bool) {
        self.write_tag(tag_class, number, constructed);
        self.write_length(content.len(), false);
        self.write_bytes(content);
    }

    pub fn write_tlv_indefinite(&mut self, tag_class: TagClass, number: u32, content: &[u8], constructed: bool) {
        self.write_tag(tag_class, number, constructed);
        self.write_length(0, true);
        self.write_bytes(content);
        self.write_eoc();
    }

    pub fn finish(self) -> Vec<u8> {
        self.buf
    }
}

pub struct BerDecoder<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> BerDecoder<'a> {
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

    pub fn read_length(&mut self) -> Result<Option<usize>, AsnError> {
        if self.pos >= self.data.len() {
            return Err(AsnError::TruncatedInput);
        }
        let first = self.data[self.pos];
        self.pos += 1;

        if first == 0x80 {
            Ok(None) // Indefinite length
        } else if first <= 0x7F {
            Ok(Some(first as usize))
        } else {
            let num_bytes = (first & 0x7F) as usize;
            if self.pos + num_bytes > self.data.len() {
                return Err(AsnError::TruncatedInput);
            }
            let mut length = 0;
            for _ in 0..num_bytes {
                length = (length << 8) | (self.data[self.pos] as usize);
                self.pos += 1;
            }
            Ok(Some(length))
        }
    }

    pub fn read_integer(&mut self) -> Result<num_bigint::BigInt, AsnError> {
        let length = self.read_length()?.ok_or(AsnError::InvalidLength)?;
        if length == 0 {
            return Ok(num_bigint::BigInt::from(0));
        }
        if self.pos + length > self.data.len() {
            return Err(AsnError::TruncatedInput);
        }

        let bytes = &self.data[self.pos..self.pos + length];
        self.pos += length;

        // Check if negative (highest bit set)
        if bytes[0] & 0x80 != 0 {
            // Two's complement
            let mut val = num_bigint::BigInt::from(0);
            for &b in bytes {
                val = (val << 8) | num_bigint::BigInt::from(b);
            }
            // Subtract 2^(length * 8)
            let shift = num_bigint::BigInt::from(1) << (length * 8);
            Ok(val - shift)
        } else {
            Ok(num_bigint::BigInt::from_bytes_be(num_bigint::Sign::Plus, bytes))
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

    pub fn read_tlv(&mut self) -> Result<((TagClass, u32, bool), Option<&'a [u8]>), AsnError> {
        let tag = self.read_tag()?;
        let length = self.read_length()?;
        if length.is_none() {
            return Ok((tag, None)); // Indefinite length
        }
        let length = length.unwrap();
        if self.pos + length > self.data.len() {
            return Err(AsnError::TruncatedInput);
        }
        let content = &self.data[self.pos..self.pos + length];
        self.pos += length;
        Ok((tag, Some(content)))
    }

    pub fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    pub fn is_eoc(&self) -> bool {
        self.pos + 1 < self.data.len() && self.data[self.pos] == 0x00 && self.data[self.pos + 1] == 0x00
    }

    pub fn read_eoc(&mut self) -> Result<(), AsnError> {
        if !self.is_eoc() {
            return Err(AsnError::TruncatedInput);
        }
        self.pos += 2;
        Ok(())
    }

    pub fn read_constructed_indefinite(&mut self) -> Result<Vec<u8>, AsnError> {
        let mut content = Vec::new();
        while !self.is_eoc() {
            let tag = self.read_tag()?;
            let length = self.read_length()?;
            if length.is_none() {
                let inner = self.read_constructed_indefinite()?;
                let mut inner_e = BerEncoder::new();
                inner_e.write_tag(tag.0, tag.1, tag.2);
                inner_e.write_length(0, true);
                inner_e.write_bytes(&inner);
                inner_e.write_eoc();
                content.extend(inner_e.finish());
            } else {
                let length = length.unwrap();
                let value = self.read_bytes(length)?;
                let mut elem_e = BerEncoder::new();
                elem_e.write_tag(tag.0, tag.1, tag.2);
                elem_e.write_length(value.len(), false);
                elem_e.write_bytes(value);
                content.extend(elem_e.finish());
            }
        }
        self.read_eoc()?;
        Ok(content)
    }

    pub fn at_end(&self) -> bool {
        self.pos >= self.data.len()
    }
}
