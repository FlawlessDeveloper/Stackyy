use std::collections::HashMap;
use std::lazy::SyncLazy;

use crate::util::{compiler_error, compiler_error_str};
use crate::util::position::Position;
use crate::util::token::TokenValue;
use crate::util::type_check::{ErrorTypes, TypeCheckError, Types};

static INTERNALS_MAP: SyncLazy<HashMap<&'static str, Internal>> = SyncLazy::new(|| {
    let mut map = HashMap::new();
    map.insert("noop", Internal::NoOp);
    map.insert("print", Internal::Print);
    map.insert("println", Internal::PrintLn);
    map.insert("to-string", Internal::ToString);
    map
});


static _INTERNALS_MAP: SyncLazy<HashMap<String, Internal>> = SyncLazy::new(|| {
    let mut map = HashMap::new();
    map
});


static REFLECTION_INTERNALS_MAP: SyncLazy<HashMap<&'static str, Internal>> = SyncLazy::new(|| {
    let mut map = HashMap::new();
    map.insert("ref-rem-str-drop", Internal::ReflectionRemoveStrDrop);
    map.insert("ref-rem-str", Internal::ReflectionRemoveStr);
    map.insert("ref-push", Internal::ReflectionPush);
    map.insert("ref-clear", Internal::ReflectionClear);
    map
});

static STACK_OPS_INTERNALS_MAP: SyncLazy<HashMap<&'static str, Internal>> = SyncLazy::new(|| {
    let mut map = HashMap::new();
    map.insert("swap", Internal::Swap);
    map.insert("drop", Internal::Drop);
    map.insert("swap", Internal::Swap);
    map.insert("dup", Internal::Dup);
    map.insert("rev-stack", Internal::RevStack);
    map.insert("drop-stack", Internal::DropStack);
    map.insert("dup-stack", Internal::DupStack);
    map.insert("dbg-stack", Internal::DbgStack);
    map
});

static BASIC_MATH_INTERNALS_MAP: SyncLazy<HashMap<&'static str, Internal>> = SyncLazy::new(|| {
    let mut map = HashMap::new();
    map.insert("+", Internal::Plus);
    map.insert("-", Internal::Minus);
    map.insert("*", Internal::Mult);
    map.insert("/", Internal::Div);
    map.insert("%", Internal::Modulo);
    map.insert("squared", Internal::Squared);
    map.insert("cubed", Internal::Cubed);
    map
});

static BOOL_INTERNALS_MAP: SyncLazy<HashMap<&'static str, Internal>> = SyncLazy::new(|| {
    let mut map = HashMap::new();
    map.insert("!", Internal::Not);
    map.insert("@!", Internal::NotPeek);
    map.insert("=", Internal::Equals);
    map.insert("<", Internal::Larger);
    map.insert(">", Internal::Smaller);
    map.insert("<=", Internal::LargerEq);
    map.insert(">=", Internal::SmallerEq);
    map
});

static INCLUDE_MAP: SyncLazy<HashMap<&'static str, &'static HashMap<&'static str, Internal>>> = SyncLazy::new(|| {
    let mut map = HashMap::new();
    map.insert("std/bool", &*BOOL_INTERNALS_MAP);
    map.insert("std/simple-maths", &*BASIC_MATH_INTERNALS_MAP);
    map.insert("std/stack-ops", &*STACK_OPS_INTERNALS_MAP);
    map.insert("std/reflection", &*REFLECTION_INTERNALS_MAP);
    map
});


#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Hash)]
pub enum Internal {
    NoOp,
    Print,
    PrintLn,
    ToString,
    Swap,
    Drop,
    Dup,
    RevStack,
    DropStack,
    DupStack,
    DbgStack,
    Plus,
    Minus,
    Mult,
    Div,
    Modulo,
    Squared,
    Cubed,
    Not,
    NotPeek,
    Equals,
    Larger,
    Smaller,
    LargerEq,
    SmallerEq,
    ReflectionRemoveStr,
    ReflectionRemoveStrDrop,
    ReflectionPush,
    ReflectionClear,
}

pub fn to_internal(includes: Vec<String>, str: &TokenValue, pos: Position) -> Internal {
    if let TokenValue::String(str) = str {
        let ops = includes.iter().fold(INTERNALS_MAP.clone(), |mut acc, include| {
            if let Some(include) = INCLUDE_MAP.get(include.as_str()) {
                acc.extend(include.clone())
            } else {
                compiler_error(format!("The system lib: {} was not found", include), pos.clone());
                unreachable!();
            }
            acc
        });

        if let Some(op) = ops.get(str.as_str()) {
            op.clone()
        } else {
            compiler_error(format!("The internal call {} does not exist or is not included", str), pos);
            unreachable!()
        }
    } else {
        compiler_error_str("Internal parser error occurred", pos);
        unreachable!()
    }
}