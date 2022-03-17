use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use crate::{compiler_error, compiler_error_str, VM};
use crate::parser::Function;
use crate::util::{compiler_warning, compiler_warning_str};
use crate::util::internals::{Internal, type_check};
use crate::util::position::Position;
use crate::util::register_type::RegisterType;
use crate::util::token::Token;
use crate::util::type_check::{ErrorTypes, TypeCheckError, Types};

pub type JumpOffset = u32;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub enum OperationType {
    Push,
    PushFunction,
    Internal,
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
}

#[derive(Debug, Clone)]
pub struct OperationData(
    pub(crate) OperationType,
    pub(crate) Token,
    pub(crate) Option<Operand>,
);

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