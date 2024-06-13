use ccjson::{reader::{FileReader, Reader, StdinReader}, writer::Writer};
use clap::Parser;
use tokio::sync::mpsc::channel;

/// Generate a compilation database for make-based build systems.
#[derive(Parser, Debug)]
#[command(version, about, long_about = "Features:
    -------------------------------------------------------------------
    1. Support for redundent build systems that use shell scripts to nest make.
    2. Simpler to use than similiar tools without losing compilation information.
    3. Open source, you can modify it according to the actual build situation.
    -------------------------------------------------------------------
    | Recommended usage: sh -x ${build.sh} | ${ccjson} -d ${build_dir} |
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
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let (tx, rx) = channel::<String>(10);
    let reader: Box<dyn Reader> = match args.parse {
        Some(p) => {
            Box::new(FileReader::new(&p, tx).await)
        },
        None => {
            Box::new(StdinReader::new(tx))
        }
    };
    
    let parser = ccjson::parser::Parser::new(rx, Some(args.directoy));
    let writer = Writer::new(Some(&args.output), 256).await;
    ccjson::run(reader, parser, writer).await;
}