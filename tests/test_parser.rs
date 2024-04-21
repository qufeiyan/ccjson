use ccjson::{parser, reader::FileReader};

#[test]
fn test_parseable(){
    let file = FileReader::new(&String::from("./tests/build.log"));

    let mut parser: parser::Parser = parser::Parser::new(
        Box::new(file), 
        Some(String::from("./"))
    );

    assert_eq!(parser.parserable(), true);
    parser.parse_line();
    parser.parse_line();
    assert_eq!(parser.parserable(), true);
}

#[test]
fn test_parse_line(){
    let file = FileReader::new(&String::from("./tests/build.log"));

    let parser: parser::Parser = parser::Parser::new(
        Box::new(file), 
        Some(String::from("./"))
    );

    // parser.next();
    // parser.next();
    let mut times = 0;
    for _ in parser{
        println!("iter: {}", times);
        times += 1;
    } 

    assert_eq!(times, 1);
}