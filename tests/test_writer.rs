use std::{fs::{self, File}, io::{self, Read}};

use ccjson::writer::{self, Writer};


#[test]
fn test_write_content(){
    let mut writer = Writer::new(Some(&"./tests/".to_string()), 1);
    let src_string = "hello, writer!!!";

    writer.write(&src_string.to_string());
    
    let mut file = File::open("./tests/compile_commands.json").unwrap();
    let mut buffer = String::new();

    let buffer_size = file.read_to_string(&mut buffer).unwrap(); 
    let src_size = src_string.len();
    assert_eq!(buffer_size, src_size);
    let _ = fs::remove_file("./tests/compile_commands.json");
    assert_eq!(buffer, src_string.to_string());
}