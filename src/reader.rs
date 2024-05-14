use std::io::ErrorKind;

pub(crate) use async_trait::async_trait;
// use std::{fs::File, io::{stdin, BufRead, BufReader, Stdin}};
use tokio::{
    fs::File, io::{
        stdin, AsyncBufReadExt, BufReader, Stdin
    }, 
    sync::mpsc::Sender
};

#[async_trait]
pub trait Reader : Send { 
    async fn read_line(&mut self) -> Option<String>;
    async fn notify(&self, line: String);
    fn readable(&self) -> bool;
}

pub struct StdinReader {
    inner: Sender<String>, 
    reader: BufReader<Stdin>,
    eof: bool,
}

pub struct FileReader {
    inner: Sender<String>, 
    reader: BufReader<File>,
    eof: bool,
}

impl FileReader {  
    pub async fn new(filename: &String, sender: Sender<String>) -> FileReader{
        FileReader {
            reader: {
                BufReader::new(File::open(filename).await.expect(&format!("can't open {}", filename)))
            },
            eof: false,
            inner: sender,
        }
    }
}

#[async_trait]    
impl Reader for FileReader{
    async fn read_line(&mut self) -> Option<String>{
        let mut str = String::new();

        let res = self.reader.read_line(&mut str).await;
        match res {
            Ok(size) => {
                if size == 0 {
                    self.eof = true;
                    return None;
                }
            }
            Err(e) => { 
                // 输入了非法字符，比如非 utf-8 编码的字符
                if let ErrorKind::InvalidData = e.kind() {
                    return None;
                }
                panic!("read line error: {:?}", e);
            }
        };
        
        Some(str)    
    }

    async fn notify(&self, line: String) {
        self.inner.send(line).await.expect("Error: send error");
    }

    fn readable(&self) -> bool {
       !self.eof 
    }        
}

impl StdinReader{
    pub fn new(sender: Sender<String>) -> StdinReader{
        StdinReader{
            reader: BufReader::new(stdin()),
            eof: false,
            inner: sender,
        }
    }
}

#[async_trait]
impl Reader for StdinReader{
    async fn read_line(&mut self) -> Option<String> {
        let mut str = String::new();
        let res = self.reader.read_line(&mut str).await;

        match res {
            Ok(size) => {
                if size == 0 {
                    self.eof = true;
                    return None;
                }
            }
            Err(e) => { 
                // 输入了非法字符，比如非 utf-8 编码的字符
                if let ErrorKind::InvalidData = e.kind() {
                    return None;
                }
                panic!("read line error: {:?}", e);
            }
        };
        
        Some(str)    
    }
 
    async fn notify(&self, line: String) {
        self.inner.send(line).await.expect("Error: send error");
    }
    
    fn readable(&self) -> bool {
        !self.eof   
    }
}


pub struct MockReader{
    inner: Sender<String>
}

#[async_trait]
impl Reader for MockReader{
    async fn read_line(&mut self) -> Option<String> {
        todo!()
    }

    async fn notify(&self, line: String) {
        self.inner.send(line).await.expect("Error: send error");
    }

    fn readable(&self) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests{

    #[should_panic]
    #[tokio::test]
    async fn test_file_no_exist() {
        let binding = String::from("./tests/noexist.txt");
        let (tx, _rx) = tokio::sync::mpsc::channel::<String>(4);
        let promise = crate::reader::FileReader::new(&binding, tx);
        promise.await;
    }
}


