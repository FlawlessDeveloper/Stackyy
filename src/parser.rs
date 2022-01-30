use std::fs::OpenOptions;
use std::io::Read;

use crate::util::{compiler_error, compiler_error_str};
use crate::util::internals::to_internal;
use crate::util::operation::{Operand, Operation, OperationAddr, OperationType};
use crate::util::position::Position;
use crate::util::token::*;

pub struct State {
    tokens: Vec<(Position, Token)>,
    operations: Vec<(Position, Operation)>,
    stack: Vec<OperationAddr>,
}

impl State {
    pub fn new(tokens: Vec<(Position, Token)>, path: PathBuf) -> Self {
        Self {
            tokens,
            operations: vec![],
            stack: vec![],
        }
    }

    pub fn push(&mut self, pos: Position, token: &Token, index: usize) {
        let value = token.value();
        let to_add = match token.typ() {
            TokenType::Word => {
                let token = token.clone();
                let internal = to_internal(value, pos.clone());
                vec![Operation {
                    typ: OperationType::Internal,
                    token,
                    operand: Some(Operand::Internal(internal)),
                }]
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
                let token = token.clone();

                let mut ops = vec![];

                if let TokenValue::Keyword(keyword) = value {
                    match keyword {
                        Keyword::INCLUDE => {
                            let path = self.tokens.remove(index);

                            if let TokenValue::String(path) = path.1.value() {
                                let state = match OpenOptions::new().read(true).open(path) {
                                    Ok(mut file) => {
                                        let mut string = String::new();
                                        match file.read_to_string(&mut string) {
                                            Ok(_) => {
                                                let pre_parsed = pre_parse(string, path.clone());
                                                tokenize(pre_parsed)
                                            }
                                            Err(_) => {
                                                compiler_error(format!("The file {} could not be read", path), pos);
                                                unreachable!()
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        compiler_error(format!("The file {} could not be opened", path), pos);
                                        unreachable!()
                                    }
                                };

                                state.operations.iter().for_each(|op| {
                                     ops.push(Operation {
                                         typ: OperationType::Include,
                                         token: token.clone(),
                                         operand: Some(Operand::Include(Box::from(op.clone())))
                                     });
                                });
                            } else {
                                compiler_error_str("No string passed to the compiler", pos);
                                unreachable!()
                            }
                        }
                    }

                }

                ops
            }
        };

        to_add.iter().for_each(|to_add| {
            let pos = pos.clone();
            self.operations.push((pos, to_add.clone()))
        });
    }

    pub fn type_check(&self) -> bool {
        true
    }

    pub fn get_ops(&self) -> Vec<(Position, Operation)> {
        self.operations.clone()
    }
}

pub fn pre_parse(string: String, file: PathBuf, path: PathBuf) -> Vec<(Position, String)> {
    let lines: Vec<(u32, String)> = string.lines().fold(vec![], |mut count, line| {
        count.push(((count.len() + 1) as u32, line.to_string()));
        count
    });

    let (unclosed, pos, _, lines): (bool, Position, String, Vec<(Position, String)>) = lines.iter()
        .filter(|line| line.1.len() > 1)
        .map(|line| {
            if !line.1.contains("//") {
                line.clone()
            } else {
                let pos = line.1.find("//").unwrap();
                let (left, _) = line.1.split_at(pos);
                (line.0, left.to_string())
            }
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

            if line_string.starts_with("\"") && line_string.ends_with("\"") {
                vec.push((position.clone(), line_string));
                change = false;
                append = String::new();
            } else {
                if !collect.0 {
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
    let rev = tokens.iter().map(|token| (token.clone().0, Token::from(token.clone()))).rev();
    let iter = rev.clone().fold(vec![], |mut acc, token| {
        acc.push((acc.len(), token));
        acc
    }).iter().fold(State::new(rev.clone().collect()), |mut acc, token| {
        acc.push(token.clone().1.clone().0, &token.1.clone().1, token.clone().0);
        acc
    });

    iter
}