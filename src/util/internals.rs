use crate::util::{compiler_error, compiler_error_str};
use crate::util::internals::Internal::{NoOp, Print};
use crate::util::position::Position;
use crate::util::token::TokenValue;

static INTERNALS: [&str; 2] = [
    "noop",
    "print"
];

#[derive(Clone, Debug)]
pub enum Internal {
    NoOp,
    Print,
}

pub fn to_internal(str: &TokenValue, pos: Position) -> Internal {
    if let TokenValue::String(str) = str {
        if INTERNALS.contains(&str.as_str()) {
            match str.as_str() {
                "noop" => NoOp,
                "print" | "puts" | "sysout" => Print,
                _ => {
                    compiler_error(format!("The internal call {} is not implemented", str), pos);
                    unreachable!()
                }
            }
        } else {
            compiler_error(format!("The internal call {} does not exit", str), pos);
            unreachable!()
        }
    } else {
        compiler_error_str("Internal parser error occurred", pos);
        unreachable!()
    }
}