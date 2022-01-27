use crate::util::operation::OperationAddr;

#[derive(Debug)]
pub enum RegisterType {
    Int(i32),
    Pointer(OperationAddr),
    String(String),
    Bool(bool),
    Empty
}