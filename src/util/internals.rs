use crate::util::{compiler_error, compiler_error_str};
use crate::util::internals::Internal::{NoOp, Print, PrintLn};
use crate::util::position::Position;
use crate::util::token::TokenValue;

static INTERNALS: [&str; 3] = [
    "noop",
    "print",
    "println"
];

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Internal {
    NoOp,
    Print,
    PrintLn,
}

pub fn to_internal(str: &TokenValue, pos: Position) -> Internal {
    if let TokenValue::String(str) = str {
        if INTERNALS.contains(&str.as_str()) {
            match str.as_str() {
                "noop" => NoOp,
                "print" => Print,
                "println"  => PrintLn,
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