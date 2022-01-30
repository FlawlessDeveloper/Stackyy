#![feature(panic_info_message)]

use std::fmt::Arguments;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::exit;

use backtrace::Backtrace;
use clap::Parser;

use crate::args::{Action, Args};
use crate::parser::{pre_parse, tokenize};
use crate::vm::VM;

pub mod args;
pub mod parser;
pub mod util;
pub mod vm;

fn main() {
    std::panic::set_hook(Box::new( |panic_info| {
        eprintln!("----- Error -----");
        let current_backtrace = Backtrace::new();
        eprintln!("Backtrace");
        eprintln!("{:?}", current_backtrace);
        eprintln!("{:?}", panic_info.message().ok_or_else(|| {}).map_err(|_| "No message provided").unwrap());
        eprintln!("----- Error -----");
    }));
    let args: Args = Args::parse();

    match args.action {
        Action::Simulate | Action::Compile => {
            let file_text = {
                let file_name = args.file.clone();
                let file = OpenOptions::new().read(true).open(&file_name).map_err(|err| format!("Could not open file {} to read from: {}", file_name, err));

                if let Err(msg) = file {
                    eprintln!("{}", msg);
                    exit(1);
                }

                let mut file = file.unwrap();

                let mut content = String::new();

                let sucess = file.read_to_string(&mut content).map_err(|err| format!("Could not read file {}: {}", file_name, err));

                if let Err(msg) = sucess {
                    eprintln!("{}", msg);
                    exit(1);
                }
                content
            };

            let file_path = PathBuf::from(args.file.clone());
            let path = file_path.clone().parent().unwrap().to_path_buf();

            let pre_parsed = pre_parse(file_text, file_path, path.clone());
            let parsed = tokenize(pre_parsed, false, path);

            if parsed.type_check() {
                let mut vm = VM::from(parsed);
                vm.run();
            }
        }
        Action::Interpret | Action::Info => {
            let file_bytes = {
                let file_name = args.file;
                let file = OpenOptions::new().read(true).open(&file_name).map_err(|err| format!("Could not open file {} to read from: {}", file_name, err));

                if let Err(msg) = file {
                    eprintln!("{}", msg);
                    exit(1);
                }

                let mut file = file.unwrap();

                let mut content = Vec::new();

                let sucess = file.read_to_end(&mut content).map_err(|err| format!("Could not read file {}: {}", file_name, err));

                if let Err(msg) = sucess {
                    eprintln!("{}", msg);
                    exit(1);
                }
                content
            };
        }
    }
}
