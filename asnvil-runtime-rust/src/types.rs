use crate::errors::AsnError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagClass {
    Universal = 0,
    Application = 1,
    Context = 2,
    Private = 3,
}

impl From<u8> for TagClass {
    fn from(value: u8) -> Self {
        match value {
            0 => TagClass::Universal,
            1 => TagClass::Application,
            2 => TagClass::Context,
            3 => TagClass::Private,
            _ => panic!("Invalid tag class"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub tag_class: TagClass,
    pub number: u32,
    pub constructed: bool,
}

impl Tag {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        if self.number < 31 {
            let mut tag_byte = ((self.tag_class as u8) << 6) | (self.number as u8 & 0x1F);
            if self.constructed {
                tag_byte |= 0x20;
            }
            buf.push(tag_byte);
        } else {
            let mut tag_byte = ((self.tag_class as u8) << 6) | 0x1F;
            if self.constructed {
                tag_byte |= 0x20;
            }
            buf.push(tag_byte);
            let mut num_bytes = Vec::new();
            let mut num = self.number;
            while num > 0 {
                num_bytes.insert(0, (num & 0x7F) as u8);
                num >>= 7;
            }
            for i in 0..num_bytes.len() {
                if i < num_bytes.len() - 1 {
                    num_bytes[i] |= 0x80;
                }
            }
            buf.extend(num_bytes);
        }
        buf
    }

    pub fn decode(data: &[u8]) -> Result<(Self, usize), AsnError> {
        if data.is_empty() {
            return Err(AsnError::TruncatedInput);
        }
        let tag_byte = data[0];
        let tag_class = TagClass::from((tag_byte >> 6) & 0x03);
        let constructed = (tag_byte & 0x20) != 0;
        let mut number = (tag_byte & 0x1F) as u32;
        let mut pos = 1;
        let mut long_form = false;

        if number == 0x1F {
            long_form = true;
            number = 0;
            loop {
                if pos >= data.len() {
                    return Err(AsnError::TruncatedInput);
                }
                let byte = data[pos];
                pos += 1;
                number = (number << 7) | ((byte & 0x7F) as u32);
                if (byte & 0x80) == 0 {
                    break;
                }
            }
        }

        if long_form && number < 31 {
            return Err(AsnError::InvalidTag);
        }

        Ok((
            Self {
                tag_class,
                number,
                constructed,
            },
            pos,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitString {
    data: Vec<u8>,
    unused_bits: u8,
}

impl BitString {
    pub fn new(data: Vec<u8>, unused_bits: u8) -> Result<Self, AsnError> {
        if unused_bits > 7 {
            return Err(AsnError::ConstraintViolation("unused_bits must be 0-7".into()));
        }
        if unused_bits > 0 && !data.is_empty() {
            if data.last().unwrap() & ((1 << unused_bits) - 1) != 0 {
                return Err(AsnError::ConstraintViolation("Unused bits must be zero".into()));
            }
        }
        Ok(Self { data, unused_bits })
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn unused_bits(&self) -> u8 {
        self.unused_bits
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectIdentifier {
    components: Vec<u32>,
}

impl ObjectIdentifier {
    pub fn new(components: Vec<u32>) -> Result<Self, AsnError> {
        if components.len() < 2 {
            return Err(AsnError::ConstraintViolation("OID must have at least 2 components".into()));
        }
        Ok(Self { components })
    }

    pub fn components(&self) -> &[u32] {
        &self.components
    }

    pub fn encode(&self) -> Result<Vec<u8>, AsnError> {
        let mut result = Vec::new();
        let first = self.components[0];
        let second = self.components[1];
        result.push((first * 40 + second) as u8);

        for &component in &self.components[2..] {
            let mut num_bytes = Vec::new();
            let mut comp = component;
            while comp > 0 {
                num_bytes.insert(0, (comp & 0x7F) as u8);
                comp >>= 7;
            }
            if num_bytes.is_empty() {
                num_bytes = vec![0];
            }
            for i in 0..num_bytes.len() - 1 {
                num_bytes[i] |= 0x80;
            }
            result.extend(num_bytes);
        }
        Ok(result)
    }

    pub fn decode(data: &[u8]) -> Result<(Self, usize), AsnError> {
        if data.is_empty() {
            return Err(AsnError::TruncatedInput);
        }
        let first = data[0];
        let mut components = vec![(first / 40) as u32, (first % 40) as u32];
        let mut pos = 1;

        while pos < data.len() {
            let mut component = 0u32;
            loop {
                if pos >= data.len() {
                    return Err(AsnError::TruncatedInput);
                }
                let byte = data[pos];
                pos += 1;
                component = (component << 7) | ((byte & 0x7F) as u32);
                if (byte & 0x80) == 0 {
                    break;
                }
            }
            components.push(component);
        }
        Ok((Self { components }, pos))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsnAny {
    pub tag_class: TagClass,
    pub number: u32,
    pub content: Vec<u8>,
}
