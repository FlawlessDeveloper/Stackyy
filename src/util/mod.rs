use crate::util::position::Position;

pub mod position;
pub mod token;
pub mod operation;
pub mod internals;
pub mod register_type;
pub mod type_check;
pub mod operations;

pub fn compiler_error(msg: String, pos: Position) {
    panic!("ERROR at {} -> {}", pos, msg);
}

pub fn compiler_error_str(msg: &str, pos: Position) {
    compiler_error(msg.to_string(), pos);
}

pub fn compiler_warning(msg: String, pos: Position) {
    eprintln!("WARNING at {} -> {}", pos, msg);
}

pub fn compiler_warning_str(msg: &str, pos: Position) {
    compiler_warning(msg.to_string(), pos);
}

pub fn runtime_error(msg: String, pos: Position) {
    panic!("RUNTIME ERROR at {} -> {}", pos, msg);
}

pub fn runtime_error_str(msg: &str, pos: Position) {
    runtime_error(msg.to_string(), pos);
}

pub fn runtime_warning(msg: String, pos: Position) {
    eprintln!("RUNTIME ERROR at {} -> {}", pos, msg);
}

pub fn runtime_warning_str(msg: &str, pos: Position) {
    runtime_warning(msg.to_string(), pos);
}