use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{OperationDataInfo, VM};
use crate::parser::{Function, FunctionData};
use crate::util::operation::{Operation, OperationData};

#[derive(Serialize, Deserialize, Clone)]
pub struct CompiledFunction {
    pub(crate) data: FunctionData,
    pub(crate) operations: Vec<(OperationDataInfo, OperationData)>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProgramMetadata {
    author: String,
    name: String,
    version: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CompiledProgram {
    pub(crate) data: ProgramMetadata,
    pub(crate) operations: HashMap<String, CompiledFunction>,
}

pub fn to_vm(program: CompiledProgram) -> VM {
    let fncs = program.operations;
    let fncs = fncs.iter().map(|entry| {
        let fnc = entry.1;
        let data = fnc.data.clone();
        let fnc_ops = fnc.operations.iter().map(|op| {
            let nop = Operation::from(op.1.clone());
            (op.0.clone(), nop)
        }).collect::<Vec<_>>();

        let fnc = Function {
            data,
            operations: fnc_ops,
        };
        (entry.0.clone(), fnc)
    }).collect::<HashMap<_, _>>();

    VM::new(fncs)
}