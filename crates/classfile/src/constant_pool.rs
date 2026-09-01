use crate::constant_pool::ConstantPoolEntry::Unusable;
use crate::{ClassReader, ParseError};
use ConstantPoolEntry::{
    Class, Double, Dynamic, FieldRef, Float, Integer, InterfaceMethodRef, InvokeDynamic, Long,
    MethodHandle, MethodRef, MethodType, Module, NameAndType, Package, Utf8,
};

/// tag generated from a utf8 tag.
/// Allows exhaustive checking and eager compiler errors.
#[repr(u8)]
// Q: does this allows the enum to be sized to max 1 byte? Rather than having to.. forgotten the term?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstantPoolTag {
    Utf8 = 1,
    Integer = 3,
    Float = 4,
    Long = 5,
    Double = 6,
    Class = 7,
    String = 8,
    FieldRef = 9,
    MethodRef = 10,
    InterfaceMethodRef = 11,
    NameAndType = 12,
    MethodHandle = 15,
    MethodType = 16,
    Dynamic = 17,
    InvokeDynamic = 18,
    Module = 19,
    Package = 20,
}

impl TryFrom<u8> for ConstantPoolTag {
    type Error = u8;

    fn try_from(tag: u8) -> Result<Self, Self::Error> {
        match tag {
            1 => Ok(Self::Utf8),
            3 => Ok(Self::Integer),
            4 => Ok(Self::Float),
            5 => Ok(Self::Long),
            6 => Ok(Self::Double),
            7 => Ok(Self::Class),
            8 => Ok(Self::String),
            9 => Ok(Self::FieldRef),
            10 => Ok(Self::MethodRef),
            11 => Ok(Self::InterfaceMethodRef),
            12 => Ok(Self::NameAndType),
            15 => Ok(Self::MethodHandle),
            16 => Ok(Self::MethodType),
            17 => Ok(Self::Dynamic),
            18 => Ok(Self::InvokeDynamic),
            19 => Ok(Self::Module),
            20 => Ok(Self::Package),
            unknown => Err(unknown),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClassIndex(u16);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NameAndTypeIndex(u16);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BootstrapMethodIndex(u16);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Utf8Index(u16);
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceIndex(u16);

pub enum ConstantPoolEntry<'a> {
    Utf8(&'a [u8]),

    Integer(i32),
    Float(u32),
    Long(i64),
    Double(u64),

    Class {
        name_index: Utf8Index,
    },

    String {
        string_index: Utf8Index,
    },

    FieldRef {
        class_index: ClassIndex,
        name_and_type_index: NameAndTypeIndex,
    },

    MethodRef {
        class_index: ClassIndex,
        name_and_type_index: NameAndTypeIndex,
    },

    InterfaceMethodRef {
        class_index: ClassIndex,
        name_and_type_index: NameAndTypeIndex,
    },

    NameAndType {
        name_index: Utf8Index,
        descriptor_index: Utf8Index,
    },

    MethodHandle {
        reference_kind: u8,
        reference_index: ReferenceIndex,
    },

    MethodType {
        descriptor_index: Utf8Index,
    },

    Dynamic {
        bootstrap_method_attr_index: BootstrapMethodIndex,
        name_and_type_index: NameAndTypeIndex,
    },

    InvokeDynamic {
        bootstrap_method_attr_index: BootstrapMethodIndex,
        name_and_type_index: NameAndTypeIndex,
    },

    Module {
        name_index: Utf8Index,
    },

    Package {
        name_index: Utf8Index,
    },

    Unusable,
}

pub struct ConstantPool<'a> {
    pub count: u16,
    entries: Vec<ConstantPoolEntry<'a>>,
}

impl<'a> TryFrom<&mut ClassReader<'a>> for ConstantPool<'a> {
    type Error = ParseError;

    fn try_from(reader: &mut ClassReader<'a>) -> Result<Self, Self::Error> {
        let count = reader.read_u16()?;

        let mut entries = Vec::with_capacity(count as usize);
        entries.push(Unusable);

        while entries.len() < count as usize {
            let offset = reader.offset();
            let raw_tag = reader.read_u8()?;

            let tag = ConstantPoolTag::try_from(raw_tag)
                .map_err(|tag| ParseError::InvalidConstantPoolTag { offset, tag })?;

            let decoded = decode_entry(tag, reader)?;

            match decoded {
                DecodedEntry::One(entry) => {
                    entries.push(entry);
                }

                DecodedEntry::Two(one, two) => {
                    if entries.len() + 2 > count as usize {
                        return Err(ParseError::InvalidConstantPoolLayout { offset });
                    }

                    entries.push(one);
                    entries.push(two);
                }
            }
        }

        Ok(Self { count, entries })
    }
}

impl ConstantPool<'_> {
    pub fn get(&self, index: u16) -> Option<&ConstantPoolEntry<'_>> {
        if index == 0 {
            return None;
        }

        self.entries.get(index as usize)
    }

    pub fn utf8(&self, index: u16) -> Option<&str> {
        match self.get(index)? {
            Utf8(bytes) => std::str::from_utf8(bytes).ok(),

            _ => None,
        }
    }
}

enum DecodedEntry<'a> {
    One(ConstantPoolEntry<'a>),
    Two(ConstantPoolEntry<'a>, ConstantPoolEntry<'a>),
}

