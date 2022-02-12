use crate::util::operation::JumpOffset;
use crate::util::type_check::Types;

#[derive(Clone, Debug)]
pub enum RegisterType {
    Int(i32),
    Function(String, Vec<Types>, Vec<Types>),
    Pointer(u32),
    String(String),
    Bool(bool),
    Empty,
}