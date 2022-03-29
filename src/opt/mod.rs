use crate::{compiler_error, Position};
use crate::util::operation::OperationDataInfo;

pub fn resolve_opt(str: &str) -> String {
    let empty = OperationDataInfo::None;

    if !str.starts_with("opt/") {
        compiler_error(format!("Invalid include path: {}", str), &empty);
        unreachable!()
    }

    let mut str = str.to_string();
    str = str.replace("opt/", "");


    let incl = match str.as_str() {
        "logging" => include_str!("logging.scy"),
        "files" => include_str!("files.scy"),
        _ => {
            compiler_error(format!("Invalid optional include path: {}", str), &empty);
            unreachable!()
        }
    };

    incl.to_string()
}