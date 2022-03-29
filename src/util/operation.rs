use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::{compiler_error_str, Position, VM};
use crate::args::Compile;
use crate::parser::Function;
use crate::util::compiler_warning_str;
use crate::util::internals::Internal;
use crate::util::operations::DescriptorAction;
use crate::util::token::Token;
use crate::util::type_check::{TypeCheckError, Types};

pub type JumpOffset = u32;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
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

#[derive(Clone, Debug)]
pub enum Operand {
    Int(i32),
    Str(String),
    Bool(bool),
    Internal(Internal),
    Jump(JumpOffset),
    PushFunction(String, Vec<Types>, Vec<Types>),
    Call(String),
    DescriptorAction(String, String),
}

#[derive(Debug, Clone)]
pub enum OperationDataInfo {
    Token(Token),
    Position(Position),
    None,
}

#[derive(Debug, Clone)]
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


    pub fn new(data: OperationData, execute_fn: Box<dyn Fn(&OperationData, &mut VM)>, type_check: Box<dyn Fn(&OperationData, &HashMap<String, Function>, &mut Vec<Types>, bool) -> TypeCheckError>) -> Self {
        Operation { data, execute_fn: Arc::new(execute_fn), type_check: Arc::new(type_check) }
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
                write!(f, "{}", token.location())?;
                write!(f, "-> '{}'", token.text())?;
            }
            OperationDataInfo::Position(pos) => {
                write!(f, "{}", pos)?;
            }
            OperationDataInfo::None => {
                write!(f, "No debug info")?;
            }
        }
        Ok(())
    }
}