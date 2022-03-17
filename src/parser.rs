use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Read;
use std::iter::TrustedRandomAccessNoCoerce;
use std::path::{Path, PathBuf};

use rayon::iter::ParallelIterator;
use rayon::prelude::{IntoParallelIterator, IntoParallelRefIterator};

use crate::opt::resolve_opt;
use crate::util::{compiler_error, compiler_error_str, compiler_warning};
use crate::util::internals::{Internal, to_internal};
use crate::util::operation::{JumpOffset, Operand, Operation, OperationData, OperationType};
use crate::util::operations::{calling_runtime, calling_typecheck, internals_runtime, internals_typecheck, simple_runtime, simple_typecheck};
use crate::util::position::Position;
use crate::util::token::*;
use crate::util::token::TokenType::Function as TokenFunction;
use crate::util::type_check::{ErrorTypes, TypeCheckError, Types};
use crate::VM;
use crate::vm::MAX_CALL_STACK_SIZE;

static MAX_INCL_DEPTH: u8 = 3;

#[derive(Debug, Clone)]
pub struct FunctionData(String, Vec<Types>, Vec<Types>);

#[derive(Clone)]
pub struct Function {
    pub(crate) data: FunctionData,
    pub(crate) operations: Vec<(Position, Operation)>,
}

impl Function {
    pub fn get_contract(&self) -> (Vec<Types>, Vec<Types>) {
        (self.data.1.clone(), self.data.2.clone())
    }

    pub fn type_check(&self, functions: HashMap<String, Function>, stack: &mut Vec<Types>) -> TypeCheckError {
        self.operations.iter()
            .fold(ErrorTypes::None.into(), |acc, op| {
                let type_check: TypeCheckError = op.1.type_check(&functions, stack, true).into();

                if type_check.is_error() {
                    let op = op.clone();
                    let data = op.1.data();
                    let typ = &data.0;
                    let operand = &data.2;
                    let pos = data.1.location().clone();
                    compiler_warning(format!("\r\nOperation caused type check failure. \r\nOperation Type: {:?} \r\nOperation Value: {:?}", typ, operand), pos);
                    type_check
                } else {
                    acc
                }
            })
    }

    pub fn name(&self) -> String {
        self.data.0.clone()
    }
}

pub struct State {
    operations: HashMap<String, Function>,
    functions: HashMap<String, (Vec<Types>, Vec<Types>)>,
    incl_lvl: u8,
    path: PathBuf,
    sys_libs: Vec<String>,
    in_fn: Option<Function>,
}

impl State {
    pub fn new(path: PathBuf) -> Self {
        Self {
            operations: HashMap::new(),
            functions: HashMap::new(),
            incl_lvl: 0,
            in_fn: None,
            sys_libs: vec![],
            path,
        }
    }

    pub fn new_with_include(old: u8, path: PathBuf) -> Self {
        Self {
            operations: HashMap::new(),
            functions: HashMap::new(),
            incl_lvl: old + 1,
            in_fn: None,
            sys_libs: vec![],
            path,
        }
    }

