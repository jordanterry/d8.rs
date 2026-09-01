use std::path::PathBuf;

use dex_rs_classfile::{ClassParser, ClassReader};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/kotlin/AddKt.class")
}

#[test]
fn parses_version_from_real_kotlin_class_file() {
    let bytes = std::fs::read(fixture()).expect("read AddKt.class fixture");

    let mut reader = ClassReader::new(&bytes);
    let class_parser = ClassParser::parse(&mut reader);

    assert_eq!(class_parser.version.java_version(), Some(8));
    assert!(!class_parser.version.preview());
}
