#[derive(Debug, Clone, PartialEq)]
pub enum JvmInstruction {
    ILoad(u16),
    IConst(i16),
    IStore(u16),
    IAdd,
    IReturn,
    Return,
    GetStatic(u16),
    InvokeVirtual(u16),
    InvokeStatic(u16),
    InvokeSpecial { index: u16 },
    Unknown { opcode: u8 },
}
