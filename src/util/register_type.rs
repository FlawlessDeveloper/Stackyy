use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::Position;
use crate::util::operations::{Descriptor, DescriptorAction};
use crate::util::runtime_error_str;
use crate::util::type_check::Types;

#[derive(Clone, Debug)]
pub enum RegisterType {
    Int(i32),
    Function(String, Vec<Types>, Vec<Types>),
    Pointer(u32),
    String(String),
    Bool(bool),
    Descriptor(Rc<Mutex<Box<dyn Descriptor>>>),
    Empty,
}


impl RegisterType {
    pub fn to_string(&self) -> Option<String> {
        match self {
            RegisterType::Int(int) => {
                Some(int.to_string())
            }
            RegisterType::Pointer(pointer) => {
                Some(format!("*{:#x}", pointer))
            }
            RegisterType::String(str) => {
                Some(format!("\"{}\"", str))
            }
            RegisterType::Bool(bool) => {
                Some(bool.to_string())
            }
            RegisterType::Function(name, ..) => {
                Some(format!("*{}()", name))
            }
            RegisterType::Descriptor(descr) => {
                let mut locked = descr.lock();
                let locked = locked.as_mut().unwrap();
                let mut tmp_stack = vec![];
                locked.action(DescriptorAction::ToString, &mut tmp_stack);
                let str = tmp_stack.get(0).unwrap();
                str.to_string()
            }
            RegisterType::Empty => {
                None
            }
        }
    }

    pub fn to_string_stacked(&self, position: Position, stack: &mut Vec<RegisterType>) {
        let str = self.to_string();
        if let Some(str) = str {
            stack.push(RegisterType::String(str));
        } else {
            runtime_error_str("Trying to convert empty to string", position);
        }
    }
}