fn decode_entry<'a>(
    tag: ConstantPoolTag,
    reader: &mut ClassReader<'a>,
) -> Result<DecodedEntry<'a>, ParseError> {
    match tag {
        ConstantPoolTag::Utf8 => decode_utf8(reader),
        ConstantPoolTag::Integer => decode_integer(reader),
        ConstantPoolTag::Float => decode_float(reader),
        ConstantPoolTag::Long => decode_long(reader),
        ConstantPoolTag::Double => decode_double(reader),
        ConstantPoolTag::Class => decode_class(reader),
        ConstantPoolTag::String => decode_string(reader),
        ConstantPoolTag::FieldRef => decode_field_ref(reader),
        ConstantPoolTag::MethodRef => decode_method_ref(reader),
        ConstantPoolTag::InterfaceMethodRef => decode_interface_method_ref(reader),
        ConstantPoolTag::NameAndType => decode_name_and_type(reader),
        ConstantPoolTag::MethodHandle => decode_method_handle(reader),
        ConstantPoolTag::MethodType => decode_method_type(reader),
        ConstantPoolTag::Dynamic => decode_dynamic(reader),
        ConstantPoolTag::InvokeDynamic => decode_invoke_dynamic(reader),
        ConstantPoolTag::Module => decode_module(reader),
        ConstantPoolTag::Package => decode_package(reader),
    }
}

fn decode_method_ref<'a>(reader: &mut ClassReader) -> Result<DecodedEntry<'a>, ParseError> {
    Ok(DecodedEntry::One(MethodRef {
        class_index: ClassIndex(reader.read_u16()?),
        name_and_type_index: NameAndTypeIndex(reader.read_u16()?),
    }))
}

fn decode_interface_method_ref<'a>(
    reader: &mut ClassReader,
) -> Result<DecodedEntry<'a>, ParseError> {
    Ok(DecodedEntry::One(InterfaceMethodRef {
        class_index: ClassIndex(reader.read_u16()?),
        name_and_type_index: NameAndTypeIndex(reader.read_u16()?),
    }))
}

fn decode_name_and_type<'a>(reader: &mut ClassReader) -> Result<DecodedEntry<'a>, ParseError> {
    Ok(DecodedEntry::One(NameAndType {
        name_index: Utf8Index(reader.read_u16()?),
        descriptor_index: Utf8Index(reader.read_u16()?),
    }))
}

fn decode_method_handle<'a>(reader: &mut ClassReader) -> Result<DecodedEntry<'a>, ParseError> {
    Ok(DecodedEntry::One(MethodHandle {
        reference_kind: reader.read_u8()?,
        reference_index: ReferenceIndex(reader.read_u16()?),
    }))
}

fn decode_method_type<'a>(reader: &mut ClassReader) -> Result<DecodedEntry<'a>, ParseError> {
    Ok(DecodedEntry::One(MethodType {
        descriptor_index: Utf8Index(reader.read_u16()?),
    }))
}

fn decode_dynamic<'a>(reader: &mut ClassReader) -> Result<DecodedEntry<'a>, ParseError> {
    Ok(DecodedEntry::One(Dynamic {
        bootstrap_method_attr_index: BootstrapMethodIndex(reader.read_u16()?),
        name_and_type_index: NameAndTypeIndex(reader.read_u16()?),
    }))
}
fn decode_invoke_dynamic<'a>(reader: &mut ClassReader) -> Result<DecodedEntry<'a>, ParseError> {
    Ok(DecodedEntry::One(InvokeDynamic {
        bootstrap_method_attr_index: BootstrapMethodIndex(reader.read_u16()?),
        name_and_type_index: NameAndTypeIndex(reader.read_u16()?),
    }))
}

fn decode_package<'a>(reader: &mut ClassReader) -> Result<DecodedEntry<'a>, ParseError> {
    Ok(DecodedEntry::One(Package {
        name_index: Utf8Index(reader.read_u16()?),
    }))
}

fn decode_module<'a>(reader: &mut ClassReader) -> Result<DecodedEntry<'a>, ParseError> {
    Ok(DecodedEntry::One(Module {
        name_index: Utf8Index(reader.read_u16()?),
    }))
}

fn decode_field_ref<'a>(reader: &mut ClassReader) -> Result<DecodedEntry<'a>, ParseError> {
    Ok(DecodedEntry::One(FieldRef {
        class_index: ClassIndex(reader.read_u16()?),
        name_and_type_index: NameAndTypeIndex(reader.read_u16()?),
    }))
}

fn decode_string<'a>(reader: &mut ClassReader) -> Result<DecodedEntry<'a>, ParseError> {
    Ok(DecodedEntry::One(ConstantPoolEntry::String {
        string_index: Utf8Index(reader.read_u16()?),
    }))
}

fn decode_class<'a>(reader: &mut ClassReader) -> Result<DecodedEntry<'a>, ParseError> {
    Ok(DecodedEntry::One(Class {
        name_index: Utf8Index(reader.read_u16()?),
    }))
}

fn decode_double<'a>(reader: &mut ClassReader) -> Result<DecodedEntry<'a>, ParseError> {
    Ok(DecodedEntry::Two(Double(reader.read_u64()?), Unusable))
}

fn decode_long<'a>(reader: &mut ClassReader) -> Result<DecodedEntry<'a>, ParseError> {
    Ok(DecodedEntry::Two(Long(reader.read_u64()? as i64), Unusable))
}

fn decode_float<'a>(reader: &mut ClassReader) -> Result<DecodedEntry<'a>, ParseError> {
    Ok(DecodedEntry::One(Float(reader.read_u32()?)))
}

fn decode_integer<'a>(reader: &mut ClassReader) -> Result<DecodedEntry<'a>, ParseError> {
    Ok(DecodedEntry::One(Integer(reader.read_u32()? as i32)))
}

fn decode_utf8<'a>(reader: &mut ClassReader<'a>) -> Result<DecodedEntry<'a>, ParseError> {
    let length = reader.read_u16()? as usize;
    let bytes = reader.read_bytes(length)?;

    Ok(DecodedEntry::One(Utf8(bytes)))
}
