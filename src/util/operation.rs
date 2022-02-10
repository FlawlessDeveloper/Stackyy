use std::collections::HashMap;

use crate::compiler_error_str;
use crate::util::{compiler_warning, compiler_warning_str};
use crate::util::internals::{Internal, type_check};
use crate::util::position::Position;
use crate::util::token::Token;
use crate::util::type_check::Types;

pub type JumpOffset = u32;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub enum OperationType {
    PushInt,
    PushPtr,
    PushBool,
    PushStr,
    PushFunction,
    Internal,
    Jump,
    JumpIf,
    Call,
    CallIf,
}

#[derive(Clone, Debug)]
pub enum Operand {
    Int(i32),
    Str(String),
    Bool(bool),
    Internal(Internal),
    Jump(JumpOffset),
    Call(String),
}

#[derive(Clone, Debug)]
pub struct Operation {
    pub(crate) typ: OperationType,
    pub(crate) token: Token,
    pub(crate) operand: Option<Operand>,
}

impl Operation {
    pub fn type_check(&self, functions: HashMap<String, (Vec<Types>, Vec<Types>)>, stack: &mut Vec<Types>) -> bool {
        match self.typ {
            OperationType::PushInt => {
                stack.push(Types::Int);
                true
            }
            OperationType::PushPtr => {
                stack.push(Types::Pointer);
                true
            }
            OperationType::PushBool => {
                stack.push(Types::Bool);
                true
            }
            OperationType::PushStr => {
                stack.push(Types::String);
                true
            }
            OperationType::PushFunction => {
                stack.push(Types::Function);
                true
            }
            OperationType::Internal => {
                if let Operand::Internal(internal) = self.operand.clone().unwrap() {
                    type_check(internal, stack)
                } else {
                    compiler_error_str("Parser bug! Internal found without Internal operand", self.token.location().clone());
                    false
                }
            }
            OperationType::Jump | OperationType::JumpIf => {
                compiler_warning_str("Jumps not implemented yet", self.token.location().clone());
                false
            }
            OperationType::Call => {
                if let Operand::Call(fnc) = self.operand.clone().unwrap() {
                    if functions.contains_key(&fnc) {
                        let (contract_in, contract_out) = functions.get(&fnc).unwrap();

                        if stack.len() >= contract_in.len() {
                            for typ in contract_in {
                                let typ = typ.clone();

                                let stack_top = stack.pop().unwrap();

                                if typ != stack_top {
                                    return false;
                                }
                            }

                            for typ in contract_out {
                                stack.push(typ.clone());
                            }

                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            OperationType::CallIf => {
                if stack.len() == 0 {
                    false
                } else {
                    let a = stack.pop().unwrap();
                    let b = stack.pop().unwrap();
                    if a == Types::Bool && b == Types::Function {
                        true
                    } else {
                        false
                    }
                }
            }
        }
    }
}