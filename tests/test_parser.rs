use ccjson::{parser, reader::{FileReader, Reader}};
use tokio_stream::StreamExt;

#[tokio::test]
async fn test_parseable(){
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(4);
    let mut file = FileReader::new(&String::from("./tests/build.log"), tx).await;

    let _ = tokio::spawn(async move {
        while let Some(line) = file.read_line().await {
            file.notify(line).await;
        }
    });

    let mut parser: parser::Parser = parser::Parser::new(
        rx, 
        Some(String::from("./"))
    );

    assert_eq!(parser.parserable(), true);
    parser.parse_line().await;
    parser.parse_line().await;
    assert_eq!(parser.parserable(), true);
}

#[tokio::test]
async fn test_parse_line(){
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(4);
    let mut file = FileReader::new(&String::from("./tests/build.log"), tx).await;

    let _ = tokio::spawn(async move {
        while let Some(line) = file.read_line().await {
            file.notify(line).await;
        }
    });

    let parser: parser::Parser = parser::Parser::new(
        rx, 
        Some(String::from("./"))
    );
    // parser.next();
    let stream = parser.into_stream();
    let mut times = 0;
    tokio::pin!(stream);
    while let Some(_line) = stream.next().await {
        println!("iter: {}", times);
        times += 1;
    } 

    assert_eq!(times, 2);
}