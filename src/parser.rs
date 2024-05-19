use std::{env, fs, path};
use crate::reader::Reader;
use serde_json::{Map, Value};
use async_stream;
use tokio::sync::mpsc::Receiver;
use tokio_stream::Stream;


pub struct Slot{
    rx: Receiver<String>,
}

impl Slot{
    fn new(rx: Receiver<String>) -> Slot{
        Slot{
            rx
        }
    }

    /// This method returns `None` if the channel has been closed and there are
    /// no remaining messages in the channel's buffer
    async fn read_line(&mut self) -> Option<String> {
        self.rx.recv().await
    }

    fn readable(&self) -> bool {
        self.rx.is_closed() == false || self.rx.is_empty() == false
    }
}
pub struct Parser{
    slot: Slot,
    build_dir: String,
    directory: String
}

impl Parser{
    pub fn new(rx: Receiver<String>, dir: Option<String>) -> Parser{
        let build_dir = match dir {
            Some(s) => {
                if !s.starts_with('/'){
                    // 判断 s 组成的路径是否存在，如果不存在，panic
                    fs::canonicalize(path::Path::new(&s)).unwrap().to_str().unwrap().to_string()
                }else {
                    Parser::norm_path(&s)
                }
            }
            None => env::current_dir().unwrap().into_os_string().into_string().unwrap(), 
        }; 
        let directory = build_dir.clone();
        Parser{
            slot: Slot::new(rx),
            build_dir,
            directory, 
        }
    }

    pub fn parserable(&self) -> bool{
        return self.slot.readable();
    }

    fn parse_directory(&mut self, str: &String) -> Option<bool>{
        let get_directory = |str: &String| -> String{
            let strs = str.split_whitespace();
            let res: Vec<&str> = strs.into_iter()
                .filter(|s| s.contains("'/") || s.starts_with('/'))
                .collect();
            assert_eq!(res.len(), 1);
            match res[0].starts_with('\'') {
                true => {
                    let len = res[0].len();
                    res[0][1..len-1].to_string()
                }
                false => res[0].to_string()
            }
        };
        if str.contains("Make[1]") && str.contains("Entering directory") || str.contains("+ cd"){
            self.directory = get_directory(str).to_string();
            Some(true)
        }else if str.contains("Make[1]") && str.contains("Leaving directory"){
            let res: String = get_directory(str);
            assert_eq!(res, self.directory);
            self.directory.clear();
            Some(false)
        }else {
            // println!("Error: {}:{} something goes wrong in \"{}\"", file!(), line!(), str);
            None
        }
    }  

    pub async fn parse_line(&mut self) -> Option<String>{
        if self.slot.readable() == false {
            return None;   
        }
        let line = match self.slot.read_line().await {
            Some(line) => line,
            None => return None
        };
            
        let res: Option<bool> = self.parse_directory(&line);
        match res {
            Some(_) => None,
            None => {
                let res = self.parser_command(&line)?;
                Some(res)
            }
        }
    }

    /// Parse build command from a string line.
    /// 
    /// # Examples
    /// 
    /// 
    /// ```no run
    /// let file = FileReader::new(&String::from("build.log"));
    /// let mut parser: parser::Parser = parser::Parser::new(
    ///    Box::new(file), 
    ///    Some(String::from("./"))
    /// );
    /// 
    /// let res = self.parser_command(&parser);
    /// 
    /// ``` 
    fn parser_command(&mut self, line_str: &str) -> Option<String> {
        let mut iter = line_str.split_whitespace();
        let mut iter_copy = iter.clone();

        let res_cc = iter.find(|s|{
            s.ends_with("gcc") 
            || s.ends_with("g++") 
            || s.ends_with("clang")
            || s.ends_with("clang++")
        });
        let cc = match res_cc {
            Some(cc) => cc,
            None => return  None
        };

        let files: Vec<&str> = iter.filter(|s|{
            s.ends_with(".c") 
            || s.ends_with(".cc") 
            || s.ends_with(".cpp") 
            || s.ends_with("cxx")
        }).collect();
        if files.is_empty(){
            return None;
        }

        let mut args:Vec<Value> = Vec::new();
        while let Some(s) = iter_copy.next() {
            if s.eq("-I") || s.eq("-D"){
                args.push(Value::String(s.to_owned() + iter_copy.next()?))
            }else if s.starts_with("-I") || s.starts_with("-D") {
                args.push(Value::String(s.to_string())); 
            }
        }

        let mut map = Map::new(); 
       
        // directory: "~/..."
        if self.directory.is_empty(){
            self.directory.clone_from(&self.build_dir);
        }
        map.insert("directory".to_string(), Value::String(self.directory.clone()));

        // arguments: "-I... -D..."
        args.insert(0, Value::String(cc.to_string()));
        let value_args = Value::Array(args);
        map.insert("arguments".to_string(), value_args);

        // file: "*.c" 
        let items: Vec<Map<String, Value>> = files.iter().map(|s|{
            let mut map = map.clone();
            let abs_file = self.absolute_path(s);
            let file_val = Parser::relative_path(
                // s,
                &abs_file,
                &self.directory 
            );
            map.insert("file".to_string(), Value::String(file_val));
            // println!("{:#?}", map);  
            map
        }).collect();

        let s = serde_json::to_string_pretty(&items).unwrap();
        Some(s)      
    }

