#![feature(panic_info_message)]
#![feature(trusted_random_access)]
#![feature(once_cell)]
#![feature(fn_traits)]
#![feature(trivial_bounds)]
#![feature(path_try_exists)]

use std::fs;
use std::fs::OpenOptions;
use std::io::{Read, stdin, stdout, Write};
use std::path::PathBuf;
use std::process::exit;

use backtrace::Backtrace;
use clap::Parser;

use crate::args::{Action, Args};
use crate::parser::{pre_parse, tokenize};
use crate::util::{compiler_error, compiler_error_str};
use crate::util::compile::{CompiledProgram, ProgramMetadata};
use crate::util::operation::OperationDataInfo;
use crate::util::position::Position;
use crate::vm::VM;

pub mod args;
pub mod parser;
pub mod util;
pub mod vm;
pub mod opt;

fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!();
        eprintln!("----- Error -----");
        if cfg!(debug_assertions) {
            let current_backtrace = Backtrace::new();
            eprintln!("Backtrace");
            eprintln!("{:?}", current_backtrace);
        } else {
            eprintln!("Backtrace removed. Run in debug mode to show")
        }
        eprintln!("{:?}", panic_info.message().ok_or_else(|| {}).map_err(|_| "No message provided").unwrap());
        eprintln!("----- Error -----");
    }));
    let args: Args = Args::parse();

    match args.action {
        Action::Simulate(simulate_options) => {
            let file_text = {
                let file_name = &simulate_options.file;
                let file = OpenOptions::new().read(true).open(file_name).map_err(|err| format!("Could not open file {} to read from: {}", file_name, err));

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

            let file_path = PathBuf::from(&simulate_options.file);
            let path = file_path.clone().parent().unwrap().to_path_buf();

            let pre_parsed = pre_parse(file_text, file_path, path.clone());
            let parsed = tokenize(pre_parsed, 0, path, None);

            let checked = parsed.type_check();

            if checked.is_ok() {
                checked.unwrap().run();
            } else {
                let error = checked.err().unwrap();
                compiler_error(format!("Type check failed:\r\n\t{}", error), &OperationDataInfo::None);
            }
        }
        Action::Compile(compiler_options) => {
            let file_text = {
                let file_name = &compiler_options.file;
                let file = OpenOptions::new().read(true).open(file_name).map_err(|err| format!("Could not open file {} to read from: {}", file_name, err));

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

            let meta = {
                let file_name = &compiler_options.meta_path;
                let file = OpenOptions::new().read(true).open(file_name).map_err(|err| format!("Could not open file {} to read from: {}", file_name, err));

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

            let file_path = PathBuf::from(compiler_options.file.clone());
            let path = file_path.clone().parent().unwrap().to_path_buf();

            let pre_parsed = pre_parse(file_text, file_path, path.clone());
            let parsed = tokenize(pre_parsed, 0, path, Some(compiler_options.clone()));

            let meta = serde_yaml::from_str(&meta);
            if meta.is_err() {
                compiler_error_str("Your meta is invalid", &OperationDataInfo::None);
            }

            let byte_code = parsed.compile(meta.unwrap(), *&compiler_options.readable);
            if byte_code.is_err() {
                compiler_error_str("Could not compile into bytecode", &OperationDataInfo::None);
            }

            let byte_code = byte_code.unwrap();

            let file_path = compiler_options.out_file.clone();
            let file_path = PathBuf::from(file_path);
            let file = OpenOptions::new().write(true).truncate(true).create(true).open(file_path);
            if file.is_err() {
                compiler_error_str("Could not open file", &OperationDataInfo::None);
            }

            let mut file = file.unwrap();
            let success = file.write_all(&byte_code);
            if success.is_err() {
                compiler_error_str("Could not write file", &OperationDataInfo::None);
            }

            println!("Sucessfully compiled file");
        }
        Action::Interpret(interpreter_options) => {
            let file_bytes = {
                let file_name = &interpreter_options.file;
                let file = OpenOptions::new().read(true).open(file_name).map_err(|err| format!("Could not open file {} to read from: {}", file_name, err));

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

            let compiled_program = bincode::deserialize::<CompiledProgram>(&file_bytes);
            if compiled_program.is_err() {
                compiler_error(format!("Could not program from file. {}", compiled_program.err().unwrap()), &OperationDataInfo::None);
            }

            let compiled_program = compiled_program.unwrap();
            let mut vm = VM::from(compiled_program);
            vm.run();
        }
        Action::Info(info_options) => {
            let file_bytes = {
                let file_name = &info_options.file;
                let file = OpenOptions::new().read(true).open(file_name).map_err(|err| format!("Could not open file {} to read from: {}", file_name, err));

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

            let compiled_program = bincode::deserialize::<CompiledProgram>(&file_bytes);
            if compiled_program.is_err() {
                compiler_error(format!("Could not program from file. {}", compiled_program.err().unwrap()), &OperationDataInfo::None);
            }

            let compiled_program = compiled_program.unwrap();

            let meta = compiled_program.data.clone();

            println!("Program name: {}", meta.name);
            println!("Program version: {}", meta.version);
            println!("Program author: {}", meta.author.as_ref().unwrap_or(&"Unknown".to_string()));
            println!("Program description: {}", meta.description.as_ref().unwrap_or(&"Unknown".to_string()));

            if info_options.extract_path.is_some() {
                let extract_path = info_options.extract_path.unwrap();
                let extract_path = PathBuf::from(extract_path);
                let mut file = OpenOptions::new().write(true).truncate(true).create(true).open(extract_path);
                if file.is_err() {
                    compiler_error_str("Could not open file", &OperationDataInfo::None);
                }

                let deserialized_meta = serde_yaml::to_string(&meta).unwrap();
                file.unwrap().write_all(deserialized_meta.as_ref()).expect("file to be written");
            }
        }
        Action::New(new_options) => {
            let root_path = new_options.path.clone();
            let root_path = PathBuf::from(root_path);
            let mut pkg_root = root_path.clone();
            pkg_root.push(format!("{}-scy", new_options.name.clone()));

            if fs::try_exists(&pkg_root).unwrap() {
                let mut buf = String::new();
                print!("Do you want to delete the folder {:?} [Yes/Y/yes/y]: ", &pkg_root);
                stdout().flush();
                stdin().read_line(&mut buf).expect("input");
                const AFFIRMATIVES: [&'static str; 4] = ["Yes", "Y", "yes", "y"];
                if AFFIRMATIVES.contains(&buf.trim()) {
                    fs::remove_dir_all(&pkg_root).expect("directory to be deleted");
                } else {
                    panic!("Could not create project");
                }
            }

            let create = fs::create_dir(&pkg_root);
            if create.is_err() {
                panic!("Could not create directory")
            }

            let mut meta_path = pkg_root.clone();
            meta_path.push(format!("{}-meta.scy.yml", new_options.name.clone()));

            let example_meta = ProgramMetadata {
                name: new_options.name.clone(),
                version: "1.0".to_string(),
                author: None,
                description: None
            };
            let file_data = serde_yaml::to_string(&example_meta).expect("metadata to be serialized");

            let mut file = OpenOptions::new().write(true).truncate(true).create(true).open(meta_path).expect("metadata to be written");
            file.write_all(file_data.as_ref()).expect("Meta to be written");

            let mut main_path = pkg_root.clone();
            main_path.push(format!("{}-main.scy", new_options.name.clone()));

            let file_data = format!(r#"// Main function for {}
@main(->int)
    "Hello {}" println
    0
end"#, new_options.name.clone(), new_options.name.clone());

            let mut file = OpenOptions::new().write(true).truncate(true).create(true).open(main_path).expect("source to be written");
            file.write_all(file_data.as_ref()).expect("source to be written");

            println!("Created project {} successfully", new_options.name.clone());
        }
    }
}
