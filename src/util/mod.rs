use crate::util::operation::{Operation, OperationDataInfo};
use crate::util::position::Position;

pub mod position;
pub mod token;
pub mod operation;
pub mod internals;
pub mod register_type;
pub mod type_check;
pub mod operations;
pub mod compile;

pub fn compiler_error(msg: String, pos: &OperationDataInfo) -> ! {
    panic!("ERROR {} -> {}", pos, msg);
}

pub fn compiler_error_str(msg: &str, pos: &OperationDataInfo) -> ! {
    compiler_error(msg.to_string(), pos);
}

pub fn compiler_warning(msg: String, pos: &OperationDataInfo) {
    eprintln!("WARNING {} -> {}", pos, msg);
}

pub fn compiler_warning_str(msg: &str, pos: &OperationDataInfo) {
    compiler_warning(msg.to_string(), pos);
}

pub fn runtime_error(msg: String, pos: &OperationDataInfo) -> ! {
    panic!("RUNTIME ERROR {} -> {}", pos, msg);
}

pub fn runtime_error_str(msg: &str, pos: &OperationDataInfo) -> ! {
    runtime_error(msg.to_string(), pos);
}

pub fn runtime_warning(msg: String, pos: &OperationDataInfo) {
    eprintln!("RUNTIME ERROR {} -> {}", pos, msg);
}

pub fn runtime_warning_str(msg: &str, pos: &OperationDataInfo) {
    runtime_warning(msg.to_string(), pos);
}