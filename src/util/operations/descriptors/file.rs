use std::any::Any;
use std::fs::{File as StdFile, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

use crate::Position;
use crate::util::operations::descriptors::{Descriptor, DescriptorAction};
use crate::util::register_type::RegisterType;
use crate::util::runtime_error_str;
use crate::util::type_check::{ErrorTypes, TypeCheckError, Types};

#[derive(Debug)]
pub struct File {
    file: Option<StdFile>,
    reader: Option<Box<BufReader<File>>>,
    path: Option<String>,
}

impl File {
    pub fn new() -> Self {
        File { file: None, reader: None, path: None }
    }
}

impl Descriptor for File {
    fn action(&mut self, action: DescriptorAction, data: &mut Vec<RegisterType>) {
        match action {
            DescriptorAction::Open => {
                let path = data.pop().unwrap();
                if let RegisterType::String(path) = path {
                    self.file = Some(OpenOptions::new().read(true).write(true).create(true).open(&path).expect("file to be opened"));
                    self.path = Some(path);
                }
            }
            DescriptorAction::Close => {
                let file = self.file.as_mut().unwrap();
                file.flush().unwrap();
                self.file = None;
            }
            DescriptorAction::ToString => {
                let path_orig = self.path.as_ref().unwrap();
                let mut path = "FileDescriptor(".to_string();
                path.push_str(path_orig);
                path.push_str(")");
                data.push(RegisterType::String(path))
            }
            DescriptorAction::ReadAll => {
                let res = self.file.as_mut().map(|mut f| {
                    let mut buf = String::new();
                    f.read_to_string(&mut buf).unwrap();
                    buf
                }).unwrap();
                data.push(RegisterType::String(res))
            }
            DescriptorAction::WriteAll => {
                let str = data.pop().unwrap();
                if let RegisterType::String(str) = str.clone() {
                    self.file.as_mut().map_or_else(|| {
                        runtime_error_str("File write failed", Position::default())
                    }, |f| {
                        f.write_all(str.as_bytes()).unwrap();
                    });
                }
            }
        }
    }

    fn typecheck(&self, action: DescriptorAction, stack: &mut Vec<Types>) -> TypeCheckError {
        match action {
            DescriptorAction::Open => {
                if stack.len() == 0 {
                    ErrorTypes::TooFewElements.into_with_ctx(vec![Types::String], vec![])
                } else {
                    let tmp_stack = stack.clone();
                    let pop = stack.pop().unwrap();
                    if pop == Types::String {
                        ErrorTypes::None.into()
                    } else {
                        ErrorTypes::InvalidTypes.into_with_ctx(vec![Types::String], tmp_stack)
                    }
                }
            }
            DescriptorAction::Close => {
                ErrorTypes::None.into()
            }
            _ => {
                ErrorTypes::ClosureError.into_txt("Tried to typecheck without implementation")
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}