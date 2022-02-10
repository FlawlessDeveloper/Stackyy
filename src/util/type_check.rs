use crate::util::{compiler_error, compiler_error_str};
use crate::util::position::Position;
use crate::util::type_check::Types::{Bool, Function, Int, Pointer, String as TString};

static TYPES: [&'static str; 5] = [
    "int",
    "str",
    "bool",
    "ptr",
    "fn",
];

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Types {
    Int,
    String,
    Bool,
    Pointer,
    Function,
}

pub enum Takes {
    None,
    AnyIn(u16),
    AnyOut(u16),
    AnyInOut(u16, u16),
    In(u16, Vec<Types>),
    Out(u16, Vec<Types>),
    InOut(u16, Vec<Types>, u16, Vec<Types>),
}

impl From<(Position, String)> for Types {
    fn from(token: (Position, String)) -> Self {
        if TYPES.contains(&token.1.as_str()) {
            match token.1.as_str() {
                "int" => Int,
                "str" => TString,
                "bool" => Bool,
                "ptr" => Pointer,
                "fn" => Function,
                _ => {
                    compiler_error(format!("Tokenizing for {} not implemented yet", token.1), token.0);
                    unreachable!()
                }
            }
        } else {
            compiler_error(format!("Invalid type: {}", token.1), token.0);
            unreachable!()
        }
    }
}