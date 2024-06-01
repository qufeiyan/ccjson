use std::{fs::File, io::{stdin, BufRead, BufReader, ErrorKind, Stdin}};

pub trait Reader {
    fn read_line(&mut self) -> Option<String>;
    fn readable(&self) -> bool;
}

pub struct StdinReader { 
    reader: BufReader<Stdin>,
    eof: bool,
}

pub struct FileReader {
    reader: BufReader<File>,
    eof: bool,
}

impl FileReader {  
    pub fn new(filename: &String) -> FileReader{
        FileReader {
            reader: BufReader::new(File::open(filename).unwrap_or_else(|_| panic!("can't open {}", filename))),
            eof: false,
        }
    }
}

    
impl Reader for FileReader{
    fn read_line(&mut self) -> Option<String>{
        let mut str = String::new();
        let res = self.reader.read_line(&mut str);

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

    fn readable(&self) -> bool {
       !self.eof 
    }        
}

impl StdinReader{
    pub fn new() -> StdinReader{
        StdinReader{
            reader: BufReader::new(stdin()),
            eof: false,
        }
    }
}

impl Default for StdinReader{
    fn default() -> Self {
        Self::new()
    }
}

impl Reader for StdinReader{
    fn read_line(&mut self) -> Option<String> {
        let mut str = String::new();
        let res = self.reader.read_line(&mut str);

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
 
    fn readable(&self) -> bool {
        !self.eof   
    }
}


pub struct MockReader();

impl Reader for MockReader{
    fn read_line(&mut self) -> Option<String> {
        todo!()
    }

    fn readable(&self) -> bool {
        todo!()
    }
}


