use std::{fs::File, io::{BufReader, BufRead}};

pub trait Reader {
    fn read_line(&mut self) -> Option<String>;
    fn readable(&self) -> bool;
}

pub struct StdinReader { 
    eof: bool,

}

pub struct FileReader {
    reader: BufReader<File>,
    eof: bool,
    
}

impl FileReader {  
    pub fn new(filename: &String) -> FileReader{
        FileReader {
            reader: BufReader::new(File::open(filename).expect(&format!("can't open {}", filename))),
            eof: false,
        }
    }
}

    
impl Reader for FileReader{
    fn read_line(&mut self) -> Option<String>{
        let mut str = String::new();
        let res = self.reader.read_line(&mut str);

        let size = match res {
            Ok(size) => size,
            Err(e) => {
                panic!("read line error: {:?}", e);
            }
        };

        if size == 0{
            self.eof = true;
            ()
        }

        assert_eq!(size, str.len());
        Some(str)    
    }

    fn readable(&self) -> bool {
       !self.eof 
    }        
}

impl Reader for StdinReader{
    fn read_line(&mut self) -> Option<String> {
        
        None
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


