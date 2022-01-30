use crate::util::internals::Internal;
use crate::util::position::Position;
use crate::util::token::Token;

pub type JumpOffset = i32;

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone, Debug)]
pub enum OperationType {
    PushInt,
    PushPtr,
    PushBool,
    PushStr,
    Internal,
    Jump,
    JumpIf,
}

#[derive(Clone, Debug)]
pub enum Operand {
    Int(i32),
    Str(String),
    Bool(bool),
    Internal(Internal),
    Jump(JumpOffset),
}

#[derive(Clone, Debug)]
pub struct Operation {
    pub(crate) typ: OperationType,
    pub(crate) token: Token,
    pub(crate) operand: Option<Operand>,
}
