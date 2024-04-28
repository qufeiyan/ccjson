use ccjson::{parser, reader::{FileReader, StdinReader}, writer::Writer};
use clap::Parser;

/// Generate a compilation database for make-based build systems.
#[derive(Parser, Debug)]
#[command(version, about, long_about = "Features:\n
    1. Support for redundent build systems that use shell scripts to nest make.
    2. Simpler to use than similiar tools without losing compilation information.
    3. Open source, you can modify it according to the actual build situation.")]
struct Args {
    /// Build log file to parse compilation commands from. (Default: stdin)"
    #[arg(short, long)]
    parse: Option<String>,

    /// Specifies the build path for current project.
    #[arg(short, long, default_value_t = String::from("./"))]
    directoy: String,

    /// Specifies the directory for compile_commands.json.
    #[arg(short, long, default_value_t = String::from("./"))]
    output: String,

}

fn main() {
    let args = Args::parse();

    let parser: parser::Parser = match args.parse {
        Some(p) => {
            let file: FileReader = FileReader::new(&p);
            ccjson::parser::Parser::new(Box::new(file), Some(args.directoy))
        },
        None => {
            ccjson::parser::Parser::new(Box::new(StdinReader::new()), Some(args.directoy))   
        }
    };

    let writer = Writer::new(Some(&args.output), 256);

    ccjson::run(parser, writer);

}