use crate::{compiler_error, Position};

pub fn resolve_opt(str: &str) -> String {
    if !str.starts_with("opt/") {
        compiler_error(format!("Invalid include path: {}", str), Position::default());
        unreachable!()
    }

    let mut str = str.to_string();
    str = str.replace("opt/", "");


    let incl = match str.as_str() {
        "logging" => include_str!("logging.scy"),
        _ => {
            compiler_error(format!("Invalid optional include path: {}", str), Position::default());
            unreachable!()
        }
    };

    incl.to_string()
}