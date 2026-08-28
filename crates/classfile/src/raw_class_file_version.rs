use crate::class_reader::{ClassReader, ParseError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RawClassFileVersion {
    pub major: u16,
    pub minor: u16,
}

impl RawClassFileVersion {
    pub fn parse(reader: &mut ClassReader<'_>) -> Result<Self, ParseError> {
        let magic = reader.read_u32()?;

        if magic != 0xCAFEBABE {
            return Err(ParseError::InvalidMagic { found: magic });
        }

        let minor = reader.read_u16()?;
        let major = reader.read_u16()?;

        Ok(Self { major, minor })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class_header(magic: u32, minor: u16, major: u16) -> [u8; 8] {
        let magic = magic.to_be_bytes();
        let minor = minor.to_be_bytes();
        let major = major.to_be_bytes();

        [
            magic[0], magic[1], magic[2], magic[3], minor[0], minor[1], major[0], major[1],
        ]
    }

    #[test]
    fn parses_java_8_version() {
        let bytes = class_header(0xCAFEBABE, 0, 52);
        let mut reader = ClassReader::new(&bytes);

        let version = RawClassFileVersion::parse(&mut reader).expect("valid class file header");

        assert_eq!(
            version,
            RawClassFileVersion {
                major: 52,
                minor: 0,
            }
        );
    }

    #[test]
    fn parses_java_21_version() {
        let bytes = class_header(0xCAFEBABE, 0, 65);
        let mut reader = ClassReader::new(&bytes);

        let version = RawClassFileVersion::parse(&mut reader).expect("valid class file header");

        assert_eq!(version.major, 65);
        assert_eq!(version.minor, 0);
    }

    #[test]
    fn parses_preview_minor_version() {
        let bytes = class_header(0xCAFEBABE, u16::MAX, 65);
        let mut reader = ClassReader::new(&bytes);

        let version = RawClassFileVersion::parse(&mut reader).expect("valid class file header");

        assert_eq!(version.major, 65);
        assert_eq!(version.minor, u16::MAX);
    }

    #[test]
    fn preserves_arbitrary_raw_minor_version() {
        let bytes = class_header(0xCAFEBABE, 123, 52);
        let mut reader = ClassReader::new(&bytes);

        let version =
            RawClassFileVersion::parse(&mut reader).expect("raw parser should preserve minor");

        assert_eq!(version.major, 52);
        assert_eq!(version.minor, 123);
    }

    #[test]
    fn rejects_invalid_magic() {
        let bytes = class_header(0xBABECAFE, 0, 65);

        let mut reader = ClassReader::new(&bytes);

        let result = RawClassFileVersion::parse(&mut reader);

        assert_eq!(result, Err(ParseError::InvalidMagic { found: 0xBABECAFE }));
    }

    #[test]
    fn rejects_empty_input() {
        let mut reader = ClassReader::new(&[]);
        let result = RawClassFileVersion::parse(&mut reader);

        assert_eq!(result, Err(ParseError::UnexpectedEof { offset: 0 }));
    }

    #[test]
    fn rejects_input_shorter_than_header() {
        let bytes = [0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00];

        let mut reader = ClassReader::new(&bytes);

        let result = RawClassFileVersion::parse(&mut reader);

        assert_eq!(result, Err(ParseError::UnexpectedEof { offset: 6 }));
    }

    #[test]
    fn accepts_exactly_eight_bytes() {
        let bytes = class_header(0xCAFEBABE, 0, 61);

        let mut reader = ClassReader::new(&bytes);

        let result = RawClassFileVersion::parse(&mut reader);

        assert!(result.is_ok());
    }

    #[test]
    fn ignores_bytes_after_header() {
        let mut bytes = class_header(0xCAFEBABE, 0, 61).to_vec();

        bytes.extend_from_slice(&[0x00, 0x42, 0xDE, 0xAD, 0xBE, 0xEF]);

        let mut reader = ClassReader::new(&bytes);

        let version = RawClassFileVersion::parse(&mut reader).expect("valid class file prefix");

        assert_eq!(version.major, 61);
        assert_eq!(version.minor, 0);
    }

    #[test]
    fn reads_values_as_big_endian() {
        let bytes = [0xCA, 0xFE, 0xBA, 0xBE, 0x12, 0x34, 0x56, 0x78];

        let mut reader = ClassReader::new(&bytes);

        let version = RawClassFileVersion::parse(&mut reader).expect("valid header");

        assert_eq!(version.minor, 0x1234);
        assert_eq!(version.major, 0x5678);
    }
}
