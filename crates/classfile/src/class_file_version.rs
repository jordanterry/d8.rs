use crate::raw_class_file_version::RawClassFileVersion;
use std::fmt;
use std::fmt::Formatter;

/// Compact representation of a .class file's versioning, held within the 8 bits of a u8.
///
/// Why am I compacting this? Well, I know that d8 compilation does not care so much about
/// the minor version of a class file. I am happy to discard this knowledge and use a single
/// bit to represent the possibility of it being a preview version.
///
/// The 8-bits are used as follows:
///
/// - Highest bit, 1 or 0 depending on whether a minor version existed.
/// - Lowest 7 bits, populated with the major version number.
///
/// Is this overdoing it? Maybe, but I do want to chase the concept of data-oriented design, and
/// the pursuit of keeping as much as possible in a single cache line. Who knows, I might get to
/// R8 one day and see some real benefits.
///
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ClassFileVersion(u8);

impl ClassFileVersion {
    /// mask for the highest bit
    const MINOR_BIT: u8 = 0b1000_0000;
    /// mask for the lower seven bits
    const MAJOR_BITS: u8 = 0b0111_1111;

    pub const fn major(self) -> u8 {
        self.0 & Self::MAJOR_BITS
    }

    pub const fn preview(self) -> bool {
        (self.0 & Self::MINOR_BIT) != 0
    }

    pub const fn java_version(self) -> Option<u8> {
        let major = self.major();
        if major >= 46 {
            // Account for the pre-Java 1.0 days of Oak.
            // Roughly, 44 versions were released before 1.0.
            // class version 55 is Java 11.
            Some(major - 44)
        } else {
            None
        }
    }

    const fn new_unchecked(major: u8, preview: bool) -> Self {
        let preview_bit = if preview { Self::MINOR_BIT } else { 0 };
        Self(major | preview_bit)
    }
}

impl TryFrom<RawClassFileVersion> for ClassFileVersion {
    type Error = VersionError;

    fn try_from(value: RawClassFileVersion) -> Result<Self, Self::Error> {
        let major = u8::try_from(value.major)
            .map_err(|_| VersionError::InvalidMajor { major: value.major })?;

        let preview = match value.minor {
            0 => false,
            u16::MAX => true,
            _ => {
                return Err(VersionError::InvalidMinor {
                    major: value.major,
                    minor: value.minor,
                });
            }
        };
        // Arbitrarily supporting Java 12 right now.
        // Supporting early dex functionality might be fun, but this is a side project, lets be honest,
        // it won't get done.
        if major < 56 && preview {
            return Err(VersionError::InvalidMajor { major: value.major });
        }

        Ok(Self::new_unchecked(major, preview))
    }
}

impl fmt::Display for ClassFileVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match (self.java_version(), self.preview()) {
            (Some(release), true) => {
                write!(f, "Java {release} preview (class file {})", self.major())
            }
            (Some(release), false) => {
                write!(f, "Java {release} (class file {})", self.major())
            }
            (None, _) => {
                write!(f, "class file version {}", self.major())
            }
        }
    }
}

