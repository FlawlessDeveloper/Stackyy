#![feature(panic_info_message)]
#![feature(trusted_random_access)]
#![feature(once_cell)]
#![feature(fn_traits)]
#![feature(trivial_bounds)]

use std::fs::OpenOptions;
use std::io::Read;
use std::path::PathBuf;
use std::process::exit;

use backtrace::Backtrace;
use clap::Parser;

use crate::args::{Action, Args};
use crate::parser::{pre_parse, tokenize};
use crate::util::{compiler_error, compiler_error_str};
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
        Action::Simulate => {
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
            let parsed = tokenize(pre_parsed, 0, path, None);

            let checked = parsed.type_check();

            if checked.is_ok() {
                checked.unwrap().run();
            } else {
                let error = checked.err().unwrap();
                compiler_error(format!("Type check failed:\r\n\t{}", error), &OperationDataInfo::None);
            }
        }
        Action::Compile(compiler_options) => {}
        Action::Interpret | Action::Info => {
            let _file_bytes = {
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
