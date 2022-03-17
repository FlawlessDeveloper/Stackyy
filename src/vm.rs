use std::collections::HashMap;
use std::process::exit;

use crate::parser::{Function, State};
use crate::util::{compiler_error_str, runtime_error, runtime_error_str, runtime_warning, runtime_warning_str};
use crate::util::operation::{Operation, OperationData, OperationType};
use crate::util::position::Position;
use crate::util::register_type::RegisterType;
use crate::util::type_check::{ErrorTypes, Types};

pub const MAX_CALL_STACK_SIZE: u8 = 40;

pub struct VM {
    ip: i32,
    ops: HashMap<String, Function>,
    stack: Vec<RegisterType>,
    type_stack: Vec<Types>,
    last_op: Option<(Position, OperationData)>,
    depth: u8,
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
        let mut ops = state.get_ops().clone();
        Self {
            ip: 0,
            ops,
            stack: vec![],
            type_stack: vec![],
            last_op: None,
            depth: 0,
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

        self.execute_fn(&start);

        if self.stack.len() != 1 {
            runtime_error_str("No return code provided", Position::default());
        }

        if let RegisterType::Int(exit_code) = self.stack.pop().unwrap() {
            exit(exit_code)
        } else {
            runtime_error_str("Return code can only be of type integer", Position::default());
        }
    }

    pub fn execute_fn(&mut self, fnc: &Function) {
        self.depth += 1;
        for operation in &fnc.operations {
            self.execute_op(operation, fnc.name())
        }
        self.depth -= 1;
    }

    pub fn stack(&self) -> &Vec<RegisterType> {
        &self.stack
    }

    pub fn stack_mut(&mut self) -> &mut Vec<RegisterType> {
        &mut self.stack
    }

    fn execute_op(&mut self, op: &(Position, Operation), fn_name: String) {
        let position = op.0.clone();
        let data = op.1.data();
        let typecheck = &op.1.type_check;
        let exec = &op.1.execute_fn;
        if self.depth > MAX_CALL_STACK_SIZE {
            runtime_error_str("Stack overflow", position.clone());
        }

        if self.type_stack.len() != self.stack.len() {
            runtime_error(format!("Typecheck desync happened. Responsible operation: {:#?}", self.last_op.clone().unwrap()), position.clone());
        }

        let tc_error = (typecheck(data, &self.ops, &mut self.type_stack, false)).is_error();

        if !tc_error {
            exec(data, self);
        } else {
            runtime_error(format!("Function {} failed type check ", fn_name), position.clone());
        };

        self.last_op = Some((position, data.clone()))
    }

