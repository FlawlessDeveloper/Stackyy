use crate::util::operation::JumpOffset;

#[derive(Clone, Debug)]
pub enum RegisterType {
    Int(i32),
    Pointer(JumpOffset),
    String(String),
    Bool(bool),
    Empty
}