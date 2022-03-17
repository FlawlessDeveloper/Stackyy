pub mod typecheck {
    use std::collections::HashMap;

    use crate::parser::Function;
    use crate::util::internals::Internal;
    use crate::util::operation::OperationData;
    use crate::util::type_check::{ErrorTypes, TypeCheckError, Types};

    pub fn get_internal_typecheck(internal: Internal) -> Box<dyn Fn(&OperationData, &HashMap<String, Function>, &mut Vec<Types>, bool) -> TypeCheckError> {
        Box::new(move |data, fncs, stack, compile_time| {
            let internal = internal;
            let tmp_stack = stack.clone();

            match internal {
                Internal::NoOp | Internal::DbgStack => {
                    ErrorTypes::None.into()
                }
                Internal::Print | Internal::PrintLn => {
                    if stack.len() == 0 {
                        ErrorTypes::TooFewElements.into_with_ctx(vec![Types::Any, Types::Any], tmp_stack)
                    } else {
                        stack.pop();
                        ErrorTypes::None.into()
                    }
                }
                Internal::Swap => {
                    if stack.len() < 2 {
                        ErrorTypes::TooFewElements.into_with_ctx(vec![Types::Any, Types::Any], tmp_stack)
                    } else {
                        let a = stack.pop().unwrap();
                        let b = stack.pop().unwrap();
                        stack.push(a);
                        stack.push(b);
                        ErrorTypes::None.into()
                    }
                }
                Internal::Drop => {
                    if stack.len() == 0 {
                        ErrorTypes::TooFewElements.into_with_ctx(vec![Types::Any], tmp_stack)
                    } else {
                        stack.pop();
                        ErrorTypes::None.into()
                    }
                }
                Internal::Dup => {
                    if stack.len() == 0 {
                        ErrorTypes::TooFewElements.into_with_ctx(vec![Types::Any], tmp_stack)
                    } else {
                        let last = stack.last().unwrap().clone();
                        stack.push(last);
                        ErrorTypes::None.into()
                    }
                }
                Internal::RevStack => {
                    stack.reverse();
                    ErrorTypes::None.into()
                }
                Internal::DropStack => {
                    stack.clear();
                    ErrorTypes::None.into()
                }
                Internal::DupStack => {
                    stack.extend(stack.clone());
                    ErrorTypes::None.into()
                }
                Internal::Plus | Internal::Minus | Internal::Mult | Internal::Div | Internal::Modulo => {
                    if stack.len() < 2 {
                        ErrorTypes::TooFewElements.into_with_ctx(vec![Types::Int, Types::Int], tmp_stack)
                    } else {
                        let a = stack.pop().unwrap();
                        let b = stack.pop().unwrap();

                        if a != Types::Int || b != Types::Int {
                            ErrorTypes::InvalidTypes.into_with_ctx(vec![Types::Int, Types::Int], tmp_stack)
                        } else {
                            stack.push(Types::Int);
                            ErrorTypes::None.into()
                        }
                    }
                }
                Internal::Squared | Internal::Cubed => {
                    if stack.len() == 0 {
                        ErrorTypes::TooFewElements.into_with_ctx(vec![Types::Int], tmp_stack)
                    } else {
                        let last = stack.last().unwrap().clone();
                        if last == Types::Int {
                            ErrorTypes::None.into()
                        } else {
                            ErrorTypes::InvalidTypes.into_with_ctx(vec![Types::Int, Types::Int], tmp_stack)
                        }
                    }
                }
                Internal::Not | Internal::NotPeek => {
                    if stack.len() == 0 {
                        ErrorTypes::TooFewElements.into_with_ctx(vec![Types::Int], tmp_stack)
                    } else {
                        let last = if internal == Internal::Not {
                            stack.pop().unwrap().clone()
                        } else {
                            stack.last().unwrap().clone()
                        };
                        if last == Types::Bool {
                            stack.push(Types::Bool);
                            ErrorTypes::None.into()
                        } else {
                            ErrorTypes::InvalidTypes.into_with_ctx(vec![Types::Bool], tmp_stack)
                        }
                    }
                }
                Internal::Equals | Internal::Larger | Internal::Smaller | Internal::LargerEq | Internal::SmallerEq => {
                    const ALLOWED_TYPES: [Types; 3] = [Types::Int, Types::String, Types::Bool];

                    if stack.len() < 2 {
                        ErrorTypes::TooFewElements.into_with_ctx_plus(vec![Types::Any, Types::Any], tmp_stack, "There are only int, string, bool allowed")
                    } else {
                        let a = stack.pop().unwrap();
                        let b = stack.pop().unwrap();

                        if ALLOWED_TYPES.contains(&a) && ALLOWED_TYPES.contains(&b) {
                            if a == b {
                                stack.push(Types::Bool);
                                ErrorTypes::None.into()
                            } else {
                                ErrorTypes::InvalidTypes.into_with_ctx(vec![a.clone(), a.clone()], stack.clone())
                            }
                        } else {
                            ErrorTypes::InvalidTypes.into_with_ctx_plus(vec![Types::Any, Types::Any], tmp_stack, "There are only int, string, bool allowed")
                        }
                    }
                }
                Internal::ReflectionRemoveStr => {
                    if stack.len() < 2 {
                        ErrorTypes::TooFewElements.into_with_ctx(vec![Types::Int, Types::FunctionPointer(vec![Types::Any], vec![Types::Any])], tmp_stack)
                    } else {
                        let fnc = stack.pop().unwrap();

                        if let Types::FunctionPointer(_, _) = fnc {
                            let amount = stack.pop().unwrap();
                            if amount == Types::Int {
                                stack.push(Types::String);
                                stack.push(fnc.clone());
                                ErrorTypes::None.into()
                            } else {
                                ErrorTypes::InvalidTypes.into_with_ctx(vec![Types::Int, Types::FunctionPointer(vec![Types::Any], vec![Types::Any])], tmp_stack)
                            }
                        } else {
                            ErrorTypes::InvalidTypes.into_with_ctx(vec![Types::Int, Types::FunctionPointer(vec![Types::Any], vec![Types::Any])], tmp_stack)
                        }
                    }
                }
                Internal::ReflectionRemoveStrDrop => {
                    if stack.len() < 2 {
                        ErrorTypes::TooFewElements.into_with_ctx(vec![Types::Int, Types::FunctionPointer(vec![Types::Any], vec![Types::Any])], tmp_stack)
                    } else {
                        let fnc = stack.pop().unwrap();

                        if let Types::FunctionPointer(_, _) = fnc {
                            let amount = stack.pop().unwrap();
                            if amount == Types::Int {
                                stack.push(fnc.clone());
                                ErrorTypes::None.into()
                            } else {
                                ErrorTypes::InvalidTypes.into_with_ctx(vec![Types::Int, Types::FunctionPointer(vec![Types::Any], vec![Types::Any])], tmp_stack)
                            }
                        } else {
                            ErrorTypes::InvalidTypes.into_with_ctx(vec![Types::Int, Types::FunctionPointer(vec![Types::Any], vec![Types::Any])], tmp_stack)
                        }
                    }
                }
                Internal::ReflectionPush => {
                    if stack.len() < 2 {
                        ErrorTypes::TooFewElements.into_with_ctx(vec![Types::String, Types::FunctionPointer(vec![Types::Any], vec![Types::Any])], tmp_stack)
                    } else {
                        let fnc = stack.pop().unwrap();
                        let str = stack.pop().unwrap();

                        if let Types::FunctionPointer(_, _) = fnc {
                            if str == Types::String {
                                stack.push(fnc.clone());
                                ErrorTypes::None.into()
                            } else {
                                ErrorTypes::InvalidTypes.into_with_ctx(vec![Types::String, Types::FunctionPointer(vec![Types::Any], vec![Types::Any])], tmp_stack)
                            }
                        } else {
                            ErrorTypes::InvalidTypes.into_with_ctx(vec![Types::String, Types::FunctionPointer(vec![Types::Any], vec![Types::Any])], tmp_stack)
                        }
                    }
                }
                Internal::ReflectionClear => {
                    if stack.len() == 0 {
                        ErrorTypes::TooFewElements.into_with_ctx(vec![Types::FunctionPointer(vec![Types::Any], vec![Types::Any])], tmp_stack)
                    } else {
                        let fnc = stack.pop().unwrap();

                        if let Types::FunctionPointer(_, _) = fnc {
                            let amount = stack.pop().unwrap();
                            if amount == Types::String {
                                stack.push(fnc.clone());
                                ErrorTypes::None.into()
                            } else {
                                ErrorTypes::InvalidTypes.into_with_ctx(vec![Types::FunctionPointer(vec![Types::Any], vec![Types::Any])], tmp_stack)
                            }
                        } else {
                            ErrorTypes::InvalidTypes.into_with_ctx(vec![Types::FunctionPointer(vec![Types::Any], vec![Types::Any])], tmp_stack)
                        }
                    }
                }
            }
        })
    }
}

