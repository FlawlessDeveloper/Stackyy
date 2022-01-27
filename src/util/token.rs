use crate::util::{compiler_error, compiler_error_str, compiler_warning};
use crate::util::position::Position;

static KEYWORDS: [&'static str; 1] = [
    "include"
];

#[derive(Clone, Debug)]
pub enum TokenType {
    Word,
    Int,
    Str,
    Keyword,
}

#[derive(Clone, Debug)]
pub enum TokenValue {
    Int(i32),
    String(String),
    Keyword(Keyword),
}


#[derive(Clone, Debug)]
pub enum Keyword {
    INCLUDE,
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
                    "include" => Keyword::INCLUDE,
                    _ => unreachable!()
                }),
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