use std::collections::HashMap;
use std::fmt::{Display, Formatter, write};
use std::lazy::SyncLazy;
use std::rc::Rc;
use std::sync::Mutex;

use crate::util::{compiler_error, compiler_error_str};
use crate::util::operation::OperationDataInfo;
use crate::util::operations::Descriptor as TDescriptor;
use crate::util::position::Position;

static TYPES_MAP: SyncLazy<HashMap<String, Types>> = SyncLazy::new(|| {
    let mut map = HashMap::new();

    map.insert("int".to_string(), Types::Int);
    map.insert("str".to_string(), Types::String);
    map.insert("bool".to_string(), Types::Bool);
    map.insert("ptr".to_string(), Types::Pointer);
    map.insert("fn".to_string(), Types::Function);
    map.insert("rsc".to_string(), Types::Descriptor);

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
    Descriptor,
}

impl Into<String> for Types {
    fn into(self) -> String {
        match self {
            Types::Any => ("any".to_string()),
            Types::Int => ("int".to_string()),
            Types::String => ("str".to_string()),
            Types::Bool => ("bool".to_string()),
            Types::Pointer => ("ptr".to_string()),
            Types::Function => ("fn".to_string()),
            Types::FunctionPointer(inp, outp) => {
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
            Types::Descriptor => {
                "rsc".to_string()
            }
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum ErrorTypes {
    None,
    TooFewElements,
    WrongData,
    InvalidTypes,
    ClosureError,
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

    pub fn into_txt(self, expl: &'static str) -> TypeCheckError {
        TypeCheckError {
            error: self,
            wanted: vec![],
            got: vec![],
            expl,
        }
    }
}

#[derive(Debug)]
pub struct TypeCheckError {
    pub error: ErrorTypes,
    pub wanted: Vec<Types>,
    pub got: Vec<Types>,
    pub expl: &'static str,
}

impl TypeCheckError {
    pub fn is_error(&self) -> bool {
        if ErrorTypes::None == self.error {
            false
        } else {
            true
        }
    }
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
            ErrorTypes::ClosureError => {
                "Could not create closure for operation".to_string()
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
            compiler_error(format!("Invalid type: {}", token.1), &OperationDataInfo::Position(token.clone().0));
            unreachable!()
        }
    }
}