pub mod runtime {
    use std::collections::HashMap;
    use std::io::{stdout, Write};

    use crate::{Position, VM};
    use crate::util::internals::Internal;
    use crate::util::operation::OperationData;
    use crate::util::register_type::RegisterType;
    use crate::util::runtime_error_str;

    fn noop(internal: Internal, stack: &mut Vec<RegisterType>, position: Position) {}

    fn print(internal: Internal, stack: &mut Vec<RegisterType>, position: Position) {
        let reg = stack.pop().unwrap();
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

        if internal == Internal::PrintLn {
            println!();
        } else {
            stdout().flush().unwrap();
        }
    }

    fn swap(internal: Internal, stack: &mut Vec<RegisterType>, position: Position) {
        let a = stack.pop().unwrap();
        let b = stack.pop().unwrap();
        stack.push(a);
        stack.push(b);
    }

    fn drop(internal: Internal, stack: &mut Vec<RegisterType>, position: Position) {
        stack.pop().unwrap();
    }

    fn dup(internal: Internal, stack: &mut Vec<RegisterType>, position: Position) {
        let top = stack.pop().unwrap();
        stack.push(top.clone());
        stack.push(top);
    }

    fn rev_stack(internal: Internal, stack: &mut Vec<RegisterType>, position: Position) {
        stack.reverse();
    }

