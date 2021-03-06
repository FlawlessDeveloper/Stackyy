pub mod typecheck {
    use std::collections::HashMap;

    use crate::parser::Function;
    use crate::util::internals::Internal;
    use crate::util::operation::{Operand, OperationData};
    use crate::util::type_check::{ErrorTypes, TypeCheckError, Types};

    pub fn get_internal_typecheck() -> Box<dyn Fn(&OperationData, &HashMap<String, Function>, &mut Vec<Types>, bool) -> TypeCheckError> {
        Box::new(move |data, fncs, stack, compile_time| {
            if let Operand::Internal(internal) = &data.operand.as_ref().unwrap() {
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
                            let last = if internal == &Internal::Not {
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
                    Internal::ToString => {
                        if stack.len() == 0 {
                            ErrorTypes::TooFewElements.into_with_ctx(vec![Types::Any], tmp_stack)
                        } else {
                            let _ = stack.pop().unwrap();
                            stack.push(Types::String);
                            ErrorTypes::None.into()
                        }
                    }
                }
            } else {
                ErrorTypes::ClosureError.into()
            }
        })
    }
}

pub mod runtime {
    use std::collections::HashMap;
    use std::io::{stdout, Write};

    use crate::{Position, VM};
    use crate::util::internals::Internal;
    use crate::util::operation::{Operand, OperationData, OperationDataInfo};
    use crate::util::operations::DescriptorAction;
    use crate::util::register_type::RegisterType;
    use crate::util::runtime_error_str;

    fn noop(internal: Internal, stack: &mut Vec<RegisterType>, info: &OperationDataInfo) {}

    fn print(internal: Internal, stack: &mut Vec<RegisterType>, info: &OperationDataInfo) {
        to_string(internal, stack, info);
        if let RegisterType::String(str) = stack.pop().unwrap() {
            if internal == Internal::PrintLn {
                println!("{str}");
            } else {
                print!("{str}");
                stdout().flush().unwrap();
            }
        }
    }

    fn to_string(internal: Internal, stack: &mut Vec<RegisterType>, info: &OperationDataInfo) {
        let reg = stack.pop().unwrap();
        reg.to_string_stacked(info, stack);
    }

    fn swap(internal: Internal, stack: &mut Vec<RegisterType>, info: &OperationDataInfo) {
        let a = stack.pop().unwrap();
        let b = stack.pop().unwrap();
        stack.push(a);
        stack.push(b);
    }

    fn drop(internal: Internal, stack: &mut Vec<RegisterType>, info: &OperationDataInfo) {
        if let RegisterType::Descriptor(descr) = stack.pop().unwrap() {
            let mut locked = descr.lock();
            let locked = locked.as_mut().unwrap();
            locked.action(DescriptorAction::Close, stack, &info);
        }
    }

    fn dup(internal: Internal, stack: &mut Vec<RegisterType>, info: &OperationDataInfo) {
        let top = stack.pop().unwrap();
        stack.push(top.clone());
        stack.push(top);
    }

    fn rev_stack(internal: Internal, stack: &mut Vec<RegisterType>, position: Position) {
        stack.reverse();
    }

    fn drop_stack(internal: Internal, stack: &mut Vec<RegisterType>, info: &OperationDataInfo) {
        stack.clear();
        stack.shrink_to_fit();
    }

    fn dup_stack(internal: Internal, stack: &mut Vec<RegisterType>, info: &OperationDataInfo) {
        let to_add = stack.clone();
        stack.extend(to_add);
    }

    fn dbg_stack(internal: Internal, stack: &mut Vec<RegisterType>, info: &OperationDataInfo) {
        for (index, item) in stack.iter().enumerate() {
            let str = if let Some(str) = item.to_string(info) {
                str
            } else {
                "No representation".to_string()
            };
            println!("[{}] -> {}", index, str)
        }
    }

