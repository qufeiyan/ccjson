use ccjson::reader::{FileReader, Reader};

#[should_panic]
#[tokio::test]
async fn test_file_no_exist(){
    let binding = String::from("./tests/noexist.txt");
    let (tx, _rx) = tokio::sync::mpsc::channel::<String>(4);
    let promise = FileReader::new(&binding, tx);
    promise.await;
}

#[tokio::test]
async fn test_read_line(){
    let (tx, _rx) = tokio::sync::mpsc::channel::<String>(4);
    let mut file = FileReader::new(&String::from("./tests/hello.txt"), tx).await;
    assert_eq!(file.read_line().await.unwrap(), "hello, ccjson!!!");
}

#[tokio::test]
async fn test_readable(){
    let (tx, _rx) = tokio::sync::mpsc::channel::<String>(4);
    let mut file = FileReader::new(&String::from("./tests/hello.txt"), tx).await;
    assert_eq!(true, file.readable());
    file.read_line().await.unwrap();
    file.read_line().await.unwrap();
    assert_eq!(false, file.readable());
}