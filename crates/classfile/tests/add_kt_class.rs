use std::path::PathBuf;

use dex_rs_classfile::{ClassFileVersion, ClassReader, RawClassFileVersion};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/kotlin/AddKt.class")
}

#[test]
fn parses_version_from_real_kotlin_class_file() {
    let bytes = std::fs::read(fixture()).expect("read AddKt.class fixture");

    let mut reader = ClassReader::new(&bytes);

    let raw = RawClassFileVersion::parse(&mut reader).expect("parse class file version");
    let version = ClassFileVersion::try_from(raw).expect("compact supported class file version");

    assert_eq!(reader.offset(), 8);
    assert_eq!(raw.major, 52);
    assert_eq!(raw.minor, 0);
    assert_eq!(version.major(), 52);
    assert_eq!(version.java_version(), Some(8));
    assert!(!version.preview());
}
