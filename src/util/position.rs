use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Default, Clone, Debug)]
pub struct Position {
    pub(crate) token_pos_line: u32,
    pub(crate) token_pos_x: u32,
    pub(crate) file: PathBuf,
}


impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}:{}:{}", self.file, self.token_pos_line, self.token_pos_x)
    }
}