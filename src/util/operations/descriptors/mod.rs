use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::lazy::SyncLazy;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::{compiler_error, compiler_error_str, Position, VM};
use crate::parser::Function;
use crate::util::{runtime_error, runtime_error_str, runtime_warning_str};
use crate::util::operation::{Operand, OperationData, OperationType};
use crate::util::operations::descriptors::file::File;
use crate::util::register_type::RegisterType;
use crate::util::type_check::{ErrorTypes, TypeCheckError, Types};

mod file;

const DESCRIPTORS_MAP: SyncLazy<HashMap<&'static str, DescriptorType>> = SyncLazy::new(|| {
    let mut map = HashMap::new();
    map.insert("file", DescriptorType::File);
    map
});

const DESCRIPTOR_ACTION_MAP: SyncLazy<HashMap<&'static str, DescriptorAction>> = SyncLazy::new(|| {
    let mut map = HashMap::new();
    map.insert("open", DescriptorAction::Open);
    map.insert("write-all", DescriptorAction::WriteAll);
    map.insert("read-all", DescriptorAction::ReadAll);
    map
});

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum DescriptorAction {
    // General Actions
    Open,
    ReadAll,
    WriteAll,
    ToString,
    Close,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum DescriptorType {
    File
}

pub trait Descriptor: Debug {
    fn action(&mut self, action: DescriptorAction, data: &mut Vec<RegisterType>);
    fn typecheck(&self, action: DescriptorAction, stack: &mut Vec<Types>) -> TypeCheckError;
    fn as_any(&self) -> &dyn Any;
}

fn map_or_error_desc_type(str: &str, is_runtime: bool) -> DescriptorType {
    if DESCRIPTORS_MAP.contains_key(str) {
        *DESCRIPTORS_MAP.get(str).unwrap()
    } else {
        let error_fn = if is_runtime { runtime_error } else { compiler_error };
        error_fn(format!("The descriptor {} is not registered", str), Position::default());
        unreachable!()
    }
}

fn map_or_error_desc_action(str: &str, is_runtime: bool) -> DescriptorAction {
    if DESCRIPTOR_ACTION_MAP.contains_key(str) {
        *DESCRIPTOR_ACTION_MAP.get(str).unwrap()
    } else {
        let error_fn = if is_runtime { runtime_error } else { compiler_error };
        error_fn(format!("The action {} is not registered", str), Position::default());
        unreachable!()
    }
}

fn get_descriptor_from_type(typ: DescriptorType) -> Box<dyn Descriptor> {
    Box::new(match typ {
        DescriptorType::File => {
            File::new()
        }
    })
}

fn descriptor_typecheck(stack: &mut Vec<Types>, typ: DescriptorType, action: DescriptorAction) -> TypeCheckError {
    let (inp, outp) = get_descriptor_contract(typ, action);
    let mut iter = inp.clone();
    let tmp_stack = stack.clone();
    for rem in iter {
        if stack.len() == 0 {
            return ErrorTypes::TooFewElements.into_with_ctx(inp, vec![]);
        }
        let pop = stack.pop().unwrap();
        if rem != pop {
            return ErrorTypes::InvalidTypes.into_with_ctx(inp, tmp_stack);
        }
    }

    if stack.len() == 0 {
        let mut tmp_inp = inp;
        tmp_inp.push(Types::Descriptor);
        return ErrorTypes::TooFewElements.into_with_ctx(tmp_inp, vec![]);
    }

    let pop = stack.pop().unwrap();

    if pop != Types::Descriptor {
        let mut tmp_inp = inp;
        tmp_inp.push(Types::Descriptor);
        return ErrorTypes::TooFewElements.into_with_ctx(tmp_inp, vec![]);
    }

    for out in outp {
        stack.push(out);
    }

    stack.push(Types::Descriptor);

    ErrorTypes::None.into()
}

fn get_descriptor_contract(typ: DescriptorType, action: DescriptorAction) -> (Vec<Types>, Vec<Types>) {
    match action {
        DescriptorAction::Open => unreachable!(),
        DescriptorAction::ReadAll => (vec![], vec![Types::String]),
        DescriptorAction::WriteAll => (vec![Types::String], vec![]),
        DescriptorAction::ToString => (vec![Types::String], vec![]),
        DescriptorAction::Close => unreachable!(),
        _ => match typ {
            _ => unreachable!()
        }
    }
}


/*

How resources should work

Write Action

SD      | Writeline 1
S       | Writeline 2
        | Writeline 3
D       | Writeline 4

Read Action

D       | Readline 1
        | Readline 2
S       | Readline 3
SD      | Readline 4

 */

/*
1. Move all necessary items to tmp stack
2. Pop descriptor
3. Move all necessary items back to stack
4. Call descriptor action
5. Push descriptor
 */

pub fn execute_fn() -> Box<dyn Fn(&OperationData, &mut VM)> {
    Box::new(|op_data, vm| {
        if let OperationType::Descriptor = op_data.0 {
            if let Operand::DescriptorAction(descr, action) = op_data.clone().2.unwrap() {
                let typ = map_or_error_desc_type(&descr, true);
                let action = map_or_error_desc_action(&action, true);

                if action == DescriptorAction::Close {
                    runtime_warning_str("Closing descriptor is not allowed via a descriptor action", op_data.1.location().clone());
                }


                if action == DescriptorAction::Open {
                    let mut descr: Box<dyn Descriptor> = get_descriptor_from_type(typ);
                    descr.action(DescriptorAction::Open, vm.stack_mut());
                    vm.stack_mut().push(RegisterType::Descriptor(Rc::new(Mutex::new(descr))))
                } else {
                    let stack = vm.stack_mut();

                    let (inp, _) = get_descriptor_contract(typ, action);
                    let inp = inp.len();

                    let mut tmp_stack = vec![];
                    for _ in 0..inp {
                        let pop = stack.pop().unwrap();
                        tmp_stack.push(pop);
                    }

                    let last = stack.pop();
                    if let Some(typ) = last {
                        if let RegisterType::Descriptor(descr) = typ {
                            let descr = descr.clone();
                            let push = descr.clone();
                            let mut lock = descr.lock();
                            let lock = lock.as_mut().unwrap();
                            stack.extend(tmp_stack);
                            lock.action(action, stack);
                            stack.push(RegisterType::Descriptor(push));
                        }
                    }
                }
            }
        }
    })
}


pub fn type_check_fn() -> Box<dyn Fn(&OperationData, &HashMap<String, Function>, &mut Vec<Types>, bool) -> TypeCheckError> {
    Box::new(|op_data, fns, stack, compile_time| {
        if let OperationType::Descriptor = op_data.0 {
            if let Operand::DescriptorAction(descr, action) = op_data.clone().2.unwrap() {
                let typ = map_or_error_desc_type(&descr, true);
                let action = map_or_error_desc_action(&action, true);

                if stack.len() == 0 {
                    ErrorTypes::TooFewElements.into()
                } else {
                    let res = match action {
                        DescriptorAction::Open => {
                            let descr = get_descriptor_from_type(typ);
                            let success = descr.typecheck(DescriptorAction::Open, stack);
                            stack.push(Types::Descriptor);
                            success
                        }
                        DescriptorAction::ToString | DescriptorAction::ReadAll | DescriptorAction::WriteAll => descriptor_typecheck(stack, typ, action),
                        DescriptorAction::Close => {
                            ErrorTypes::ClosureError.into_txt("Closing descriptor is not allowed via a descriptor action")
                        }
                        _ => {
                            let descr = get_descriptor_from_type(typ);
                            descr.typecheck(action, stack)
                        }
                    };

                    res
                }
            } else {
                ErrorTypes::ClosureError.into()
            }
        } else {
            ErrorTypes::ClosureError.into()
        }
    })
}