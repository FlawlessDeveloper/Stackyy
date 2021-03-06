use std::collections::HashMap;
use std::process::exit;

use crate::CompiledProgram;
use crate::parser::{Function, State};
use crate::util::{compiler_error_str, runtime_error, runtime_error_str, runtime_warning, runtime_warning_str};
use crate::util::operation::{Operation, OperationData, OperationDataInfo, OperationType};
use crate::util::position::Position;
use crate::util::register_type::RegisterType;
use crate::util::type_check::{ErrorTypes, Types};

pub const MAX_CALL_STACK_SIZE: u8 = 40;

pub struct VM {
    ip: i32,
    ops: HashMap<String, Function>,
    stack: Vec<RegisterType>,
    type_stack: Vec<Types>,
    last_op: Option<(OperationDataInfo, OperationData)>,
    depth: u8,
    reg_a: RegisterType,
    reg_b: RegisterType,
    reg_c: RegisterType,
    reg_d: RegisterType,
    reg_e: RegisterType,
    reg_f: RegisterType,
    reg_g: RegisterType,
    reg_h: RegisterType,
}

impl From<State> for VM {
    fn from(state: State) -> Self {
        let mut ops = state.get_ops().clone();
        VM::new(ops)
    }
}

impl From<CompiledProgram> for VM {
    fn from(program: CompiledProgram) -> Self {
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
}

impl VM {
    pub fn new(ops: HashMap<String, Function>) -> Self {
        Self {
            ip: 0,
            ops,
            stack: vec![],
            type_stack: vec![],
            last_op: None,
            depth: 0,
            reg_a: RegisterType::Empty,
            reg_b: RegisterType::Empty,
            reg_c: RegisterType::Empty,
            reg_d: RegisterType::Empty,
            reg_e: RegisterType::Empty,
            reg_f: RegisterType::Empty,
            reg_g: RegisterType::Empty,
            reg_h: RegisterType::Empty,
        }
    }

    pub fn run(&mut self) {
        let empty = OperationDataInfo::None;

        if !self.ops.contains_key("main") {
            runtime_error_str("Program does not contain a main function", &empty);
        }

        let start = self.ops.get("main").unwrap().clone();

        self.execute_fn(&start);

        if self.stack.len() != 1 {
            runtime_error_str("No return code provided", &empty);
        }

        if let RegisterType::Int(exit_code) = self.stack.pop().unwrap() {
            exit(exit_code)
        } else {
            runtime_error_str("Return code can only be of type integer", &empty);
        }
    }

    pub fn execute_fn(&mut self, fnc: &Function) {
        self.depth += 1;
        for operation in &fnc.operations {
            self.execute_op(operation, fnc.name())
        }
        self.depth -= 1;
    }

    pub fn stack(&self) -> &Vec<RegisterType> {
        &self.stack
    }

    pub fn stack_mut(&mut self) -> &mut Vec<RegisterType> {
        &mut self.stack
    }

    fn execute_op(&mut self, op: &(OperationDataInfo, Operation), fn_name: String) {
        let info = &op.0;
        let data = op.1.data();
        let typecheck = &op.1.type_check;
        let exec = &op.1.execute_fn;
        if self.depth > MAX_CALL_STACK_SIZE {
            runtime_error_str("Stack overflow", info);
        }


        if self.type_stack.len() != self.stack.len() {
            if cfg!(debug_assertions) {
                runtime_error(format!("Typecheck desync happened.\r\nResponsible operation: {:#?}\r\nStack {:?}\r\nTypestack {:?}", self.last_op.clone().unwrap(), self.stack, self.type_stack), info);
            } else {
                runtime_error(format!("Typecheck desync happened. Please create a issue on github"), info);
            }
        }

        let tc_error = (typecheck(data, &self.ops, &mut self.type_stack, false)).is_error();

        if !tc_error {
            exec(data, self);
        } else {
            runtime_error(format!("Function {} failed type check ", fn_name), info);
        };

        self.last_op = Some((info.clone(), data.clone()))
    }

    pub fn ops(&self) -> &HashMap<String, Function> {
        &self.ops
    }
}