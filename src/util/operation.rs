use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{compiler_error_str, Position, VM};
use crate::args::Compile;
use crate::parser::Function;
use crate::util::compiler_warning_str;
use crate::util::internals::Internal;
use crate::util::operations::{CALLING_RUNTIME, CALLING_TYPECHECK, DESCRIPTOR_RUNTIME, DESCRIPTOR_TYPECHECK, DescriptorAction, INTERNAL_RUNTIME, INTERNAL_TYPECHECK, SIMPLE_RUNTIME, SIMPLE_TYPECHECK};
use crate::util::token::Token;
use crate::util::type_check::{TypeCheckError, Types};

#[derive(Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub enum OperationType {
    Push,
    PushFunction,
    Internal,
    Descriptor,
    Jump,
    JumpIf,
    Call,
    CallIf,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Operand {
    Int(i32),
    Str(String),
    Bool(bool),
    Internal(Internal),
    PushFunction(String, Vec<Types>, Vec<Types>),
    Call(String),
    DescriptorAction(String, String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OperationDataInfo {
    Token(Token),
    Position(Position),
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OperationData {
    pub(crate) typ: OperationType,
    pub(crate) data: OperationDataInfo,
    pub(crate) operand: Option<Operand>,
}

#[derive(Clone)]
pub struct Operation {
    pub(crate) data: OperationData,
    pub(crate) execute_fn: Arc<Box<dyn Fn(&OperationData, &mut VM)>>,
    pub(crate) type_check: Arc<Box<dyn Fn(&OperationData, &HashMap<String, Function>, &mut Vec<Types>, bool) -> TypeCheckError>>,
}

impl Display for Operation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self.data)
    }
}

impl Operation {
    pub fn type_check(&self, functions: &HashMap<String, Function>, stack: &mut Vec<Types>, compile_time: bool) -> TypeCheckError {
        self.type_check.call((&self.data, functions, stack, compile_time))
    }

    pub fn execute_op(&self, vm: &mut VM) {
        self.execute_fn.call((&self.data, vm));
    }

    pub fn data(&self) -> &OperationData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut OperationData {
        &mut self.data
    }


    pub fn new(data: OperationData, execute_fn: Arc<Box<dyn Fn(&OperationData, &mut VM)>>, type_check: Arc<Box<dyn Fn(&OperationData, &HashMap<String, Function>, &mut Vec<Types>, bool) -> TypeCheckError>>) -> Self {
        Operation { data, execute_fn, type_check }
    }
}

impl OperationData {
    pub fn new(typ: OperationType, data: Token, compile_config: &Compile, operand: Option<Operand>) -> Self {
        let data = match compile_config.strip_data {
            0 => OperationDataInfo::Token(data),
            1 => OperationDataInfo::Position(data.location().clone()),
            _ => OperationDataInfo::None,
        };

        Self { typ, data, operand }
    }

    pub fn new_interpret(typ: OperationType, data: Token, operand: Option<Operand>) -> Self {
        Self { typ, data: OperationDataInfo::Token(data), operand }
    }
}

impl OperationDataInfo {
    pub fn from_token(token: Token, compile_config: &Compile) -> Self {
        let data = match compile_config.strip_data {
            0 => OperationDataInfo::Token(token),
            1 => OperationDataInfo::Position(token.location().clone()),
            _ => OperationDataInfo::None,
        };
        data
    }
}

impl Display for OperationDataInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationDataInfo::Token(token) => {
                write!(f, "at {}", token.location())?;
                write!(f, "-> '{}'", token.text())?;
            }
            OperationDataInfo::Position(pos) => {
                write!(f, "at {}", pos)?;
            }
            OperationDataInfo::None => {
                write!(f, "")?;
            }
        }
        Ok(())
    }
}

impl From<OperationData> for Operation {
    fn from(data: OperationData) -> Self {
        let typ = data.clone().typ;
        match typ {
            OperationType::Push => {
                Operation::new(data, SIMPLE_RUNTIME.clone(), SIMPLE_TYPECHECK.clone())
            }
            OperationType::PushFunction => {
                Operation::new(data, CALLING_RUNTIME.clone(), CALLING_TYPECHECK.clone())
            }
            OperationType::Internal => {
                Operation::new(data, INTERNAL_RUNTIME.clone(), INTERNAL_TYPECHECK.clone())
            }
            OperationType::Descriptor => {
                Operation::new(data, DESCRIPTOR_RUNTIME.clone(), DESCRIPTOR_TYPECHECK.clone())
            }
            OperationType::Call => {
                Operation::new(data, CALLING_RUNTIME.clone(), CALLING_TYPECHECK.clone())
            }
            OperationType::CallIf => {
                Operation::new(data, CALLING_RUNTIME.clone(), CALLING_TYPECHECK.clone())
            }
            _ => {
                unreachable!()
            }
        }
    }
}