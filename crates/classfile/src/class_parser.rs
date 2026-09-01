pub(crate) use crate::instruction_set::InstructionSet;
use crate::{ClassFileVersion, ClassReader, ConstantPool, ParseError, RawClassFileVersion};

pub struct ClassParser<'a> {
    pub version: ClassFileVersion,
    pub constant_pool: ConstantPool<'a>,
}
impl<'a> ClassParser<'a> {
    fn decode_instructions(code: &[u8], code_length: usize) -> Result<InstructionSet, ParseError> {
        let mut reader = ClassReader::new(code);
        InstructionSet::decode(&mut reader, code_length)
    }

    pub fn parse(class_reader: &mut ClassReader<'a>) -> Self {
        fn parse_code_attribute(reader: &mut ClassReader, constant_pool: &ConstantPool) {
            let max_stack = reader.read_u16().unwrap();
            let max_locals = reader.read_u16().unwrap();

            let code_length = reader.read_u32().unwrap() as usize;
            let code = reader.read_bytes(code_length).unwrap();

            println!(
                "    Code max_stack={max_stack} \
                 max_locals={max_locals} \
                 code_length={code_length}"
            );

            let instructions =
                ClassParser::decode_instructions(code, code_length).expect("valid JVM bytecode");

            println!("    instructions:");

            for instruction in instructions.instructions {
                println!("      {instruction:?}");
            }

            let exception_table_length = reader.read_u16().unwrap();

            for _ in 0..exception_table_length {
                reader.read_u16().unwrap(); // start_pc
                reader.read_u16().unwrap(); // end_pc
                reader.read_u16().unwrap(); // handler_pc
                reader.read_u16().unwrap(); // catch_type
            }

            let attributes_count = reader.read_u16().unwrap();

            for _ in 0..attributes_count {
                parse_attribute(reader, constant_pool);
            }
        }

        fn parse_attribute(reader: &mut ClassReader, constant_pool: &ConstantPool) {
            let name_index = reader.read_u16().unwrap();

            let length = reader.read_u32().unwrap();

            let name = constant_pool.utf8(name_index).unwrap_or("<unknown>");

            println!(
                "  attribute \
                 name=#{name_index} \
                 {name:?} \
                 length={length}"
            );

            println!("name: {name}");
            match name {
                "Code" => {
                    println!("FOUND CODE ATTRIBUTE");
                    parse_code_attribute(reader, constant_pool);
                }

                _ => {
                    reader.read_bytes(length as usize).unwrap();
                }
            }
        }

        let raw = RawClassFileVersion::parse(class_reader).expect("parse class file version");

        let version =
            ClassFileVersion::try_from(raw).expect("compact supported class file version");

        let constant_pool = ConstantPool::try_from(&mut *class_reader).expect("parsed pool");

        let access_flags = class_reader.read_u16().unwrap();

        println!("# Access Flags={access_flags:#06x}");

        let this_class = class_reader.read_u16().unwrap();

        println!("# This class=#{this_class}");

        let super_class = class_reader.read_u16().unwrap();

        println!("# Super class=#{super_class}");

        let interface_count = class_reader.read_u16().unwrap();

        println!("# interface_count={interface_count}");

        for interface_index in 0..interface_count {
            let interface = class_reader.read_u16().unwrap();

            println!(
                "interface #{interface_index}: \
                 class=#{interface}"
            );
        }

        let fields_count = class_reader.read_u16().unwrap();

        println!("# fields_count={fields_count}");

        for field_index in 0..fields_count {
            let access_flags = class_reader.read_u16().unwrap();

            let name_index = class_reader.read_u16().unwrap();

            let descriptor_index = class_reader.read_u16().unwrap();

            let attributes_count = class_reader.read_u16().unwrap();

            let name = constant_pool.utf8(name_index).unwrap_or("<unknown>");

            let descriptor = constant_pool.utf8(descriptor_index).unwrap_or("<unknown>");

            println!(
                "field #{field_index}: \
                 flags={access_flags:#06x} \
                 name=#{name_index} {name:?} \
                 descriptor=#{descriptor_index} \
                 {descriptor:?} \
                 attributes={attributes_count}"
            );

            for _ in 0..attributes_count {
                parse_attribute(class_reader, &constant_pool);
            }
        }

        let methods_count = class_reader.read_u16().unwrap();

        println!("# methods_count={methods_count}");

        for method_index in 0..methods_count {
            let access_flags = class_reader.read_u16().unwrap();

            let name_index = class_reader.read_u16().unwrap();

            let descriptor_index = class_reader.read_u16().unwrap();

            let attributes_count = class_reader.read_u16().unwrap();

            let name = constant_pool.utf8(name_index).unwrap_or("<unknown>");

            let descriptor = constant_pool.utf8(descriptor_index).unwrap_or("<unknown>");

            println!(
                "method #{method_index}: \
                 flags={access_flags:#06x} \
                 name=#{name_index} {name:?} \
                 descriptor=#{descriptor_index} \
                 {descriptor:?} \
                 attributes={attributes_count}"
            );

            for _ in 0..attributes_count {
                parse_attribute(class_reader, &constant_pool);
            }
        }

        let attributes_count = class_reader.read_u16().unwrap();

        println!("# class attributes_count={attributes_count}");

        for _ in 0..attributes_count {
            parse_attribute(class_reader, &constant_pool);
        }

        ClassParser {
            version,
            constant_pool,
        }
    }
}
