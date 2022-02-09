use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Read;
use std::iter::TrustedRandomAccessNoCoerce;
use std::os::linux::raw::stat;
use std::path::{Path, PathBuf};

use crate::util::{compiler_error, compiler_error_str};
use crate::util::internals::{Internal, to_internal};
use crate::util::operation::{JumpOffset, Operand, Operation, OperationType};
use crate::util::position::Position;
use crate::util::token::*;
use crate::util::types::Types;
use crate::VM;

#[derive(Debug, Clone)]
pub struct Function {
    data: (String, Vec<Types>, Vec<Types>),
    pub(crate) operations: Vec<(Position, Operation)>,
}

pub struct State {
    operations: HashMap<String, Function>,
    included: bool,
    path: PathBuf,
    in_fn: Option<Function>,
}

impl State {
    pub fn new(path: PathBuf) -> Self {
        Self {
            operations: HashMap::new(),
            included: false,
            in_fn: None,
            path,
        }
    }

    pub fn new_with_include(path: PathBuf) -> Self {
        Self {
            operations: HashMap::new(),
            included: true,
            in_fn: None,
            path,
        }
    }

    pub fn update(&mut self, tokens: Vec<(Position, Token)>) {
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
                                        if self.included {
                                            compiler_error_str("Nested includes are not allowed", pos.clone());
                                        }

                                        let path = iterator.next();

                                        if let None = path {
                                            compiler_error_str("No string provided. Empty tokenstream", pos.clone());
                                        }

                                        let path = path.unwrap();

                                        if let TokenValue::String(s_path) = path.1.value().clone() {
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
                                            let state = tokenize(parsed, true, incl_path.parent().unwrap().to_path_buf());

                                            self.operations.extend(state.operations);
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
                                    data: (name.clone(), inp.clone(), outp.clone()),
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
                    let mut function = self.in_fn.clone().unwrap();
                    let to_add = match token.typ() {
                        TokenType::Word => {
                            let token = token.clone();

                            let text = token.text().to_string();

                            if text.starts_with("~") && {
                                let mut token = text.clone();
                                token.remove(0);

                                self.operations.contains_key(&token)
                            } {
                                let mut func_name = text.clone();
                                func_name.remove(0);

                                vec![Operation {
                                    typ: OperationType::PushFunction,
                                    token,
                                    operand: Some(Operand::Str(func_name)),
                                }]
                            } else if self.operations.contains_key(&text) {
                                vec![Operation {
                                    typ: OperationType::PushFunction,
                                    token: token.clone(),
                                    operand: Some(Operand::Str(text)),
                                }, Operation {
                                    typ: OperationType::Call,
                                    token,
                                    operand: None,
                                }]
                            } else {
                                let internal = to_internal(value, pos.clone());
                                vec![Operation {
                                    typ: OperationType::Internal,
                                    token,
                                    operand: Some(Operand::Internal(internal)),
                                }]
                            }
                        }
                        TokenType::Int => {
                            let token = token.clone();

                            vec![Operation {
                                typ: OperationType::PushInt,
                                token,
                                operand: Some(Operand::Int(*if let TokenValue::Int(val) = value {
                                    val
                                } else {
                                    compiler_error_str("Internal parser error occurred", pos);
                                    unreachable!();
                                })),
                            }]
                        }
                        TokenType::Str => {
                            vec![Operation {
                                typ: OperationType::PushStr,
                                token: token.clone(),
                                operand: Some(Operand::Str(if let TokenValue::String(str) = value {
                                    str.clone()
                                } else {
                                    compiler_error_str("Internal parser error occurred", pos);
                                    unreachable!();
                                })),
                            }]
                        }
                        TokenType::Keyword => {
                            let mut ops = vec![];

                            if let TokenValue::Keyword(keyword) = value {
                                match keyword {
                                    Keyword::INCLUDE => {
                                        compiler_error_str("Include is only allowed on the top level", pos.clone());
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
                    };

                    to_add.iter().for_each(|to_add| {
                        let pos = pos.clone();
                        function.operations.push((pos, to_add.clone()));
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

        if self.in_fn.is_some() {
            let fnc = self.in_fn.clone().unwrap();
            compiler_error(format!("Unclosed function {}", fnc.data.0), Position::default())
        }
    }

    pub fn type_check(self) -> Result<VM, (Position, String)> {
        Ok(VM::from(self))
    }

    pub fn get_ops(&self) -> HashMap<String, Function> {
        self.operations.clone()
    }
}

pub fn pre_parse(string: String, file: PathBuf, path: PathBuf) -> Vec<(Position, String)> {
    let lines: Vec<(u32, String)> = string.lines().fold(vec![], |mut count, line| {
        count.push(((count.len() + 1) as u32, line.to_string()));
        count
    });

    let (unclosed, pos, _, lines): (bool, Position, String, Vec<(Position, String)>) = lines.iter()
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
                tokens.push(((tokens.len() + 1) as u32, token.to_string()));
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
        })
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
                        position = line.0;
                    } else {
                        vec.push(line);
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

            (change, position, append, vec)
        });

    if unclosed {
        compiler_error_str("Unclosed string sequence", pos);
    }

    let lines = lines.iter()
        .filter(|line| line.1.len() > 0)
        .map(|line| line.clone())
        .collect();

    lines
}

pub fn tokenize(tokens: Vec<(Position, String)>, included: bool, path: PathBuf) -> State {
    let mut state = if included { State::new_with_include(path) } else { State::new(path) };

    state.update(tokens.iter().map(|token| {
        let token = token.clone();

        (token.clone().0, Token::from(token))
    }).collect());

    state
}