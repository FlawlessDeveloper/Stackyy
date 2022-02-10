use crate::util::{compiler_error, compiler_error_str};
use crate::util::internals::Internal::{Cubed, DbgStack, Div, Drop, DropStack, Dup, DupStack, Equals, Larger, LargerEq, Minus, Modulo, Mult, NoOp, Not, NotPeek, Plus, Print, PrintLn, RevStack, Smaller, SmallerEq, Squared, Swap};
use crate::util::position::Position;
use crate::util::token::TokenValue;
use crate::util::type_check::{Takes, Types};

static INTERNALS: [&str; 24] = [
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
    "+",
    "-",
    "*",
    "/",
    "%",
    "squared",
    "cubed",
    "!",
    "@!",
    "=",
    "<",
    ">",
    "<=",
    ">=",
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
    Plus,
    Minus,
    Mult,
    Div,
    Modulo,
    Squared,
    Cubed,
    Not,
    NotPeek,
    Equals,
    Larger,
    Smaller,
    LargerEq,
    SmallerEq,
}

pub fn to_internal(str: &TokenValue, pos: Position) -> Internal {
    if let TokenValue::String(str) = str {
        if INTERNALS.contains(&str.as_str()) {
            match str.as_str() {
                "noop" => NoOp,
                "print" => Print,
                "println" => PrintLn,
                "swap" => Swap,
                "drop" => Drop,
                "swap" => Swap,
                "dup" => Dup,
                "rev_stack" => RevStack,
                "drop_stack" => DropStack,
                "dup_stack" => DupStack,
                "dbg_stack" => DbgStack,
                "!" => Not,
                "@!" => NotPeek,
                "+" => Plus,
                "-" => Minus,
                "*" => Mult,
                "/" => Div,
                "%" => Modulo,
                "squared" => Squared,
                "cubed" => Cubed,
                "=" => Equals,
                "<" => Larger,
                ">" => Smaller,
                "<=" => LargerEq,
                ">=" => SmallerEq,
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

pub fn type_check(int: Internal, stack: &mut Vec<Types>) -> bool {
    match int {
        NoOp | DbgStack => {
            true
        }
        Print | PrintLn => {
            if stack.len() == 0 {
                false
            } else {
                stack.pop();
                true
            }
        }
        Swap => {
            if stack.len() < 2 {
                false
            } else {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();
                stack.push(a);
                stack.push(b);
                true
            }
        }
        Drop => {
            if stack.len() == 0 {
                false
            } else {
                stack.pop();
                true
            }
        }
        Dup => {
            if stack.len() == 0 {
                false
            } else {
                let last = stack.last().unwrap().clone();
                stack.push(last);
                true
            }
        }
        RevStack => {
            stack.reverse();
            true
        }
        DropStack => {
            stack.clear();
            true
        }
        DupStack => {
            stack.extend(stack.clone());
            true
        }
        Plus | Minus | Mult | Div | Modulo => {
            if stack.len() < 2 {
                false
            } else {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();

                if a != Types::Int || b != Types::Int {
                    false
                } else {
                    stack.push(Types::Int);
                    true
                }
            }
        }
        Squared | Cubed => {
            if stack.len() == 0 {
                false
            } else {
                let last = stack.last().unwrap().clone();
                if last == Types::Int {
                    true
                } else {
                    false
                }
            }
        }
        Not | NotPeek => {
            if stack.len() == 0 {
                false
            } else {
                let last = if int == Internal::Not {
                    stack.pop().unwrap().clone()
                } else {
                    stack.last().unwrap().clone()
                };
                if last == Types::Bool {
                    stack.push(Types::Bool);
                    true
                } else {
                    false
                }
            }
        }
        Equals | Larger | Smaller | LargerEq | SmallerEq => {
            const allowed_types: [Types; 3] = [Types::Int, Types::String, Types::Bool];

            if stack.len() < 2 {
                false
            } else {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();

                if allowed_types.contains(&a) && allowed_types.contains(&b) {
                    stack.push(Types::Bool);
                    true
                } else {
                    false
                }
            }
        }
    }
}