    fn drop_stack(internal: Internal, stack: &mut Vec<RegisterType>, position: Position) {
        stack.clear();
        stack.shrink_to_fit();
    }

    fn dup_stack(internal: Internal, stack: &mut Vec<RegisterType>, position: Position) {
        let to_add = stack.clone();
        stack.extend(to_add);
    }

    fn dbg_stack(internal: Internal, stack: &mut Vec<RegisterType>, position: Position) {
        println!("{:#?}", stack);
    }

    fn math(internal: Internal, stack: &mut Vec<RegisterType>, position: Position) {
        let top = stack.pop().unwrap();
        if let RegisterType::Int(top) = top {
            match internal {
                Internal::Plus => {
                    let bottom = stack.pop().unwrap();

                    if let RegisterType::Int(bottom) = bottom {
                        stack.push(RegisterType::Int(top + bottom))
                    } else {
                        runtime_error_str("Usage of invalid types", position.clone());
                    }
                }
                Internal::Minus => {
                    let bottom = stack.pop().unwrap();

                    if let RegisterType::Int(bottom) = bottom {
                        stack.push(RegisterType::Int(top - bottom))
                    } else {
                        runtime_error_str("Usage of invalid types", position.clone());
                    }
                }
                Internal::Mult => {
                    let bottom = stack.pop().unwrap();

                    if let RegisterType::Int(bottom) = bottom {
                        stack.push(RegisterType::Int(top * bottom))
                    } else {
                        runtime_error_str("Usage of invalid types", position.clone());
                    }
                }
                Internal::Div => {
                    let bottom = stack.pop().unwrap();

                    if let RegisterType::Int(bottom) = bottom {
                        if bottom == 0 {
                            runtime_error_str("Divison by 0 is undefined", position.clone());
                        }
                        stack.push(RegisterType::Int(top / bottom))
                    } else {
                        runtime_error_str("Usage of invalid types", position.clone());
                    }
                }
                Internal::Modulo => {
                    let bottom = stack.pop().unwrap();

                    if let RegisterType::Int(bottom) = bottom {
                        stack.push(RegisterType::Int(top % bottom))
                    } else {
                        runtime_error_str("Usage of invalid types", position.clone());
                    }
                }
                Internal::Squared | Internal::Cubed => {
                    stack.push(RegisterType::Int(if internal == Internal::Cubed {
                        top * top * top
                    } else {
                        top * top
                    }))
                }
                _ => {}
            }
        }
    }

