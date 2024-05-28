use ccjson::reader::{FileReader, Reader};

#[should_panic]
#[test]
fn test_file_no_exist(){
    FileReader::new(&String::from("text.txt"));   
}

#[test]
fn test_read_line(){
    let mut file = FileReader::new(&String::from("./tests/hello.txt"));
    file.read_line();
    assert_eq!(file.read_line().unwrap(), "hello, ccjson!!!");
}

#[test]
fn test_readable(){
    let mut file = FileReader::new(&String::from("./tests/hello.txt"));
    assert_eq!(true, file.readable());
    file.read_line();
    file.read_line();
    assert_eq!(true, file.readable());
    file.read_line();
    assert_eq!(false, file.readable());
}