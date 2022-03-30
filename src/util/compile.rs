use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{OperationDataInfo, VM};
use crate::args::Compile;
use crate::parser::{Function, FunctionData};
use crate::util::operation::{Operation, OperationData};

#[derive(Serialize, Deserialize, Clone)]
pub struct CompiledFunction {
    pub(crate) data: FunctionData,
    pub(crate) operations: Vec<(OperationDataInfo, OperationData)>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProgramMetadata {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) author: Option<String>,
    pub(crate) description: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CompiledProgram {
    pub(crate) data: ProgramMetadata,
    pub(crate) operations: HashMap<String, CompiledFunction>,
}