    fn bool_ops(internal: Internal, stack: &mut Vec<RegisterType>, position: Position) {
        match internal {
            Internal::Not | Internal::NotPeek => {
                let top = stack.pop().unwrap();
                if let RegisterType::Bool(bool) = top {
                    if internal != Internal::Not {
                        stack.push(top);
                    }
                    stack.push(RegisterType::Bool(!bool));
                }
            }
            Internal::Equals => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();

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

                stack.push(RegisterType::Bool(success));
            }
            Internal::Larger | Internal::LargerEq | Internal::Smaller | Internal::SmallerEq => {
                let a = stack.pop().unwrap();
                let b = stack.pop().unwrap();

                let success = if let RegisterType::Int(inta) = a {
                    if let RegisterType::Int(intb) = b {
                        match internal {
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

                stack.push(RegisterType::Bool(success));
            }
            _ => {}
        }
    }

    fn reflection(internal: Internal, stack: &mut Vec<RegisterType>, position: Position) {
        let fnc = stack.pop().unwrap();
        if let RegisterType::Function(mut str, inp, outp) = fnc {
            match internal {
                Internal::ReflectionRemoveStr | Internal::ReflectionRemoveStrDrop => {
                    let amount = stack.pop().unwrap();
                    let (str, mod_fnc) = {
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
                    };
                    if internal != Internal::ReflectionRemoveStrDrop {
                        stack.push(RegisterType::String(str));
                    }
                    stack.push(mod_fnc);
                }
                Internal::ReflectionPush | Internal::ReflectionClear => {
                    let string = stack.pop().unwrap();
                    let mod_fnc = {
                        if internal == Internal::ReflectionPush {
                            if let RegisterType::String(val) = string {
                                str.push_str(&val);
                                RegisterType::Function(str, inp, outp)
                            } else {
                                runtime_error_str("Can not push non string to function", position.clone());
                                unreachable!();
                            }
                        } else {
                            RegisterType::Function(String::new(), inp, outp)
                        }
                    };

                    stack.push(mod_fnc)
                }
                _ => {}
            }
        }
    }


    pub fn get_internal_executor(internal: Internal) -> Box<dyn Fn(&OperationData, &mut VM)> {
        Box::new(move |op, vm| {
            let internal = internal;
            let position = op.1.location().clone();
            match internal {
                Internal::NoOp => noop(internal, vm.stack_mut(), position),
                Internal::Print | Internal::PrintLn => print(internal, vm.stack_mut(), position),
                Internal::Swap => swap(internal, vm.stack_mut(), position),
                Internal::Drop => drop(internal, vm.stack_mut(), position),
                Internal::Dup => dup(internal, vm.stack_mut(), position),
                Internal::RevStack => dup_stack(internal, vm.stack_mut(), position),
                Internal::DropStack => drop_stack(internal, vm.stack_mut(), position),
                Internal::DupStack => dup_stack(internal, vm.stack_mut(), position),
                Internal::DbgStack => dbg_stack(internal, vm.stack_mut(), position),
                Internal::Plus | Internal::Minus | Internal::Mult | Internal::Div | Internal::Modulo | Internal::Squared | Internal::Cubed => math(internal, vm.stack_mut(), position),
                Internal::Not | Internal::NotPeek | Internal::Equals | Internal::Larger | Internal::LargerEq | Internal::Smaller | Internal::SmallerEq => bool_ops(internal, vm.stack_mut(), position),
                Internal::ReflectionRemoveStr | Internal::ReflectionRemoveStrDrop | Internal::ReflectionPush | Internal::ReflectionClear => reflection(internal, vm.stack_mut(), position),
                _ => {
                    println!("Internal: {:?} not implemented yet", internal)
                }
            }
        })
    }
}