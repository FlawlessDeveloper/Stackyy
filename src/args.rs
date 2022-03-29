use clap::Args as CArgs;
use clap::Parser;
use clap::Subcommand;

#[derive(Parser, Debug)]
#[clap(name = "Stackyy", author, version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    /// The action to perform
    pub action: Action,

}

#[derive(CArgs, Debug, Clone)]
pub struct Simulate {
    #[clap(short, long)]
    /// The file to simulate
    pub file: String,
}

#[derive(CArgs, Debug, Clone)]
pub struct Interpret {
    #[clap(short, long)]
    /// The file to interpret
    pub file: String,
}

#[derive(CArgs, Debug, Clone)]
pub struct New {
    #[clap(short, long)]
    /// Project name
    pub name: String,
    #[clap(short, long)]
    /// Project path
    pub path: String,
}

#[derive(CArgs, Debug, Clone)]
pub struct Info {
    #[clap(short, long)]
    /// The file to extract the information from
    pub file: String,

    #[clap(short, long)]
    /// If the yml should be extracted from the included metadata
    pub extract_path: Option<String>,
}

#[derive(CArgs, Debug, Clone)]
pub struct Compile {
    #[clap(short, long)]
    /// A path to the metadata for your program
    /// This is a yml file which looks like this:
    /// ---
    /// name: "ExampleProgram"
    /// version: "1.0"
    /// author: "Flawlesscode"
    ///
    /// All fields except the program name and the version are optional
    pub meta_path: String,

    #[clap(short, long, parse(from_occurrences))]
    /// To which level the debug symbols will be stripped.
    /// Level 0: Full token data
    /// Level 1: Only positional data
    /// Level 2: Totally stripped data
    pub strip_data: usize,

    #[clap(short, long)]
    /// How the bytecode should be generated.
    /// If toggled the output will be generated in yaml
    pub readable: bool,

    #[clap(short, long)]
    /// The file to perform to compile
    pub file: String,

    #[clap(short, long)]
    /// The output path of the compiled program.
    pub out_file: String,
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// Simulates the program
    Simulate(Simulate),
    /// Compiles the program into bytecode.
    /// Here you need the optional parameters: <out-file>,
    Compile(Compile),
    /// Interpret the byte code
    Interpret(Interpret),
    /// Dump the metadata of the program
    Info(Info),
    /// Create a new stackyy program
    New(New),
}