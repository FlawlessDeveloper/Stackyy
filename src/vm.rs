use std::any::Any;

use crate::parser::State;
use crate::util::{compiler_error, compiler_error_str, runtime_error_str};
use crate::util::internals::Internal;
use crate::util::operation::{Operand, Operation, OperationType};
use crate::util::position::Position;
use crate::util::register_type::RegisterType;
use crate::vm;

pub struct VM {
    ip: i32,
    ops: Vec<(Position, Operation)>,
    stack: Vec<RegisterType>,
    reg_a: RegisterType,
    reg_b: RegisterType,
    reg_c: RegisterType,
    reg_d: RegisterType,
    reg_e: RegisterType,
    reg_f: RegisterType,
    reg_g: RegisterType,
    reg_h: RegisterType,
}

impl From<State> for VM {
    fn from(state: State) -> Self {
        let mut ops = state.get_ops();
        Self {
            ip: 0,
            ops,
            stack: vec![],
            reg_a: RegisterType::Empty,
            reg_b: RegisterType::Empty,
            reg_c: RegisterType::Empty,
            reg_d: RegisterType::Empty,
            reg_e: RegisterType::Empty,
            reg_f: RegisterType::Empty,
            reg_g: RegisterType::Empty,
            reg_h: RegisterType::Empty,
        }
    }
}

impl VM {
    pub fn run(&mut self) {
        while self.ip < self.ops.len() as i32 {
            let op = self.ops.get(self.ip as usize).unwrap();

            self.execute(op.clone());
            self.ip += 1;
        }
    }

    fn execute(&mut self, op: (Position, Operation)) {
        let postion = op.0;
        let operation = op.1;
        match operation.typ {
            OperationType::PushInt => {
                if let Operand::Int(op) = operation.operand.unwrap() {
                    self.stack.push(RegisterType::Int(op))
                }
            }
            OperationType::PushPtr => {
                if let Operand::Jump(op) = operation.operand.unwrap() {
                    self.stack.push(RegisterType::Pointer(op))
                }
            }
            OperationType::PushBool => {
                if let Operand::Bool(bool) = operation.operand.unwrap() {
                    self.stack.push(RegisterType::Bool(bool))
                }
            }
            OperationType::PushStr => {
                if let Operand::Str(str) = operation.operand.unwrap() {
                    self.stack.push(RegisterType::String(str))
                }
            }
            OperationType::Internal => {
                if let Operand::Internal(int) = operation.operand.unwrap() {
                    match int {
                        Internal::NoOp => {}
                        Internal::Print | Internal::PrintLn => {
                            if self.stack.len() == 0 {
                                runtime_error_str("To few elements on stack", postion.clone());
                            }
                            let reg = self.stack.pop().unwrap();
                            match reg {
                                RegisterType::Int(int) => {
                                    print!("{}", int);
                                }
                                RegisterType::Pointer(pointer) => {
                                    print!("*{:#x}", pointer);
                                }
                                RegisterType::String(str) => {
                                    print!("{}", str)
                                }
                                RegisterType::Bool(bool) => {
                                    print!("{}", bool)
                                }
                                RegisterType::Empty => {
                                    runtime_error_str("Stack is empty", postion.clone());
                                }
                            }

                            if int == Internal::PrintLn {
                                println!();
                            }
                        }
                        Internal::Swap => {
                            if self.stack.len() < 2 {
                                runtime_error_str("To few elements on stack", postion.clone());
                            }

                            let a = self.stack.pop().unwrap();
                            let b = self.stack.pop().unwrap();
                            self.stack.push(a);
                            self.stack.push(b);
                        }
                        Internal::Drop => {
                            if self.stack.len() < 1 {
                                runtime_error_str("To few elements on stack", postion.clone());
                            }
                            self.stack.pop().unwrap();
                        }
                        Internal::Dup => {
                            if self.stack.len() < 1 {
                                runtime_error_str("To few elements on stack", postion.clone());
                            }

                            let top = self.stack.pop().unwrap();
                            self.stack.push(top.clone());
                            self.stack.push(top);
                        }
                        Internal::RevStack => {
                            self.stack.reverse();
                        }
                        Internal::DropStack => {
                            self.stack.clear();
                            self.stack.shrink_to_fit()
                        }
                        Internal::DupStack => {
                            let to_add = self.stack.clone();
                            self.stack.extend(to_add);
                        }
                        Internal::DbgStack => {
                            println!("{:#?}", self.stack);
                        }
                        Internal::_IfStarts => {}
                        Internal::Equals => {
                            if self.stack.len() < 2 {
                                runtime_error_str("To few elements on stack", postion.clone());
                            }
                            let a = self.stack.pop().unwrap();
                            let b = self.stack.pop().unwrap();

                            let success = if let RegisterType::String(stra) = a {
                                if let RegisterType::String(strb) = b {
                                    stra == strb
                                } else {
                                    false
                                }
                            } else if let RegisterType::Int(inta) = a {
                                if let RegisterType::Int(intb) = b {
                                    inta == intb
                                } else {
                                    false
                                }
                            } else if let RegisterType::Bool(boola) = a {
                                if let RegisterType::Bool(boolb) = b {
                                    boola == boolb
                                } else {
                                    false
                                }
                            } else {
                                runtime_error_str("Comparison of invalid types", postion.clone());
                                unreachable!()
                            };

                            self.stack.push(RegisterType::Bool(success));
                        }
                        Internal::Larger | Internal::LargerEq | Internal::Smaller | Internal::SmallerEq => {
                            if self.stack.len() < 2 {
                                runtime_error_str("To few elements on stack", postion.clone());
                            }

                            let a = self.stack.pop().unwrap();
                            let b = self.stack.pop().unwrap();

                            let success = if let RegisterType::Int(inta) = a {
                                if let RegisterType::Int(intb) = b {
                                    match int {
                                        Internal::Larger => { inta > intb }
                                        Internal::Smaller => { inta < intb }
                                        Internal::LargerEq => { inta >= intb }
                                        Internal::SmallerEq => { inta <= intb }
                                        _ => {
                                            unreachable!()
                                        }
                                    }
                                } else {
                                    runtime_error_str("Comparison of invalid types", postion.clone());
                                    unreachable!();
                                }
                            } else {
                                runtime_error_str("Comparison of invalid types", postion.clone());
                                unreachable!();
                            };

                            self.stack.push(RegisterType::Bool(success));
                        }

                        _ => {
                            println!("Internal: {:?} not implemented yet", int)
                        }
                    }
                }
            }
            OperationType::Jump => {
                if let Operand::Jump(offset) = operation.operand.unwrap() {
                    let new_ip = self.ip + offset;
                    if new_ip > (self.ops.len() - 1) as i32 {
                        runtime_error_str("Jump outside of operations", postion);
                    }

                    self.ip = new_ip;
                }
            }
            _ => {
                println!("Operation: {:?} not implemented yet", operation)
            }
        }
    }
}