    fn norm_path(path: &str) -> String{
        let path_items: Vec<_> = path.split('/').collect();
        let initial_slashs = match path.starts_with('/') {
            true =>{
                if path.starts_with("//") && !path.starts_with("///"){ 2 } else { 1 }
            },
            false => 0
        }; 

        let mut new_path_items: Vec<&str> = Vec::new();       
        for item in path_items{
            if ["", "."].iter().any(|s| *s == item){ continue; }
            
            let lens = new_path_items.len(); 
            if item != ".." 
                || (initial_slashs == 0 && lens == 0) 
                || (lens > 0 && new_path_items[lens - 1] == ".."){
                new_path_items.push(item);
            }else {
                new_path_items.pop();
            }
        }

        let mut new_path: String = new_path_items.iter()
            .map(|&s| [s, "/"]
            .concat())
            .collect();
        new_path.pop(); // remove last "/" in {@line 161}
        
        if initial_slashs != 0 {
            return "/".repeat(initial_slashs) + &new_path;
        }

        new_path
    }

    fn absolute_path(&self, src_path: &str) -> String{
        let norm_src_path = Parser::norm_path(src_path);
        
        if norm_src_path.starts_with('/'){
            return norm_src_path;
        }
        
        match self.directory.as_str() {
            "" => panic!("Error: directory is not exits."),
            _ => {
                let mut norm_dir = Parser::norm_path(&self.directory);
                let _ = &norm_dir.push('/');
                Parser::norm_path(&[norm_dir, norm_src_path].concat())
            }
        }
    }

    fn relative_path(src_path: &str, base_path: &str) -> String{
        let abs_src = Parser::norm_path(src_path);
        let mut abs_base = Parser::norm_path(base_path);

        if !abs_base.ends_with('/'){
            abs_base.push('/');
        }

        String::from(abs_src.strip_prefix(&abs_base).unwrap())
    }

    
}

impl Parser{
    pub fn into_stream(mut self) -> impl Stream<Item = String> {
        async_stream::stream! {
            loop{
                if let Some(message) = self.parse_line().await {
                    yield message;
                }
                
                if self.parserable() == false {
                    break;
                }
            }
        }
    }
}

#[test]
fn test_norm_path(){
    let src = "..//./../a//b/c/";
    let dst = Parser::norm_path(src);
    assert_eq!(std::path::PathBuf::from(src), std::path::PathBuf::from(&dst));

    let src = "../a/./b/c//";
    let dst = Parser::norm_path(src);
    assert_eq!(std::path::PathBuf::from(src), std::path::PathBuf::from(&dst));

    let src = "..///..";
    let dst = Parser::norm_path(src);
    assert_eq!(std::path::PathBuf::from(src), std::path::PathBuf::from(&dst));
    
    // let src = "//./../a/../..//b/c/";
    // let dst = Parser::norm_path(src);
    // assert_eq!(std::path::PathBuf::from(src), std::path::PathBuf::from(&dst));
    
    // let src = "///./../a//b//./c/";
    // let dst = Parser::norm_path(src);
    // assert_eq!(std::path::PathBuf::from(src), std::path::PathBuf::from(&dst));
}

#[test]
fn test_relative_path(){
    let src_path = "../././tests/code//config.txt";
    let base_path = "../tests/./../tests";

    assert_eq!(Parser::relative_path(src_path, base_path), "code/config.txt");
}

#[test]
fn test_parser_path(){
    let file = crate::reader::FileReader::new(&String::from("./tests/build.log"));

    let parser: Parser = Parser::new(
        Box::new(file), 
        Some(String::from("../"))
    );

    let src = Parser::norm_path(&[env::current_dir().unwrap().to_str().unwrap(), "/../"].concat());
    assert_eq!(parser.directory, src); 
}