use clap::Args as CArgs;
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
    pub out_file: Option<String>,
}

#[derive(CArgs, Debug, Clone)]
pub struct Compile {
    #[clap(short, long)]
    /// The author of the compiled program
    pub author: String,

    #[clap(short, long)]
    /// The description of the compiled program
    pub description: Option<String>,

    #[clap(short, long, parse(from_occurrences))]
    /// To which level the debug symbols will be stripped.
    /// Level 0: Full token data
    /// Level 1: Only positional data
    /// Level 2: Totally stripped data
    pub strip_data: usize,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// Simulates the program
    Simulate,
    /// Compiles the program into bytecode.
    /// Here you need the optional parameters: <out-file>,
    Compile(Compile),
    /// Interpret the byte code
    Interpret,
    /// Dump the metadata of the program
    Info(Info),
    /// Create a new stackyy program
    New(New),
}