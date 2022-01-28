use crate::util::{compiler_error, compiler_error_str};
use crate::util::internals::Internal::{DbgStack, Drop, DropStack, Dup, DupStack, NoOp, Print, PrintLn, RevStack, Swap};
use crate::util::position::Position;
use crate::util::token::TokenValue;

static INTERNALS: [&str; 10] = [
    "noop",
    "print",
    "println",
    "swap",
    "drop",
    "dup",
    "rev_stack",
    "drop_stack",
    "dup_stack",
    "dbg_stack",
];

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Internal {
    NoOp,
    Print,
    PrintLn,
    Swap,
    Drop,
    Dup,
    RevStack,
    DropStack,
    DupStack,
    DbgStack,
}

pub fn to_internal(str: &TokenValue, pos: Position) -> Internal {
    if let TokenValue::String(str) = str {
        if INTERNALS.contains(&str.as_str()) {
            match str.as_str() {
                "noop" => NoOp,
                "print" => Print,
                "println"  => PrintLn,
                "swap"  => Swap,
                "drop"  => Drop,
                "swap"  => Swap,
                "dup"  => Dup,
                "rev_stack"  => RevStack,
                "drop_stack"  => DropStack,
                "dup_stack"  => DupStack,
                "dbg_stack"  => DbgStack,
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