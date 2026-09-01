use crate::class_file_version::VersionError;

/// Not sure if this is right, but this is a catch-all for all parsing errors.
/// I ended up with some duplication of the ParseError type before this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedEof { offset: usize },
    InvalidMagic { found: u32 },
    InvalidVersion(VersionError),
    InvalidConstantPoolTag { offset: usize, tag: u8 },
    InvalidConstantPoolLayout { offset: usize },

    InvalidOpcode { opcode: u8 },
}

pub struct ClassReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

///
/// A simple byte reader that allows us to satisfy the following defintions from the class file documentation:
/// > A class file consists of a stream of 8-bit bytes. 16-bit and 32-bit quantities are constructed by reading in two and four consecutive 8-bit bytes
///
/// Takes an array of bytes and allows traversal of them. Uses an offset to represent progress through the array of bytes.
impl<'a> ClassReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn read_u8(&mut self) -> Result<u8, ParseError> {
        Ok(self.read_bytes(1)?[0])
    }

    pub fn read_u16(&mut self) -> Result<u16, ParseError> {
        let bytes: [u8; 2] = self
            .read_bytes(2)?
            .try_into()
            .expect("read_bytes guaranteed length of 2");

        Ok(u16::from_be_bytes(bytes))
    }

    pub fn read_u32(&mut self) -> Result<u32, ParseError> {
        let bytes: [u8; 4] = self
            .read_bytes(4)?
            .try_into()
            .expect("read_bytes guaranteed length of 4");
        Ok(u32::from_be_bytes(bytes))
    }

    pub fn read_u64(&mut self) -> Result<u64, ParseError> {
        let bytes: [u8; 8] = self
            .read_bytes(8)?
            .try_into()
            .expect("read_bytes guaranteed length of 8");
        Ok(u64::from_be_bytes(bytes))
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], ParseError> {
        let start = self.offset;

        let end = start
            .checked_add(len)
            .ok_or(ParseError::UnexpectedEof { offset: start })?;

        let bytes = self
            .bytes
            .get(start..end)
            .ok_or(ParseError::UnexpectedEof { offset: start })?;

        self.offset = end;

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_offset_zero() {
        let reader = ClassReader::new(&[1, 2, 3]);

        assert_eq!(reader.offset(), 0);
    }

    #[test]
    fn reads_u8_and_advances_offset() {
        let mut reader = ClassReader::new(&[0xAB, 0xCD]);

        let value = reader.read_u8().unwrap();

        assert_eq!(value, 0xAB);
        assert_eq!(reader.offset(), 1);
    }

    #[test]
    fn reads_u16_as_big_endian() {
        let mut reader = ClassReader::new(&[0x12, 0x34]);

        let value = reader.read_u16().unwrap();

        assert_eq!(value, 0x1234);
        assert_eq!(reader.offset(), 2);
    }

    #[test]
    fn reads_u32_as_big_endian() {
        let mut reader = ClassReader::new(&[0xCA, 0xFE, 0xBA, 0xBE]);

        let value = reader.read_u32().unwrap();

        assert_eq!(value, 0xCAFEBABE);
        assert_eq!(reader.offset(), 4);
    }

    #[test]
    fn reads_u64_as_big_endian() {
        let mut reader = ClassReader::new(&[0xCA, 0xFE, 0xBA, 0xBE, 0xAA, 0xBB, 0xCC, 0xDD]);

        let value = reader.read_u64().unwrap();

        assert_eq!(value, 0xCAFEBABEAABBCCDD);
        assert_eq!(reader.offset(), 8);
    }

    #[test]
    fn reads_multiple_values_sequentially() {
        let bytes = [0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x3D];

        let mut reader = ClassReader::new(&bytes);

        let magic = reader.read_u32().unwrap();
        let minor = reader.read_u16().unwrap();
        let major = reader.read_u16().unwrap();

        assert_eq!(magic, 0xCAFEBABE);
        assert_eq!(minor, 0);
        assert_eq!(major, 61);
        assert_eq!(reader.offset(), 8);
    }

    #[test]
    fn read_bytes_returns_requested_slice() {
        let bytes = [1, 2, 3, 4, 5];
        let mut reader = ClassReader::new(&bytes);

        let result = reader.read_bytes(3).unwrap();

        assert_eq!(result, &[1, 2, 3]);
        assert_eq!(reader.offset(), 3);
    }

    #[test]
    fn read_bytes_can_read_remaining_bytes() {
        let bytes = [1, 2, 3, 4];
        let mut reader = ClassReader::new(&bytes);

        reader.read_u8().unwrap();

        let remaining = reader.read_bytes(3).unwrap();

        assert_eq!(remaining, &[2, 3, 4]);
        assert_eq!(reader.offset(), 4);
    }

    #[test]
    fn read_u8_returns_eof_on_empty_input() {
        let mut reader = ClassReader::new(&[]);

        let result = reader.read_u8();

        assert_eq!(result, Err(ParseError::UnexpectedEof { offset: 0 }));
    }

    #[test]
    fn read_u16_returns_eof_when_only_one_byte_remains() {
        let mut reader = ClassReader::new(&[0x12]);

        let result = reader.read_u16();

        assert_eq!(result, Err(ParseError::UnexpectedEof { offset: 0 }));
    }

    #[test]
    fn read_u32_returns_eof_when_input_is_too_short() {
        let mut reader = ClassReader::new(&[0xCA, 0xFE, 0xBA]);

        let result = reader.read_u32();

        assert_eq!(result, Err(ParseError::UnexpectedEof { offset: 0 }));
    }

    #[test]
    fn eof_reports_current_offset() {
        let bytes = [0x12, 0x34, 0x56];
        let mut reader = ClassReader::new(&bytes);

        assert_eq!(reader.read_u16().unwrap(), 0x1234);

        let result = reader.read_u16();

        assert_eq!(result, Err(ParseError::UnexpectedEof { offset: 2 }));
    }

    #[test]
    fn failed_read_does_not_advance_offset() {
        let mut reader = ClassReader::new(&[0x12, 0x34, 0x56]);

        reader.read_u16().unwrap();

        assert_eq!(reader.offset(), 2);

        let result = reader.read_u32();

        assert!(result.is_err());
        assert_eq!(reader.offset(), 2);
    }

    #[test]
    fn reading_exactly_to_end_succeeds() {
        let mut reader = ClassReader::new(&[0x12, 0x34]);

        assert_eq!(reader.read_u16().unwrap(), 0x1234);
        assert_eq!(reader.offset(), 2);
    }

    #[test]
    fn next_read_after_end_returns_eof() {
        let mut reader = ClassReader::new(&[0x12, 0x34]);

        reader.read_u16().unwrap();

        let result = reader.read_u8();

        assert_eq!(result, Err(ParseError::UnexpectedEof { offset: 2 }));
    }

    #[test]
    fn read_zero_bytes_does_not_advance() {
        let bytes = [1, 2, 3];
        let mut reader = ClassReader::new(&bytes);

        let result = reader.read_bytes(0).unwrap();

        assert!(result.is_empty());
        assert_eq!(reader.offset(), 0);
    }
}
