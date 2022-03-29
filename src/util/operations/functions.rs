pub mod typecheck {
    use std::collections::HashMap;

    use crate::parser::Function;
    use crate::util::operation::{Operand, OperationData};
    use crate::util::type_check::{ErrorTypes, TypeCheckError, Types};

    pub fn create_calling_type_check() -> Box<dyn Fn(&OperationData, &HashMap<String, Function>, &mut Vec<Types>, bool) -> TypeCheckError> {
        Box::new(|data, fns, stack, compile_time| {
            let (success, inp, outp): (TypeCheckError, Vec<Types>, Vec<Types>) = if let Some(inner) = data.operand.clone() {
                match inner {
                    Operand::Call(ref name) | Operand::PushFunction(ref name, _, _) => {
                        if fns.contains_key(name) {
                            let (inp, outp) = if let Operand::PushFunction(_, inp, outp) = inner {
                                (inp, outp)
                            } else {
                                fns.get(name).unwrap().get_contract()
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
                        if inp.len() == 0 {
                            true
                        } else {
                            let mut tmp_inp = inp.clone();
                            let mut tmp_stack = stack.clone();
                            if inp.len() <= tmp_stack.len() {
                                let mut success = true;
                                while tmp_inp.len() > 0 {
                                    let a = tmp_inp.pop().unwrap();
                                    let b = tmp_stack.pop().unwrap();
                                    if a != b {
                                        success = false;
                                        break;
                                    }
                                }
                                success
                            } else {
                                false
                            }
                        }
                    };

                    if !success {
                        ErrorTypes::InvalidTypes.into_with_ctx(inp, stack.clone())
                    } else {
                        if compile_time {
                            for _ in 0..inp.len() {
                                stack.pop().unwrap();
                            }
                            for typ in outp {
                                stack.push(typ.clone());
                            }
                        }
                        ErrorTypes::None.into()
                    }
                } else {
                    ErrorTypes::TooFewElements.into_with_ctx_plus(vec![Types::FunctionPointer(vec![Types::Any], vec![Types::Any])], stack.clone(), "Function pointer can contain any amount/sort of types")
                }
            } else {
                success
            }
        })
    }
}

pub mod runtime {
    use crate::util::{runtime_error, runtime_error_str};
    use crate::util::operation::{Operand, OperationData};
    use crate::util::register_type::RegisterType;
    use crate::util::type_check::Types;
    use crate::VM;

    pub fn create_fn() -> Box<dyn Fn(&OperationData, &mut VM)> {
        Box::new(|data, vm| {
            let info = &data.data;

            #[derive(Ord, PartialOrd, Eq, PartialEq)]
            enum CallEnum {
                None,
                SomeInline(String, Vec<Types>, Vec<Types>),
                SomeDynamic(String, Vec<Types>, Vec<Types>),
            }

            let top = {
                if let Some(operand) = data.clone().operand {
                    if let Operand::Call(fnc) = operand {
                        let (inp, outp) = vm.ops().get(&fnc).unwrap().get_contract();
                        CallEnum::SomeInline(fnc, inp, outp)
                    } else {
                        CallEnum::None
                    }
                } else {
                    if vm.stack().len() != 0 {
                        let last = vm.stack().last().unwrap().clone();
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
                    let top = vm.stack_mut().pop().unwrap();
                    if let RegisterType::Function(fnc, inp, outp) = top {
                        (fnc, inp, outp)
                    } else {
                        unreachable!()
                    }
                };

                if !vm.ops().contains_key(&fnc) {
                    runtime_error(format!("Function: {} does not exist", fnc), info);
                }

                let fnc = vm.ops().get(&fnc).unwrap().clone();
                if (inp, outp) == fnc.get_contract() {
                    vm.execute_fn(&fnc);
                } else {
                    runtime_error_str("Typecheck for dynamic function call failed", info);
                }
            } else {
                runtime_error_str(
                    "Invalid function call",
                    info,
                );
            }
        })
    }
}