    pub fn update(&mut self, tokens: Vec<(Position, Token)>) {
        self.functions = tokens.clone().iter().fold(HashMap::new(), |mut acc, instr| {
            if instr.1.typ().clone() == TokenType::Function {
                if let TokenValue::Function(name, inp, outp) = instr.1.value().clone() {
                    acc.insert(name, (inp, outp));
                }
            }
            acc
        });

        let mut iterator = tokens.iter();


        while iterator.size() != 0 {
            let token = iterator.next();

            if let Some(token) = token {
                let token = token.clone();
                let pos = token.clone().0;
                let token = token.clone().1;
                let value = token.value();
                if self.in_fn.is_none() {
                    match token.typ() {
                        TokenType::Keyword => {
                            if let TokenValue::Keyword(keyword) = value {
                                match keyword {
                                    Keyword::INCLUDE => {
                                        if self.incl_lvl > MAX_INCL_DEPTH {
                                            compiler_error_str("Nested includes are not allowed", pos.clone());
                                        }

                                        let path = iterator.next();

                                        if let None = path {
                                            compiler_error_str("No string provided. Empty tokenstream", pos.clone());
                                        }

                                        let path = path.unwrap();

                                        if let TokenValue::String(mut s_path) = path.1.value().clone() {
                                            if s_path.starts_with("@") {
                                                s_path.remove(0);
                                                if s_path.starts_with("std/") {
                                                    if !self.sys_libs.contains(&s_path) {
                                                        self.sys_libs.push(s_path);
                                                    }
                                                } else {
                                                    let pathbuf = PathBuf::from(Path::new(&s_path));
                                                    let parsed = pre_parse(resolve_opt(&s_path), pathbuf.clone(), pathbuf.clone());
                                                    let state = tokenize(parsed, self.incl_lvl, pathbuf);

                                                    self.functions.extend(state.functions);
                                                    self.operations.extend(state.operations);
                                                }
                                            } else {
                                                let mut incl_path = self.path.clone();
                                                incl_path.push(s_path);

                                                let file = OpenOptions::new().read(true).open(&incl_path);
                                                if file.is_err() {
                                                    compiler_error(format!("The file {:?} could not be found", incl_path), pos.clone());
                                                }

                                                let mut string = String::new();

                                                let mut file = file.unwrap();
                                                if file.read_to_string(&mut string).is_err() {
                                                    compiler_error(format!("The file {:?} could not be read from", incl_path), pos.clone());
                                                }

                                                let parsed = pre_parse(string, incl_path.clone(), incl_path.parent().unwrap().to_path_buf());
                                                let state = tokenize(parsed, self.incl_lvl, incl_path.parent().unwrap().to_path_buf());

                                                self.operations.extend(state.operations);
                                            }
                                        } else {
                                            compiler_error(format!("No string passed to include. Found: {:?}", path.1.value()), pos.clone());
                                            unreachable!()
                                        }
                                    }
                                    _ => {
                                        compiler_error_str("Only includes and functions are allowed on the top level", pos);
                                    }
                                }
                            }
                        }
                        TokenType::Function => {
                            if let TokenValue::Function(name, inp, outp) = value {
                                self.in_fn = Some(Function {
                                    data: FunctionData(name.clone(), inp.clone(), outp.clone()),
                                    operations: vec![],
                                })
                            } else {
                                compiler_error_str("Compiler bug! Got function without function data", pos)
                            }
                        }
                        _ => {
                            compiler_error_str("Only includes and functions are allowed on the top level", pos);
                        }
                    }
                } else {
                    let sys_libs = self.sys_libs.clone();
                    let mut function = self.in_fn.clone().unwrap();
                    let to_add = match token.typ() {
                        TokenType::Word => {
                            let token = token.clone();

                            let text = token.text().to_string();

                            if text.starts_with("~") && {
                                let mut token = text.clone();
                                token.remove(0);

                                self.functions.contains_key(&token)
                            } {
                                let mut func_name = text.clone();
                                func_name.remove(0);

                                let (inp, outp) = self.functions.get(&func_name).unwrap();

                                vec![
                                    Operation::new(
                                        OperationData(OperationType::PushFunction, token, Some(Operand::PushFunction(func_name, inp.clone(), outp.clone()))),
                                        simple_runtime::create_push(),
                                        simple_typecheck::create_push_type_check(),
                                    )
                                ]
                            } else if self.functions.contains_key(&text) {
                                vec![Operation::new(OperationData(OperationType::Call, token, Some(Operand::Call(text))),
                                                    calling_runtime::create_fn(),
                                                    calling_typecheck::create_push_type_check(), )]
                            } else {
                                let internal = to_internal(sys_libs, value, pos.clone());
                                vec![Operation::new(
                                    OperationData(OperationType::Internal, token, Some(Operand::Internal(internal))),
                                    internals_runtime::get_internal_executor(internal),
                                    internals_typecheck::get_internal_typecheck(internal),
                                )]
                            }
                        }
                        TokenType::Int | TokenType::Str => {
                            let token = token.clone();

                            let operand = if token.typ() == &TokenType::Int {
                                Operand::Int(*if let TokenValue::Int(val) = value {
                                    val
                                } else {
                                    compiler_error_str("Internal parser error occurred", pos);
                                    unreachable!();
                                })
                            } else {
                                Operand::Str(if let TokenValue::String(str) = value {
                                    str.clone()
                                } else {
                                    compiler_error_str("Internal parser error occurred", pos);
                                    unreachable!();
                                })
                            };

                            vec![
                                Operation::new(
                                    OperationData(OperationType::Push, token, Some(operand)),
                                    simple_runtime::create_push(),
                                    simple_typecheck::create_push_type_check(),
                                )
                            ]
                        }
                        TokenType::Keyword => {
                            let mut ops = vec![];

                            if let TokenValue::Keyword(keyword) = value {
                                match keyword {
                                    Keyword::INCLUDE => {
                                        compiler_error_str("Include is only allowed on the top level", pos.clone());
                                    }
                                    Keyword::Call | Keyword::CallIf => {
                                        ops.push(Operation::new(OperationData(
                                            if keyword.clone() == Keyword::Call {
                                                OperationType::Call
                                            } else {
                                                OperationType::CallIf
                                            }, token, None),
                                                                calling_runtime::create_fn(),
                                                                calling_typecheck::create_push_type_check(),
                                        ))
                                    }
                                    Keyword::End => {
                                        self.operations.insert(function.data.clone().0, function);
                                        self.in_fn = None;
                                        continue;
                                    }
                                }
                            }

                            ops
                        }
                        TokenType::Function => {
                            compiler_error_str("Functions are only allowed on the top level", pos.clone());
                            unreachable!()
                        }
                        TokenType::FunctionPtr => {
                            if let TokenValue::Function(name, inp, outp) = token.value().clone() {
                                vec![Operation::new(
                                    OperationData(OperationType::Push, token, Some(Operand::PushFunction(name, inp, outp))),
                                    simple_runtime::create_push(),
                                    simple_typecheck::create_push_type_check(),
                                )]
                            } else {
                                compiler_error_str("Internal parser error occurred", pos);
                                unreachable!();
                            }
                        }
                    };

                    to_add.into_iter().for_each(|to_add| {
                        let pos = pos.clone();
                        function.operations.push((pos, to_add));
                    });

                    if self.in_fn.is_some() {
                        self.in_fn = Some(function)
                    } else {
                        compiler_error_str("Compiler bug! Wanted to modify in_fn even though we are not in a function anymore", pos.clone());
                    }
                }
            } else {
                compiler_error_str("Empty token stream", Position::default());
            }
        }

        if let Some(ref fnc) = self.in_fn {
            compiler_error(format!("Unclosed function {}", fnc.data.0), Position::default())
        }


        self.operations = self.operations.clone().iter().map(|entry: (&String, &Function)| {
            (entry.clone().0.clone(), Function {
                data: entry.1.data.clone(),
                operations: entry.1.operations.iter().map(|op| {
                    if op.1.data.0 == OperationType::PushFunction {
                        let mut operation = op.1.clone();

                        if let Operand::Str(func_name) = operation.data.2.clone().unwrap() {
                            let FunctionData(name, inp, outp) = self.operations.get(&func_name).unwrap().data.clone();

                            operation.data.2 = Some(Operand::PushFunction(name, inp, outp));
                        }

                        (op.0.clone(), operation)
                    } else {
                        op.clone()
                    }
                }).collect(),
            })
        }).collect();
    }