    /*
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
                    if let Operand::PushFunction(fnc, inp, outp) = operation.operand.unwrap() {
                        self.stack.push(RegisterType::Function(fnc, inp, outp))
                    }
                }
                OperationType::Internal => {
                    if let Operand::Internal(int) = operation.operand.unwrap() {
                        match int {
                            Internal::NoOp => {}
                            Internal::Print | Internal::PrintLn => {
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
                                    RegisterType::Function(name, ..) => {
                                        print!("*{}()", name)
                                    }
                                    RegisterType::Empty => {
                                        runtime_error_str("Stack is empty", position.clone());
                                    }
                                }

                                if int == Internal::PrintLn {
                                    println!();
                                } else {
                                    stdout().flush().unwrap();
                                }
                            }
                            Internal::Swap => {
                                let a = self.stack.pop().unwrap();
                                let b = self.stack.pop().unwrap();
                                self.stack.push(a);
                                self.stack.push(b);
                            }
                            Internal::Drop => {
                                self.stack.pop().unwrap();
                            }
                            Internal::Dup => {
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
                            Internal::Squared | Internal::Cubed => {
                                let top = self.stack.pop().unwrap();
                                if let RegisterType::Int(top) = top {
                                    self.stack.push(RegisterType::Int(if int == Internal::Cubed {
                                        top * top * top
                                    } else {
                                        top * top
                                    }))
                                } else {
                                    runtime_error_str("Usage of invalid types", position.clone());
                                }
                            }
                            Internal::Not => {
                                let top = self.stack.pop().unwrap();

                                if let RegisterType::Bool(top) = top {
                                    self.stack.push(RegisterType::Bool(!top))
                                } else {
                                    runtime_error_str("Usage of invalid types", position.clone());
                                }
                            }
                            Internal::NotPeek => {
                                let top = self.stack.last().unwrap();

                                if let RegisterType::Bool(top) = top {
                                    self.stack.push(RegisterType::Bool(!top.clone()))
                                } else {
                                    runtime_error_str("Usage of invalid types", position.clone());
                                }
                            }
                            Internal::Equals => {
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

                            Internal::ReflectionRemoveStr | Internal::ReflectionRemoveStrDrop => {
                                let fnc = self.stack.pop().unwrap();
                                let amount = self.stack.pop().unwrap();
                                let (str, mod_fnc) = if let RegisterType::Function(mut str, inp, outp) = fnc {
                                    if let RegisterType::Int(val) = amount {
                                        if str.len() == 0 {
                                            runtime_error_str("Cannot remove string from empty function name", position.clone());
                                            unreachable!();
                                        }
                                        if val > str.len() as i32 {
                                            runtime_error_str("Tried to remove too much from function name", position.clone());
                                            unreachable!();
                                        }

                                        let mut str_add = String::new();

                                        for _ in 0..val {
                                            let char = str.pop();
                                            if let Some(char) = char {
                                                str_add.push(char);
                                            } else {
                                                runtime_error_str("Tried to remove too much from function name", position.clone());
                                                unreachable!();
                                            }
                                        }


                                        (str_add, RegisterType::Function(str, inp, outp))
                                    } else {
                                        runtime_error_str("Comparison of invalid types", position.clone());
                                        unreachable!();
                                    }
                                } else {
                                    runtime_error_str("Comparison of invalid types", position.clone());
                                    unreachable!();
                                };
                                if int != Internal::ReflectionRemoveStrDrop {
                                    self.stack.push(RegisterType::String(str));
                                }
                                self.stack.push(mod_fnc);
                            }
                            Internal::ReflectionPush => {
                                let fnc = self.stack.pop().unwrap();
                                let string = self.stack.pop().unwrap();
                                let mod_fnc = if let RegisterType::Function(mut str, inp, outp) = fnc {
                                    if let RegisterType::String(val) = string {
                                        str.push_str(&val);
                                        RegisterType::Function(str, inp, outp)
                                    } else {
                                        runtime_error_str("Comparison of invalid types", position.clone());
                                        unreachable!();
                                    }
                                } else {
                                    runtime_error_str("Comparison of invalid types", position.clone());
                                    unreachable!();
                                };

                                self.stack.push(mod_fnc)
                            }
                            Internal::ReflectionClear => {
                                let fnc = self.stack.pop().unwrap();
                                let mod_fnc = if let RegisterType::Function(_, inp, outp) = fnc {
                                    RegisterType::Function(String::new(), inp, outp)
                                } else {
                                    runtime_error_str("Comparison of invalid types", position.clone());
                                    unreachable!();
                                };
                                self.stack.push(mod_fnc);
                            }

                            _ => {
                                println!("Internal: {:?} not implemented yet", int)
                            }
                        }
                    }
                }
                OperationType::Call => {
                    #[derive(Ord, PartialOrd, Eq, PartialEq)]
                    enum CallEnum {
                        None,
                        SomeInline(String, Vec<Types>, Vec<Types>),
                        SomeDynamic(String, Vec<Types>, Vec<Types>),
                    }

                    let top = {
                        if let Some(operand) = operation.operand {
                            if let Operand::Call(fnc) = operand {
                                let (inp, outp) = self.ops.get(&fnc).unwrap().get_contract();
                                CallEnum::SomeInline(fnc, inp, outp)
                            } else {
                                CallEnum::None
                            }
                        } else {
                            if self.stack.len() != 0 {
                                let last = self.stack.last().unwrap().clone();
                                if let RegisterType::Function(fnc, inp, outp) = last.clone() {
                                    CallEnum::SomeDynamic(fnc, inp, outp)
                                } else {
                                    CallEnum::None
                                }
                            } else {
                                CallEnum::None
                            }
                        }
                    };

                    if top != CallEnum::None {
                        let (fnc, inp, outp) = if let CallEnum::SomeInline(fnc, inp, outp) = top {
                            (fnc, inp, outp)
                        } else {
                            let top = self.stack.pop().unwrap();
                            if let RegisterType::Function(fnc, inp, outp) = top {
                                (fnc, inp, outp)
                            } else {
                                unreachable!()
                            }
                        };

                        if !self.ops.contains_key(&fnc) {
                            runtime_error(format!("Function: {} does not exist", fnc), position.clone());
                        }

                        let fnc_str = fnc.clone();
                        let fnc = self.ops.get(&fnc).unwrap().clone();
                        if (inp, outp) == fnc.get_contract() {
                            for operation in fnc.operations {
                                self.execute_op(operation, depth + 1, fnc_str.clone());
                            }
                        } else {
                            runtime_error_str("Typecheck for dynamic function call failed", position.clone());
                        }
                    } else {
                        runtime_error_str("Invalid function call", position.clone());
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


        self.last_op = Some(op);
     */
    pub fn ops(&self) -> &HashMap<String, Function> {
        &self.ops
    }
}