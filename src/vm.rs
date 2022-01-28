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
        ops.reverse();
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
                if let Operand::OpAddr(op) = operation.operand.unwrap() {
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
                            let pop = self.stack.pop();
                            if let Some(reg) = pop {
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
                            } else {
                                runtime_error_str("Stack is empty", postion.clone());
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
                    }
                }
            }
            OperationType::Include => {
                if let Operand::Include(str) = operation.operand.unwrap() {
                    self.ops.push(str.as_ref().clone())
                }
            }
        }
    }
}