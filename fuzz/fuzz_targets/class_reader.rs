#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;

use dex_rs_classfile::{ClassReader, ParseError};

#[derive(Debug, Arbitrary)]
enum Operation {
    U8,
    U16,
    U32,
    U64,
    Bytes(u16),
}

#[derive(Debug, Arbitrary)]
struct Input {
    bytes: Vec<u8>,
    operations: Vec<Operation>,
}

fuzz_target!(|input: Input| {
    let mut reader = ClassReader::new(&input.bytes);

    for operation in input.operations {
        let before = reader.offset();

        match operation {
            Operation::U8 => {
                let result = reader.read_u8();
                check_result(result, &reader, before, 1, input.bytes.len());
                continue;
            }

            Operation::U16 => {
                let result = reader.read_u16();
                check_result(result, &reader, before, 2, input.bytes.len());
                continue;
            }

            Operation::U32 => {
                let result = reader.read_u32();
                check_result(result, &reader, before, 4, input.bytes.len());
                continue;
            }

            Operation::U64 => {
                let result = reader.read_u64();
                check_result(result, &reader, before, 8, input.bytes.len());
                continue;
            }

            Operation::Bytes(len) => {
                let len = len as usize;
                let result = reader.read_bytes(len);
                check_result(result, &reader, before, len, input.bytes.len());
                continue;
            }
        };
    }
});

fn check_result<T>(
    result: Result<T, ParseError>,
    reader: &ClassReader<'_>,
    before: usize,
    expected_advance: usize,
    total_len: usize,
) {
    match result {
        Ok(_) => {
            assert_eq!(reader.offset(), before + expected_advance);

            assert!(reader.offset() <= total_len);
        }

        Err(_) => {
            assert_eq!(
                reader.offset(),
                before,
                "failed read must not advance offset"
            );
        }
    }
}