    pub fn type_check(self) -> Result<VM, String> {
        for (name, function) in &self.operations {
            let mut stack = function.get_contract().0;

            let type_check = function.type_check(self.operations.clone(), &mut stack);

            if type_check.error != ErrorTypes::None {
                return Err(format!("Function {} failed type check: {}", name, type_check));
            }

            if stack != function.get_contract().1 {
                return Err(format!("Function {} failed type check! You still have unused elements left", name));
            }
        }


        Ok(VM::from(self))
    }

    pub fn get_ops(&self) -> &HashMap<String, Function> {
        &self.operations
    }
}

pub fn pre_parse(string: String, file: PathBuf, path: PathBuf) -> Vec<(Position, String)> {
    let lines: Vec<(u32, String)> = string.lines().fold(vec![], |mut count, line| {
        count.push(((count.len() + 1) as u32, line.to_string()));
        count
    });

    let (unclosed, pos, _, lines): (bool, Position, String, Vec<(Position, String)>) = lines.into_par_iter()
        .filter(|line| line.1.len() > 0)
        .map(|line| {
            if !line.1.contains("//") {
                line.clone()
            } else {
                let pos = line.1.find("//").unwrap();
                let (left, _) = line.1.split_at(pos);
                (line.0, left.to_string())
            }
        })
        .filter(|line| line.1.len() > 0)
        .map(|line| {
            (line.clone().0, line.1.trim().to_string())
        })
        .map(|line| {
            let tokens = line.1.split(" ").fold(vec![], |mut tokens, token| {
                let token_len = (tokens.len() + tokens.iter().fold(0, |acc, tkn| {
                    acc + token.len()
                })) as u32;

                tokens.push((token_len, token.to_string()));
                tokens
            });

            (line.0, tokens)
        })
        .flat_map(|line| {
            line.1.iter().map(|token| {
                (Position {
                    token_pos_line: line.0,
                    token_pos_x: token.0,
                    file: file.clone(),
                }, token.clone().1)
            }).collect::<Vec<(Position, String)>>()
        }).collect::<Vec<(Position, String)>>()
        .iter()
        .fold((false, Position::default(), String::new(), vec![]), |collect, line| {
            let mut change = collect.0;
            let mut position = collect.1;
            let mut append = collect.2;
            let mut vec = collect.3;
            let line_string = line.clone().1;

            if line_string.starts_with("\"") && line_string.ends_with("\"") && line_string.len() != 1 {
                vec.push((position.clone(), line_string));
                change = false;
                append = String::new();
            } else {
                if !change {
                    if line_string.starts_with("\"") {
                        change = true;
                        append.push_str(&line_string);
                        append.push_str(" ");
                        position = line.clone().0;
                    } else {
                        vec.push(line.clone());
                    }
                } else {
                    if line_string.ends_with("\"") {
                        change = false;
                        append.push_str(&line_string);
                        vec.push((position, append.clone()));
                        append = String::new();
                        position = Position::default();
                    } else {
                        append.push_str(&line_string);
                        append.push_str(" ");
                    }
                };
            }

            let ret = (change, position, append, vec);

            ret
        });

    if unclosed {
        compiler_error_str("Unclosed string sequence", pos);
    }

    let lines = lines.par_iter()
        .filter(|line| line.1.len() > 0)
        .map(|line| line.clone())
        .collect();

    lines
}

pub fn tokenize(tokens: Vec<(Position, String)>, included: u8, path: PathBuf) -> State {
    let mut state = if included != 0 { State::new_with_include(included, path) } else { State::new(path) };

    state.update(tokens.par_iter().map(|token| {
        let token = token.clone();

        (token.clone().0, Token::from(token))
    }).collect());

    state
}