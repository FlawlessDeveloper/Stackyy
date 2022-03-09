use std::collections::HashMap;

use crate::{compiler_error, compiler_error_str};
use crate::parser::Function;
use crate::util::{compiler_warning, compiler_warning_str};
use crate::util::internals::{Internal, type_check};
use crate::util::position::Position;
use crate::util::token::Token;
use crate::util::type_check::{ErrorTypes, TypeCheckError, Types};

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
    PushFunction(String, Vec<Types>, Vec<Types>),
    Call(String),
}

#[derive(Clone, Debug)]
pub struct Operation {
    pub(crate) typ: OperationType,
    pub(crate) token: Token,
    pub(crate) operand: Option<Operand>,
}

impl Operation {
    pub fn type_check(&self, functions: &HashMap<String, Function>, stack: &mut Vec<Types>, compile_time: bool) -> TypeCheckError {
        match self.typ {
            OperationType::PushInt => {
                stack.push(Types::Int);
                ErrorTypes::None.into()
            }
            OperationType::PushPtr => {
                stack.push(Types::Pointer);
                ErrorTypes::None.into()
            }
            OperationType::PushBool => {
                stack.push(Types::Bool);
                ErrorTypes::None.into()
            }
            OperationType::PushStr => {
                stack.push(Types::String);
                ErrorTypes::None.into()
            }
            OperationType::PushFunction => {
                if let Operand::PushFunction(_, inp, outp) = self.operand.clone().unwrap() {
                    stack.push(Types::FunctionPointer(inp, outp));
                    ErrorTypes::None.into()
                } else {
                    ErrorTypes::InvalidTypes.into_with_ctx(vec![Types::Function], stack.clone())
                }
            }
            OperationType::Internal => {
                if let Operand::Internal(internal) = self.operand.clone().unwrap() {
                    type_check(internal, stack)
                } else {
                    compiler_error_str("Parser bug! Internal found without Internal operand", self.token.location().clone());
                    ErrorTypes::Raw("Parser bug! Internal found without Internal operand".to_string()).into()
                }
            }
            OperationType::Jump | OperationType::JumpIf => {
                compiler_warning_str("Jumps not implemented yet", self.token.location().clone());
                ErrorTypes::Raw("Jumps not implemented yet".to_string()).into()
            }
            OperationType::Call | OperationType::CallIf => {
                let (success, inp, outp): (TypeCheckError, Vec<Types>, Vec<Types>) = if let Some(inner) = self.operand.clone() {
                    match inner {
                        Operand::Call(ref name) | Operand::PushFunction(ref name, _, _) => {
                            if functions.contains_key(name) {
                                let (inp, outp) = if let Operand::PushFunction(_, inp, outp) = inner {
                                    (inp, outp)
                                } else {
                                    functions.get(name).unwrap().get_contract()
                                };
                                (ErrorTypes::None.into(), inp, outp)
                            } else {
                                (ErrorTypes::WrongData.into(), vec![], stack.clone())
                            }
                        }
                        _ => {
                            println!("{:?}", inner);
                            (ErrorTypes::WrongData.into(), vec![], stack.clone())
                        }
                    }
                } else {
                    if stack.len() == 0 {
                        (ErrorTypes::TooFewElements.into_with_ctx_plus(vec![Types::FunctionPointer(vec![Types::Any], vec![Types::Any])], stack.clone(), "Function pointer can contain any amount/sort of types"), vec![], vec![])
                    } else {
                        let top = stack.pop().unwrap();
                        if let Types::FunctionPointer(inp, outp) = top {
                            (ErrorTypes::None.into(), inp, outp)
                        } else {
                            (ErrorTypes::WrongData.into(), vec![], vec![])
                        }
                    }
                };

                if success.error == ErrorTypes::None {
                    if stack.len() >= inp.len() {
                        let success = {
                            let mut tmp_inp = inp.clone();
                            let mut tmp_stack = stack.clone();
                            if inp.len() <= tmp_stack.len() {
                                let mut success = true;
                                while tmp_stack.len() > 0 {
                                    let a = tmp_inp.pop().unwrap();
                                    let b = tmp_stack.pop().unwrap();
                                    if a != b {
                                        success = false;
                                    }
                                }
                                success
                            } else {
                                false
                            }
                        };

                        if !success {
                            ErrorTypes::InvalidTypes.into_with_ctx(inp, stack.clone())
                        } else {
                            for _ in 0..inp.len() {
                                stack.pop().unwrap();
                            }
                            for typ in outp {
                                stack.push(typ.clone());
                            }
                            ErrorTypes::None.into()
                        }
                    } else {
                        ErrorTypes::TooFewElements.into_with_ctx_plus(vec![Types::FunctionPointer(vec![Types::Any], vec![Types::Any])], stack.clone(), "Function pointer can contain any amount/sort of types")
                    }
                } else {
                    success
                }
            }
        }
    }
}