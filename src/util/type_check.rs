use std::collections::HashMap;
use std::lazy::SyncLazy;

use crate::util::{compiler_error, compiler_error_str};
use crate::util::position::Position;
use crate::util::type_check::Types::{Bool, Function, Int, Pointer, String as TString};

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
    Int,
    String,
    Bool,
    Pointer,
    Function,
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