    fn math(internal: Internal, stack: &mut Vec<RegisterType>, info: &OperationDataInfo) {
        let top = stack.pop().unwrap();
        if let RegisterType::Int(top) = top {
            match internal {
                Internal::Plus => {
                    let bottom = stack.pop().unwrap();

                    if let RegisterType::Int(bottom) = bottom {
                        stack.push(RegisterType::Int(top + bottom))
                    } else {
                        runtime_error_str("Usage of invalid types", info);
                    }
                }
                Internal::Minus => {
                    let bottom = stack.pop().unwrap();

                    if let RegisterType::Int(bottom) = bottom {
                        stack.push(RegisterType::Int(top - bottom))
                    } else {
                        runtime_error_str("Usage of invalid types", info);
                    }
                }
                Internal::Mult => {
                    let bottom = stack.pop().unwrap();

                    if let RegisterType::Int(bottom) = bottom {
                        stack.push(RegisterType::Int(top * bottom))
                    } else {
                        runtime_error_str("Usage of invalid types", info);
                    }
                }
                Internal::Div => {
                    let bottom = stack.pop().unwrap();

                    if let RegisterType::Int(bottom) = bottom {
                        if bottom == 0 {
                            runtime_error_str("Divison by 0 is undefined", info);
                        }
                        stack.push(RegisterType::Int(top / bottom))
                    } else {
                        runtime_error_str("Usage of invalid types", info);
                    }
                }
                Internal::Modulo => {
                    let bottom = stack.pop().unwrap();

                    if let RegisterType::Int(bottom) = bottom {
                        stack.push(RegisterType::Int(top % bottom))
                    } else {
                        runtime_error_str("Usage of invalid types", info);
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

    fn bool_ops(internal: Internal, stack: &mut Vec<RegisterType>, info: &OperationDataInfo) {
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
                    runtime_error_str("Comparison of invalid types", info);
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
                        runtime_error_str("Comparison of invalid types", info);
                        unreachable!();
                    }
                } else {
                    runtime_error_str("Comparison of invalid types", info);
                    unreachable!();
                };

                stack.push(RegisterType::Bool(success));
            }
            _ => {}
        }
    }

    fn reflection(internal: Internal, stack: &mut Vec<RegisterType>, info: &OperationDataInfo) {
        let fnc = stack.pop().unwrap();
        if let RegisterType::Function(mut str, inp, outp) = fnc {
            match internal {
                Internal::ReflectionRemoveStr | Internal::ReflectionRemoveStrDrop => {
                    let amount = stack.pop().unwrap();
                    let (str, mod_fnc) = {
                        if let RegisterType::Int(val) = amount {
                            if str.len() == 0 {
                                runtime_error_str("Cannot remove string from empty function name", info);
                                unreachable!();
                            }
                            if val > str.len() as i32 {
                                runtime_error_str("Tried to remove too much from function name", info);
                                unreachable!();
                            }

                            let mut str_add = String::new();

                            for _ in 0..val {
                                let char = str.pop();
                                if let Some(char) = char {
                                    str_add.push(char);
                                } else {
                                    runtime_error_str("Tried to remove too much from function name", info);
                                    unreachable!();
                                }
                            }


                            (str_add, RegisterType::Function(str, inp, outp))
                        } else {
                            runtime_error_str("Comparison of invalid types", info);
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
                                runtime_error_str("Can not push non string to function", info);
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


    pub fn get_internal_executor() -> Box<dyn Fn(&OperationData, &mut VM)> {
        Box::new(move |op, vm| {
            if let Operand::Internal(internal) = &op.operand.as_ref().unwrap() {
                let internal = internal.clone();
                let info = &op.data;
                match internal {
                    Internal::NoOp => noop(internal, vm.stack_mut(), info),
                    Internal::Print | Internal::PrintLn => print(internal, vm.stack_mut(), info),
                    Internal::Swap => swap(internal, vm.stack_mut(), info),
                    Internal::Drop => drop(internal, vm.stack_mut(), info),
                    Internal::Dup => dup(internal, vm.stack_mut(), info),
                    Internal::RevStack => dup_stack(internal, vm.stack_mut(), info),
                    Internal::DropStack => drop_stack(internal, vm.stack_mut(), info),
                    Internal::DupStack => dup_stack(internal, vm.stack_mut(), info),
                    Internal::DbgStack => dbg_stack(internal, vm.stack_mut(), info),
                    Internal::Plus | Internal::Minus | Internal::Mult | Internal::Div | Internal::Modulo | Internal::Squared | Internal::Cubed => math(internal, vm.stack_mut(), info),
                    Internal::Not | Internal::NotPeek | Internal::Equals | Internal::Larger | Internal::LargerEq | Internal::Smaller | Internal::SmallerEq => bool_ops(internal, vm.stack_mut(), info),
                    Internal::ReflectionRemoveStr | Internal::ReflectionRemoveStrDrop | Internal::ReflectionPush | Internal::ReflectionClear => reflection(internal, vm.stack_mut(), info),
                    _ => {
                        println!("Internal: {:?} not implemented yet", internal)
                    }
                }
            }
        })
    }
}