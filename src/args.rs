use clap::Parser;
use clap::Subcommand;

#[derive(Parser, Debug)]
#[clap(name = "Stackyy", author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    /// The action to perform
    pub action: Action,

    #[clap(short, long)]
    /// The file to perform the action
    pub file: String,

    #[clap(short, long)]
    /// If compiling the output file
    pub out_file: Option<String>
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// Simulates the program
    Simulate,
    /// Compiles the program into bytecode.
    /// Here you need the optional parameters: <out-file>,
    Compile,
    /// Interpret the byte code
    Interpret,
    /// Dump the metadata of the program
    Info,
}