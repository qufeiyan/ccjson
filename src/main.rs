use ccjson::{parser, reader::{FileReader, StdinReader}, writer::Writer};
use clap::Parser;

/// Generate a compilation database for make-based build systems.
#[derive(Parser, Debug)]
#[command(version, about, long_about = "Features:
    -------------------------------------------------------------------
    1. Support for redundent build systems that use shell scripts to nest make.
    2. Simpler to use than similiar tools without losing compilation information.
    3. Open source, you can modify it according to the actual build situation.
    -------------------------------------------------------------------
   | Recommended usage:                                                |
   | 1. sh -x ${build.sh} | ${ccjson} -d ${build_dir}                  |
   | 2. ${ccjson} -p ${build.log} -d ${build_dir}                      |
    -------------------------------------------------------------------
    Check out at https://github.com/qufeiyan/ccjson for more details"
)]
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

    /// Specifies the command strings instead of arguments list for the compile_commands.json.
    #[arg(short, long)]
    command: bool,
}

fn main() {
    let args = Args::parse();

    let parser: parser::Parser = match args.parse {
        Some(p) => {
            let file: FileReader = FileReader::new(&p);
            ccjson::parser::Parser::new(Box::new(file), Some(args.directoy), args.command)
        },
        None => {
            ccjson::parser::Parser::new(Box::new(StdinReader::new()), Some(args.directoy), args.command)
        }
    };

    let writer = Writer::new(Some(&args.output), 256);
    ccjson::run(parser, writer);

}