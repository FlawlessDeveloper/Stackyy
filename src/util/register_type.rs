use crate::util::operation::JumpOffset;

#[derive(Clone, Debug)]
pub enum RegisterType {
    Int(i32),
    Function(String),
    Pointer(u32),
    String(String),
    Bool(bool),
    Empty,
}