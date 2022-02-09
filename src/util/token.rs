use crate::util::{compiler_error, compiler_error_str, compiler_warning};
use crate::util::position::Position;
use crate::util::token::Keyword::{End, INCLUDE};
use crate::util::types::Types;

static KEYWORDS: [&'static str; 2] = [
    "include",
    "end"
];

#[derive(Clone, Debug)]
pub enum TokenType {
    Word,
    Int,
    Str,
    Keyword,
    Function,
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
        if KEYWORDS.contains(&str.1.as_str()) {
            Self {
                typ: TokenType::Keyword,
                text: str.clone().1,
                location: str.0,
                value: TokenValue::Keyword(match str.1.as_ref() {
                    "include" => INCLUDE,
                    "end" => End,
                    _ => unreachable!()
                }),
            }
        } else if str.1.starts_with("@") && str.1.ends_with(")") {
            let mut token = str.1.clone();
            let token_raw = token.clone();
            token.remove(0);
            token.remove(token.len() - 1);

            if token.matches("(").count() != 1 {
                compiler_error(format!("Invalid function declaration. Function name cant contain '('. Got function decleration: {}", str.clone().1), str.clone().0);
            }

            let parts = token.split_once("(").unwrap();
            let name = parts.0.to_string();

            let params = parts.1.to_string();
            let params = params.replace(")", "");

            let (input, output, _) = if params.len() > 0 {
                parts.1.split("->").map(|part| part.split(",").filter(|sub_part| {
                    sub_part.len() > 0
                }).map(|sub_part| {
                    Types::from((str.clone().0, sub_part.to_string()))
                }).collect::<Vec<Types>>()).fold((vec![], vec![], 0), |acc, typ| {
                    if acc.2 == 1 {
                        let mut outp = vec![];
                        outp.extend(typ);
                        (acc.0, outp, 2)
                    } else if acc.2 == 0 {
                        let mut inp = vec![];
                        inp.extend(typ);
                        (inp, vec![], 1)
                    } else {
                        compiler_error_str("Only one pipe operator allowed in function input/output declearation", str.clone().0);
                        unreachable!()
                    }
                })
            } else {
                (vec![], vec![], 0)
            };

            Self {
                typ: TokenType::Function,
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
                                compiler_warning(format!("Invalid escape sequence \\{}", char), str.clone().0);
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