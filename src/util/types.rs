use crate::util::{compiler_error, compiler_error_str};
use crate::util::position::Position;
use crate::util::types::Types::{Bool, Function, Int, Pointer, String as TString};

static TYPES: [&'static str; 5] = [
    "int",
    "str",
    "bool",
    "ptr",
    "fn",
];

#[derive(Debug, Clone)]
pub enum Types {
    Int,
    String,
    Bool,
    Pointer,
    Function,
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