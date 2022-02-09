use std::any::Any;
use std::collections::HashMap;
use std::process::exit;

use crate::parser::{Function, State};
use crate::util::{compiler_error, compiler_error_str, runtime_error, runtime_error_str, runtime_warning, runtime_warning_str};
use crate::util::internals::Internal;
use crate::util::operation::{Operand, Operation, OperationType};
use crate::util::position::Position;
use crate::util::register_type::RegisterType;
use crate::vm;

pub struct VM {
    ip: i32,
    ops: HashMap<String, Function>,
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
        if !self.ops.contains_key("main") {
            runtime_error_str("Program does not contain a main function", Position::default());
        }

        let start = self.ops.get("main").unwrap().clone();

        self.execute_fn((&start, 0));

        if self.stack.len() != 1 {
            runtime_error_str("No return code provided", Position::default());
        }

        if let RegisterType::Int(exit_code) = self.stack.pop().unwrap() {
            exit(exit_code)
        } else {
            runtime_error_str("Return code can only be of type integer", Position::default());
        }
    }

    fn execute_fn(&mut self, fnc: (&Function, u8)) {
        for operation in &fnc.0.operations {
            self.execute_op(operation.clone(), fnc.1)
        }
    }

    fn execute_op(&mut self, op: (Position, Operation), depth: u8) {
        let position = op.0;
        let operation = op.1;
        if depth > 40 {
            runtime_error_str("Stack overflow", position.clone());
        }
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
            OperationType::PushFunction => {
                if let Operand::Str(fnc) = operation.operand.unwrap() {
                    self.stack.push(RegisterType::Function(fnc))
                }
            }
            OperationType::Internal => {
                if let Operand::Internal(int) = operation.operand.unwrap() {
                    match int {
                        Internal::NoOp => {}
                        Internal::Print | Internal::PrintLn => {
                            if self.stack.len() == 0 {
                                runtime_error_str("To few elements on stack", position.clone());
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
                                RegisterType::Function(name) => {
                                    print!("*{}()", name)
                                }
                                RegisterType::Empty => {
                                    runtime_error_str("Stack is empty", position.clone());
                                }
                            }

                            if int == Internal::PrintLn {
                                println!();
                            }
                        }
                        Internal::Swap => {
                            if self.stack.len() < 2 {
                                runtime_error_str("To few elements on stack", position.clone());
                            }

                            let a = self.stack.pop().unwrap();
                            let b = self.stack.pop().unwrap();
                            self.stack.push(a);
                            self.stack.push(b);
                        }
                        Internal::Drop => {
                            if self.stack.len() < 1 {
                                runtime_error_str("To few elements on stack", position.clone());
                            }
                            self.stack.pop().unwrap();
                        }
                        Internal::Dup => {
                            if self.stack.len() < 1 {
                                runtime_error_str("To few elements on stack", position.clone());
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
                        Internal::Plus => {
                            if self.stack.len() < 2 {
                                runtime_error_str("To few elements on stack", position.clone());
                            }

                            let top = self.stack.pop().unwrap();
                            let bottom = self.stack.pop().unwrap();

                            if let RegisterType::Int(top) = top {
                                if let RegisterType::Int(bottom) = bottom {
                                    self.stack.push(RegisterType::Int(top + bottom))
                                } else {
                                    runtime_error_str("Usage of invalid types", position.clone());
                                }
                            } else {
                                runtime_error_str("Usage of invalid types", position.clone());
                            }
                        }
                        Internal::Minus => {
                            if self.stack.len() < 2 {
                                runtime_error_str("To few elements on stack", position.clone());
                            }

                            let top = self.stack.pop().unwrap();
                            let bottom = self.stack.pop().unwrap();

                            if let RegisterType::Int(top) = top {
                                if let RegisterType::Int(bottom) = bottom {
                                    self.stack.push(RegisterType::Int(top - bottom))
                                } else {
                                    runtime_error_str("Usage of invalid types", position.clone());
                                }
                            } else {
                                runtime_error_str("Usage of invalid types", position.clone());
                            }
                        }
                        Internal::Mult => {
                            if self.stack.len() < 2 {
                                runtime_error_str("To few elements on stack", position.clone());
                            }

                            let top = self.stack.pop().unwrap();
                            let bottom = self.stack.pop().unwrap();

                            if let RegisterType::Int(top) = top {
                                if let RegisterType::Int(bottom) = bottom {
                                    self.stack.push(RegisterType::Int(top * bottom))
                                } else {
                                    runtime_error_str("Usage of invalid types", position.clone());
                                }
                            } else {
                                runtime_error_str("Usage of invalid types", position.clone());
                            }
                        }
                        Internal::Div => {
                            if self.stack.len() < 2 {
                                runtime_error_str("To few elements on stack", position.clone());
                            }

                            let top = self.stack.pop().unwrap();
                            let bottom = self.stack.pop().unwrap();

                            if let RegisterType::Int(top) = top {
                                if let RegisterType::Int(bottom) = bottom {
                                    if bottom == 0 {
                                        runtime_error_str("Divison by 0 is undefined", position.clone());
                                    }

                                    self.stack.push(RegisterType::Int(top / bottom))
                                } else {
                                    runtime_error_str("Usage of invalid types", position.clone());
                                }
                            } else {
                                runtime_error_str("Usage of invalid types", position.clone());
                            }
                        }
                        Internal::Modulo => {
                            if self.stack.len() < 2 {
                                runtime_error_str("To few elements on stack", position.clone());
                            }

                            let top = self.stack.pop().unwrap();
                            let bottom = self.stack.pop().unwrap();

                            if let RegisterType::Int(top) = top {
                                if let RegisterType::Int(bottom) = bottom {
                                    self.stack.push(RegisterType::Int(top % bottom))
                                } else {
                                    runtime_error_str("Usage of invalid types", position.clone());
                                }
                            } else {
                                runtime_error_str("Usage of invalid types", position.clone());
                            }
                        }
                        Internal::Squared => {
                            if self.stack.len() < 1 {
                                runtime_error_str("To few elements on stack", position.clone());
                            }

                            let top = self.stack.pop().unwrap();

                            if let RegisterType::Int(top) = top {
                                self.stack.push(RegisterType::Int(top * top))
                            } else {
                                runtime_error_str("Usage of invalid types", position.clone());
                            }
                        }
                        Internal::Cubed => {
                            if self.stack.len() < 1 {
                                runtime_error_str("To few elements on stack", position.clone());
                            }

                            let top = self.stack.pop().unwrap();

                            if let RegisterType::Int(top) = top {
                                self.stack.push(RegisterType::Int(top * top * top))
                            } else {
                                runtime_error_str("Usage of invalid types", position.clone());
                            }
                        }
                        Internal::Equals => {
                            if self.stack.len() < 2 {
                                runtime_error_str("To few elements on stack", position.clone());
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
                                runtime_error_str("Comparison of invalid types", position.clone());
                                unreachable!()
                            };

                            self.stack.push(RegisterType::Bool(success));
                        }
                        Internal::Larger | Internal::LargerEq | Internal::Smaller | Internal::SmallerEq => {
                            if self.stack.len() < 2 {
                                runtime_error_str("To few elements on stack", position.clone());
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
                                    runtime_error_str("Comparison of invalid types", position.clone());
                                    unreachable!();
                                }
                            } else {
                                runtime_error_str("Comparison of invalid types", position.clone());
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
            OperationType::Call => {
                if self.stack.len() == 0 {
                    runtime_error_str("To few elements on stack", position.clone());
                }

                let top = self.stack.pop().unwrap();

                if let RegisterType::Function(fnc) = top {
                    if !self.ops.contains_key(&fnc) {
                        runtime_error(format!("Function: {} does not exist", fnc), position.clone());
                    }

                    let fnc = self.ops.get(&fnc).unwrap().clone();

                    for operation in fnc.operations {
                        self.execute_op(operation, depth + 1);
                    }
                }
            }
            OperationType::Jump => {
                if let Operand::Jump(_offset) = operation.operand.unwrap() {
                    runtime_warning_str("Jump operation not implemented yet", position.clone());
                }
            }
            _ => {
                runtime_warning(format!("Operation: {:?} not implemented yet", operation), position.clone())
            }
        }
    }
}