use std::collections::HashMap;
use std::lazy::SyncLazy;

use crate::util::{compiler_error, compiler_error_str, compiler_warning};
use crate::util::operation::OperationDataInfo;
use crate::util::position::Position;
use crate::util::token::Keyword::{Call, CallIf, End, INCLUDE};
use crate::util::type_check::Types;

static KEY_WORD_MAP: SyncLazy<HashMap<String, Keyword>> = SyncLazy::new(|| {
    let mut map = HashMap::new();
    map.insert("include".to_string(), INCLUDE);
    map.insert("end".to_string(), End);
    map.insert("@".to_string(), Call);
    map.insert("@if".to_string(), CallIf);
    map
});

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum TokenType {
    Word,
    Int,
    Str,
    Keyword,
    Function,
    FunctionPtr,
}

#[derive(Clone, Debug)]
pub enum TokenValue {
    Int(i32),
    String(String),
    Keyword(Keyword),
    Function(String, Vec<Types>, Vec<Types>),
}


#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Keyword {
    INCLUDE,
    End,
    Call,
    CallIf,
}


#[derive(Clone, Debug)]
pub struct Token {
    typ: TokenType,
    text: String,
    location: Position,
    value: TokenValue,
}

impl Token {
    pub fn typ(&self) -> &TokenType {
        &self.typ
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn location(&self) -> &Position {
        &self.location
    }
    pub fn value(&self) -> &TokenValue {
        &self.value
    }
}

impl From<(Position, String)> for Token {
    fn from(str: (Position, String)) -> Self {
        let op_data_info = OperationDataInfo::Position(str.clone().0);

        if KEY_WORD_MAP.contains_key(&str.1) {
            Self {
                typ: TokenType::Keyword,
                text: str.clone().1,
                location: str.0,
                value: TokenValue::Keyword(KEY_WORD_MAP.get(&str.1).unwrap().clone()),
            }
        } else if (str.1.starts_with("@") && str.1.ends_with(")")) || (str.1.starts_with("#") && str.1.ends_with(")")) {
            let is_ptr = str.1.starts_with("#");
            let typ = if is_ptr {
                " pointer"
            } else {
                ""
            };

            let mut token = str.1.clone();
            let token_raw = token.clone();
            token.remove(0);
            token.remove(token.len() - 1);

            if token.matches("(").count() != 1 {
                compiler_error(format!("Invalid function{typ} declaration. Function{typ} name cant contain '('. Got function decleration: {}", str.clone().1, typ = typ, ), &op_data_info);
            }

            let parts = token.split_once("(").unwrap();
            let name = parts.0.to_string();

            let params = parts.1.to_string();
            let params = params.replace(")", "");

            let (input, output) = if params.len() > 0 {
                let parts = parts.1.split("->").collect::<Vec<&str>>();

                if parts.len() != 2 {
                    compiler_error(format!("Function{typ} {} can only have input & output", name.clone(), typ = typ), &op_data_info);
                }

                let input = parts.get(0).unwrap().to_string();
                let output = parts.get(1).unwrap().to_string();

                let input = if input.len() > 0 {
                    input.split(",").map(|inp| Types::from((str.clone().0, inp.to_string()))).collect::<Vec<Types>>()
                } else {
                    vec![]
                };

                let output = if output.len() > 0 {
                    output.split(",").map(|inp| Types::from((str.clone().0, inp.to_string()))).collect::<Vec<Types>>()
                } else {
                    vec![]
                };

                (input, output)
            } else {
                (vec![], vec![])
            };

            Self {
                typ: if is_ptr {
                    TokenType::FunctionPtr
                } else {
                    TokenType::Function
                },
                text: token_raw,
                location: str.0,
                value: TokenValue::Function(name, input, output),
            }
        } else if str.1.starts_with("\"") && str.1.ends_with("\"") {
            let local = str.clone().1.chars().fold(String::new(), |mut acc, char| {
                if char == '"' {
                    if acc.ends_with("\\") {
                        acc.pop().unwrap();
                        acc.push(char);
                    } else {}
                } else {
                    if acc.ends_with("\\") {
                        match char {
                            'n' => {
                                acc.pop().unwrap();
                                acc.push('\n');
                            }
                            't' => {
                                acc.pop().unwrap();
                                acc.push('\t');
                            }
                            _ => {
                                compiler_warning(format!("Invalid escape sequence \\{}", char), &op_data_info);
                                acc.push(char);
                            }
                        }
                    } else {
                        acc.push(char);
                    }
                }
                acc
            });
            Self {
                typ: TokenType::Str,
                text: str.1,
                location: str.0,
                value: TokenValue::String(local),
            }
        } else if let Ok(num) = str.1.parse::<i32>() {
            Self {
                typ: TokenType::Int,
                text: str.1,
                location: str.0,
                value: TokenValue::Int(num),
            }
        } else {
            Self {
                typ: TokenType::Word,
                text: str.clone().1,
                location: str.0,
                value: TokenValue::String(str.1),
            }
        }
    }
}