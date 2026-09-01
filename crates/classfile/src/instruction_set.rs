use crate::{ClassReader, ParseError};
use crate::instructions::JvmInstruction;

pub struct InstructionSet {
    pub instructions: Vec<JvmInstruction>,
}

impl InstructionSet {
    pub fn decode(class_reader: &mut ClassReader, code_length: usize) -> Result<Self, ParseError> {
        let mut instructions = Vec::new();

        while class_reader.offset() < code_length {
            let instruction = Self::decode_instruction(class_reader)?;

            instructions.push(instruction);
        }

        Ok(InstructionSet {
            instructions
        })
    }

    fn decode_instruction(
        class_reader: &mut ClassReader
    ) -> Result<JvmInstruction, ParseError> {
        let opcode = class_reader.read_u8()?;
        Self::decode_opcode(opcode, class_reader)
    }

    fn decode_opcode(
        opcode: u8,
        class_reader: &mut ClassReader,
    ) -> Result<JvmInstruction, ParseError> {
        let instruction = match opcode {

            // ILOAD
            0x15 => JvmInstruction::ILoad(
                class_reader.read_u8()? as u16
            ),
            // ILOAD_0
            0x1a => JvmInstruction::ILoad(0),
            // ILOAD_1
            0x1b => JvmInstruction::ILoad(1),
            // ILOAD_2
            0x1c => JvmInstruction::ILoad(2),
            // ILOAD_3
            0x1d => JvmInstruction::ILoad(3),

            // ICONST_M1
            0x02 => JvmInstruction::IConst(-1),
            // ICONST_0
            0x03 => JvmInstruction::IConst(0),
            // ICONST_1
            0x04 => JvmInstruction::IConst(1),
            // ICONST_2
            0x05 => JvmInstruction::IConst(2),
            // ICONST_3
            0x06 => JvmInstruction::IConst(3),
            // ICONST_4
            0x07 => JvmInstruction::IConst(4),
            // ICONST_5
            0x08 => JvmInstruction::IConst(5),

            // ISTORE_0
            0x3b => JvmInstruction::IStore(0),
            // ISTORE_1
            0x3c => JvmInstruction::IStore(1),
            // ISTORE_2
            0x3d => JvmInstruction::IStore(2),
            // ISTORE_3
            0x3e => JvmInstruction::IStore(3),

            0xb2 => Self::decode_get_static(class_reader)?,
            0xb6 => Self::decode_invoke_static(class_reader)?,
            0xb8 => Self::decode_invoke_virtual(class_reader)?,
            0x60 => JvmInstruction::IAdd,
            0xac => JvmInstruction::IReturn,
            0xb1 => JvmInstruction::Return,

            0xb7 => {
                let index = class_reader.read_u16()?;
                JvmInstruction::InvokeSpecial { index }
            }

            opcode => return Err(ParseError::InvalidOpcode { opcode }),
        };
        Ok(instruction)
    }

    fn decode_invoke_virtual(class_reader: &mut ClassReader) -> Result<JvmInstruction, ParseError> {
        let index = class_reader.read_u16()?;
        Ok(JvmInstruction::InvokeVirtual(index))
    }

    fn decode_invoke_static(class_reader: &mut ClassReader) -> Result<JvmInstruction, ParseError> {
        let index = class_reader.read_u16()?;
        Ok(JvmInstruction::InvokeStatic(index))
    }


    fn decode_get_static(class_reader: &mut ClassReader) -> Result<JvmInstruction, ParseError> {
        let index = class_reader.read_u16()?;
        Ok(JvmInstruction::GetStatic(index))
    }
}
