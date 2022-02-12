use std::collections::HashMap;
use std::fmt::{Display, Formatter, write};
use std::lazy::SyncLazy;

use crate::util::{compiler_error, compiler_error_str};
use crate::util::position::Position;
use crate::util::type_check::Types::{Bool, Function, FunctionPointer, Int, Pointer, String as TString};

static TYPES_MAP: SyncLazy<HashMap<String, Types>> = SyncLazy::new(|| {
    let mut map = HashMap::new();

    map.insert("int".to_string(), Int);
    map.insert("str".to_string(), TString);
    map.insert("bool".to_string(), Bool);
    map.insert("ptr".to_string(), Pointer);
    map.insert("fn".to_string(), Function);

    map
});

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Types {
    Any,
    Int,
    String,
    Bool,
    Pointer,
    Function,
    FunctionPointer(Vec<Types>, Vec<Types>),
}

impl Into<String> for Types {
    fn into(self) -> String {
        match self {
            Any => format!("any"),
            Int => format!("int"),
            Types::String => format!("str"),
            Bool => format!("bool"),
            Pointer => format!("ptr"),
            Function => format!("fn"),
            FunctionPointer(inp, outp) => {
                let inp = inp.iter().fold(String::new(), |mut acc, wanted| {
                    acc.push_str(",");
                    let str: String = wanted.clone().into();
                    acc.push_str(&str);
                    acc
                }).replacen(",", "", 1);
                let out = outp.iter().fold(String::new(), |mut acc, wanted| {
                    acc.push_str(",");
                    let str: String = wanted.clone().into();
                    acc.push_str(&str);
                    acc
                }).replacen(",", "", 1);
                format!("fn-ptr({}->{})", inp, out)
            }
        }
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum ErrorTypes {
    None,
    TooFewElements,
    WrongData,
    InvalidTypes,
    Raw(String),
}

impl Into<TypeCheckError> for ErrorTypes {
    fn into(self) -> TypeCheckError {
        TypeCheckError {
            error: self,
            wanted: vec![],
            got: vec![],
            expl: "",
        }
    }
}

impl ErrorTypes {
    pub fn into_with_ctx(self, wanted: Vec<Types>, got: Vec<Types>) -> TypeCheckError {
        TypeCheckError {
            error: self,
            wanted,
            got,
            expl: "",
        }
    }

    pub fn into_with_ctx_plus(self, wanted: Vec<Types>, got: Vec<Types>, expl: &'static str) -> TypeCheckError {
        TypeCheckError {
            error: self,
            wanted,
            got,
            expl,
        }
    }
}

pub struct TypeCheckError {
    pub error: ErrorTypes,
    pub wanted: Vec<Types>,
    pub got: Vec<Types>,
    pub expl: &'static str,
}

impl Display for TypeCheckError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", format_args!("Error message: {}", self.error))?;

        let wanted = self.wanted.iter().fold(String::new(), |mut acc, wanted| {
            acc.push_str(",");
            let str: String = wanted.clone().into();
            acc.push_str(&str);
            acc
        }).replacen(" ,", "", 1);

        write!(f, "\t\t")?;
        if wanted.len() != 0 {
            if self.expl.len() == 0 {
                writeln!(f, "{}", format_args!("Wanted: {}", wanted))?;
            } else {
                writeln!(f, "{}", format_args!("Wanted: {} | {}", wanted, self.expl))?;
            }
        } else {
            writeln!(f, "{}", format_args!("Wanted: Empty"))?;
        }

        write!(f, "\t\t")?;
        let got = self.got.iter().fold(String::new(), |mut acc, wanted| {
            acc.push_str(",");
            let str: String = wanted.clone().into();
            acc.push_str(&str);
            acc
        }).replacen(" ,", "", 1);
        if wanted.len() != 0 {
            writeln!(f, "{}", format_args!("Got: {}", got))?;
        } else {
            writeln!(f, "{}", format_args!("Got: Empty"))?;
        }
        Ok(())
    }
}

impl Display for ErrorTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format_args!("{}", match self {
            ErrorTypes::None => {
                "None".to_string()
            }
            ErrorTypes::TooFewElements => {
                "To few elements on the stack at that point".to_string()
            }
            ErrorTypes::InvalidTypes => {
                "Incompatible elements on the stack at that point".to_string()
            }
            ErrorTypes::WrongData => {
                "Stack contains invalid data at that point".to_string()
            }
            ErrorTypes::Raw(str) => {
                str.clone()
            }
        }))
    }
}

impl From<(Position, String)> for Types {
    fn from(token: (Position, String)) -> Self {
        if TYPES_MAP.contains_key(&token.1) {
            TYPES_MAP.get(&token.1).unwrap().clone()
        } else {
            compiler_error(format!("Invalid type: {}", token.1), token.0);
            unreachable!()
        }
    }
}