impl fmt::Debug for ClassFileVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClassFileVersion")
            .field("major", &self.major())
            .field("java_version", &self.java_version())
            .field("preview", &self.preview())
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionError {
    InvalidMajor { major: u16 },
    InvalidMinor { major: u16, minor: u16 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compacts_java_8() {
        let raw = RawClassFileVersion {
            major: 52,
            minor: 0,
        };

        let version = ClassFileVersion::try_from(raw).expect("valid Java 8 version");

        assert_eq!(version.major(), 52);
        assert_eq!(version.java_version(), Some(8));
        assert!(!version.preview());
    }

    #[test]
    fn compacts_java_9() {
        let raw = RawClassFileVersion {
            major: 53,
            minor: 0,
        };

        let version = ClassFileVersion::try_from(raw).expect("valid Java 9 version");

        assert_eq!(version.major(), 53);
        assert_eq!(version.java_version(), Some(9));
        assert!(!version.preview());
    }

    #[test]
    fn compacts_java_10() {
        let raw = RawClassFileVersion {
            major: 54,
            minor: 0,
        };

        let version = ClassFileVersion::try_from(raw).expect("valid Java 10 version");

        assert_eq!(version.major(), 54);
        assert_eq!(version.java_version(), Some(10));
        assert!(!version.preview());
    }

    #[test]
    fn compacts_java_11() {
        let raw = RawClassFileVersion {
            major: 55,
            minor: 0,
        };

        let version = ClassFileVersion::try_from(raw).expect("valid Java 11 version");

        assert_eq!(version.major(), 55);
        assert_eq!(version.java_version(), Some(11));
        assert!(!version.preview());
    }

    #[test]
    fn compacts_java_12() {
        let raw = RawClassFileVersion {
            major: 56,
            minor: 0,
        };

        let version = ClassFileVersion::try_from(raw).expect("valid Java 12 version");

        assert_eq!(version.major(), 56);
        assert_eq!(version.java_version(), Some(12));
        assert!(!version.preview());
    }

    #[test]
    fn compacts_java_17() {
        let raw = RawClassFileVersion {
            major: 61,
            minor: 0,
        };

        let version = ClassFileVersion::try_from(raw).expect("valid Java 17 version");

        assert_eq!(version.major(), 61);
        assert_eq!(version.java_version(), Some(17));
        assert!(!version.preview());
    }

    #[test]
    fn compacts_java_21() {
        let raw = RawClassFileVersion {
            major: 65,
            minor: 0,
        };

        let version = ClassFileVersion::try_from(raw).expect("valid Java 21 version");

        assert_eq!(version.major(), 65);
        assert_eq!(version.java_version(), Some(21));
        assert!(!version.preview());
    }

    #[test]
    fn compacts_java_26() {
        let raw = RawClassFileVersion {
            major: 70,
            minor: 0,
        };

        let version = ClassFileVersion::try_from(raw).expect("valid Java 26 version");

        assert_eq!(version.major(), 70);
        assert_eq!(version.java_version(), Some(26));
        assert!(!version.preview());
    }

    #[test]
    fn compacts_preview_version() {
        let raw = RawClassFileVersion {
            major: 65,
            minor: u16::MAX,
        };

        let version = ClassFileVersion::try_from(raw).expect("valid preview version");

        assert_eq!(version.major(), 65);
        assert_eq!(version.java_version(), Some(21));
        assert!(version.preview());
    }

    #[test]
    fn preserves_major_when_preview_bit_is_set() {
        let raw = RawClassFileVersion {
            major: 70,
            minor: u16::MAX,
        };

        let version = ClassFileVersion::try_from(raw).expect("valid preview version");

        assert_eq!(version.major(), 70);
        assert!(version.preview());
    }

    #[test]
    fn preview_and_non_preview_versions_are_distinct() {
        let normal = ClassFileVersion::try_from(RawClassFileVersion {
            major: 65,
            minor: 0,
        })
        .unwrap();

        let preview = ClassFileVersion::try_from(RawClassFileVersion {
            major: 65,
            minor: u16::MAX,
        })
        .unwrap();

        assert_ne!(normal, preview);

        assert_eq!(normal.major(), preview.major());
        assert!(!normal.preview());
        assert!(preview.preview());
    }

    #[test]
    fn rejects_major_larger_than_u8() {
        let raw = RawClassFileVersion {
            major: 256,
            minor: 0,
        };

        let result = ClassFileVersion::try_from(raw);

        assert_eq!(result, Err(VersionError::InvalidMajor { major: 256 }));
    }

    #[test]
    fn rejects_maximum_u16_major() {
        let raw = RawClassFileVersion {
            major: u16::MAX,
            minor: 0,
        };

        let result = ClassFileVersion::try_from(raw);

        assert_eq!(result, Err(VersionError::InvalidMajor { major: u16::MAX }));
    }

    #[test]
    fn rejects_unsupported_minor_version() {
        let raw = RawClassFileVersion {
            major: 61,
            minor: 1,
        };

        let result = ClassFileVersion::try_from(raw);

        assert_eq!(
            result,
            Err(VersionError::InvalidMinor {
                major: 61,
                minor: 1,
            })
        );
    }

    #[test]
    fn rejects_arbitrary_minor_version() {
        let raw = RawClassFileVersion {
            major: 65,
            minor: 1234,
        };

        let result = ClassFileVersion::try_from(raw);

        assert_eq!(
            result,
            Err(VersionError::InvalidMinor {
                major: 65,
                minor: 1234,
            })
        );
    }

    #[test]
    fn rejects_preview_before_class_file_version_56() {
        let raw = RawClassFileVersion {
            major: 55,
            minor: u16::MAX,
        };

        let result = ClassFileVersion::try_from(raw);

        assert_eq!(result, Err(VersionError::InvalidMajor { major: 55 }));
    }

    #[test]
    fn accepts_preview_at_class_file_version_56() {
        let raw = RawClassFileVersion {
            major: 56,
            minor: u16::MAX,
        };

        let version = ClassFileVersion::try_from(raw).expect("major 56 supports preview");

        assert_eq!(version.major(), 56);
        assert_eq!(version.java_version(), Some(12));
        assert!(version.preview());
    }

    #[test]
    fn maps_major_52_to_java_8() {
        let version = ClassFileVersion::new_unchecked(52, false);

        assert_eq!(version.java_version(), Some(8));
    }

    #[test]
    fn maps_major_61_to_java_17() {
        let version = ClassFileVersion::new_unchecked(61, false);

        assert_eq!(version.java_version(), Some(17));
    }

    #[test]
    fn maps_major_65_to_java_21() {
        let version = ClassFileVersion::new_unchecked(65, false);

        assert_eq!(version.java_version(), Some(21));
    }

    #[test]
    fn maps_major_70_to_java_26() {
        let version = ClassFileVersion::new_unchecked(70, false);

        assert_eq!(version.java_version(), Some(26));
    }

    #[test]
    fn java_version_is_independent_of_preview_status() {
        let normal = ClassFileVersion::new_unchecked(65, false);
        let preview = ClassFileVersion::new_unchecked(65, true);

        assert_eq!(normal.java_version(), Some(21));
        assert_eq!(preview.java_version(), Some(21));
    }

    #[test]
    fn display_normal_version() {
        let version = ClassFileVersion::new_unchecked(65, false);

        assert_eq!(version.to_string(), "Java 21 (class file 65)");
    }

    #[test]
    fn display_preview_version() {
        let version = ClassFileVersion::new_unchecked(65, true);

        assert_eq!(version.to_string(), "Java 21 preview (class file 65)");
    }

    #[test]
    fn debug_normal_version() {
        let version = ClassFileVersion::new_unchecked(65, false);

        let debug = format!("{version:?}");

        assert!(debug.contains("ClassFileVersion"));
        assert!(debug.contains("major"));
        assert!(debug.contains("65"));
        assert!(debug.contains("java_version"));
        assert!(debug.contains("21"));
        assert!(debug.contains("preview"));
        assert!(debug.contains("false"));
    }

    #[test]
    fn debug_preview_version() {
        let version = ClassFileVersion::new_unchecked(65, true);

        let debug = format!("{version:?}");

        assert!(debug.contains("ClassFileVersion"));
        assert!(debug.contains("65"));
        assert!(debug.contains("21"));
        assert!(debug.contains("true"));
    }

    #[test]
    fn equivalent_versions_compare_equal() {
        let a = ClassFileVersion::new_unchecked(61, false);
        let b = ClassFileVersion::new_unchecked(61, false);

        assert_eq!(a, b);
    }

    #[test]
    fn different_major_versions_compare_not_equal() {
        let java_17 = ClassFileVersion::new_unchecked(61, false);
        let java_21 = ClassFileVersion::new_unchecked(65, false);

        assert_ne!(java_17, java_21);
    }

    #[test]
    fn copy_preserves_version() {
        let original = ClassFileVersion::new_unchecked(61, false);
        let copy = original;

        assert_eq!(original, copy);
        assert_eq!(copy.major(), 61);
    }

    #[test]
    fn all_internal_encodings_round_trip() {
        for major in 0..=127 {
            for preview in [false, true] {
                let version = ClassFileVersion::new_unchecked(major, preview);

                assert_eq!(version.major(), major);
                assert_eq!(version.preview(), preview);
            }
        }
    }

    #[test]
    fn raw_version_boundaries() {
        let cases = [
            (55, 0),
            (55, u16::MAX),
            (56, 0),
            (56, u16::MAX),
            (127, 0),
            (128, 0),
            (255, 0),
            (256, 0),
        ];

        for (major, minor) in cases {
            let _ = ClassFileVersion::try_from(RawClassFileVersion { major, minor });
        }
    }
}
