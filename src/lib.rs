pub mod reader;
pub mod parser;
pub mod writer;


pub fn run(parser: parser::Parser, mut writer: writer::Writer){
    if !parser.parserable() {
        ()
    }    
    
    writer.write("[\n");
    for items in parser{
        writer.write(&items[2..items.len() - 2]);
    }
    writer.write("\n]")
}


