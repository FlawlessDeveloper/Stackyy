pub mod typecheck {
    use std::collections::HashMap;

    use crate::parser::Function;
    use crate::util::operation::{Operand, OperationData, OperationType};
    use crate::util::type_check::{ErrorTypes, TypeCheckError, Types};

    pub fn create_push_type_check() -> Box<dyn Fn(&OperationData, &HashMap<String, Function>, &mut Vec<Types>, bool) -> TypeCheckError> {
        Box::new(|data, _, stack, _| {
            if let OperationType::Push = data.typ {
                if data.operand.is_some() {
                    let value = data.clone().operand.unwrap();
                    let opt = match value {
                        Operand::Int(_) => {
                            Some(Types::Int)
                        }
                        Operand::Str(_) => {
                            Some(Types::String)
                        }
                        Operand::Bool(_) => {
                            Some(Types::Bool)
                        }
                        Operand::PushFunction(_, _, _) => {
                            Some(Types::Function)
                        }
                        _ => {
                            None
                        }
                    };
                    if opt.is_some() {
                        stack.push(opt.unwrap());
                        ErrorTypes::None.into()
                    } else {
                        ErrorTypes::WrongData.into()
                    }
                } else {
                    ErrorTypes::InvalidTypes.into()
                }
            } else {
                ErrorTypes::InvalidTypes.into()
            }
        })
    }
}

pub mod runtime {
    use crate::{compiler_error_str, VM};
    use crate::util::operation::{Operand, OperationData, OperationType};
    use crate::util::register_type::RegisterType;

    pub fn create_push() -> Box<dyn Fn(&OperationData, &mut VM)> {
        Box::new(|data, vm| {
            let info = &data.data;
            if let OperationType::Push = data.typ {
                if data.operand.is_some() {
                    let value = data.clone().operand.unwrap();
                    let opt = match value {
                        Operand::Int(val) => {
                            Some(RegisterType::Int(val))
                        }
                        Operand::Str(val) => {
                            Some(RegisterType::String(val))
                        }
                        Operand::Bool(val) => {
                            Some(RegisterType::Bool(val))
                        }
                        Operand::PushFunction(name, inp, outp) => {
                            Some(RegisterType::Function(name, inp, outp))
                        }
                        _ => {
                            None
                        }
                    };
                    if opt.is_some() {
                        vm.stack_mut().push(opt.unwrap())
                    }
                } else {
                    compiler_error_str("Could not create closure for operation", info);
                }
            } else {
                compiler_error_str("Could not create closure for operation", info);
            }
        })
    }
}