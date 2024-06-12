use std::{env, fs, path};
use crate::reader::Reader;
use serde_json::{Map, Value};

pub struct Parser{
    reader: Box<dyn Reader>,
    build_dir: String,
    directory: String
}

impl Parser{
    pub fn new(reader: Box<dyn Reader>, dir: Option<String>) -> Parser{
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
            reader,
            build_dir,
            directory, 
        }
    }

    pub fn parserable(&self) -> bool{
        self.reader.readable()
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

    pub fn parse_line(&mut self) -> Option<String>{
        let line = match self.reader.read_line() {
            Some(line) => line,
            None => return None,
        };
        
        let res = self.parse_directory(&line);
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
        macro_rules! find_target {
            ($s:expr, $($t:expr), *) => {{
                $($s.ends_with($t)) || *
            }};
        }
        let find_cc = |s: &str| -> bool {
            find_target!(s, "gcc", "g++", "clang", "clang++")
        };
        let find_src = |s: &str| -> bool {
            find_target!(s, ".c", ".cc", ".cpp", ".cxx")
        };
        let find_obj = |s: &str| -> bool {
            find_target!(s, ".o", ".obj")
        };
        
        let res_cc = iter.find(|s| find_cc(s) );
        let cc = match res_cc {
            Some(cc) => cc,
            None => return  None
        };

        let files: Vec<&str> = iter.filter(|s| find_src(s) ).collect();
        if files.is_empty(){
            return None;
        }

        let mut args:Vec<Value> = Vec::new();
        while let Some(s) = iter_copy.next() {
            if s.starts_with("-I") {
                let target = match s.eq("-I") {
                    true => s.to_owned() + &Parser::relative_path(
                        &self.absolute_path(iter_copy.next()?),
                        &self.build_dir
                    ), 
                    false => s.split_at(2).0.to_owned() + &Parser::relative_path(
                        &self.absolute_path(s.split_at(2).1), 
                        &self.build_dir
                    ) 
                };
                args.push(Value::String(target));
            }else if s.starts_with("-D") {
                let target = match s.eq("-D") {
                    true => s.to_owned() + iter_copy.next()?,
                    false => s.to_string()
                };
                args.push(Value::String(target)); 
            }else if find_obj(s) {
                let abs_file = self.absolute_path(s);
                args.push(Value::String(Parser::relative_path(
                    &abs_file,
                    &self.build_dir
                )));
            }else if !find_src(s) && !find_cc(s) {
                args.push(Value::String(s.to_string()));
            }
        }

        let mut map = Map::new(); 
       
        // directory: "~/..."
        if self.directory.is_empty(){
            self.directory.clone_from(&self.build_dir);
        }
        map.insert("directory".to_string(), Value::String(self.build_dir.clone()));

        // arguments: "-I... -D..."
        args.insert(0, Value::String(cc.to_string()));

        // file: "*.c" 
        let items: Vec<Map<String, Value>> = files.iter().map(|s|{
            let mut map = map.clone();
            let mut args_copy = args.clone();
            let abs_file = self.absolute_path(s);
            let file_val = Parser::relative_path(
                // s,
                &abs_file,
                &self.build_dir 
            );
            args_copy.push(Value::String(file_val.clone()));
            let value_args = Value::Array(args_copy);
            map.insert("arguments".to_string(), value_args);
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
        
        match abs_src.strip_prefix(&abs_base) {
            None => return ".".to_owned(),
            Some(s) => {
                s.to_owned()
            }
        }
    }
    
}

impl Iterator for Parser{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        // println!("parseable: {}", self.parserable());
        while self.parserable() {
            if let Some(item) = self.parse_line() {
                return Some(item);
            }
        }
        
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_command() {
        let reader = crate::reader::MockReader(); 

        let base_path = "/coder/build";
        let mut parser: Parser = Parser::new(
            Box::new(reader), 
            Some(String::from(base_path))
        );

        let test_cases = [
            (
                "gcc main.c -o main",
                "[\n  {\n    \"arguments\": [\n      \"gcc\"\n    ],\n    \"directory\": \"/coder/build\",\n    \"file\": \"main.c\"\n  }\n]"
            ),
            (
                "g++ main.cpp -o main -I/usr/include -DFLAG",
                "[\n  {\n    \"arguments\": [\n      \"g++\",\n      \"-I/usr/include\",\n      \"-DFLAG\"\n    ],\n    \"directory\": \"/coder/build\",\n    \"file\": \"main.cpp\"\n  }\n]"
            ),
            (
                "clang main.c -I/usr/include -DFLAG",
                "[\n  {\n    \"arguments\": [\n      \"clang\",\n      \"-I/usr/include\",\n      \"-DFLAG\"\n    ],\n    \"directory\": \"/coder/build\",\n    \"file\": \"main.c\"\n  }\n]"
            ),
            (
                "clang++ main.cxx -o main -DFLAG",
                "[\n  {\n    \"arguments\": [\n      \"clang++\",\n      \"-DFLAG\"\n    ],\n    \"directory\": \"/coder/build\",\n    \"file\": \"main.cxx\"\n  }\n]"
            ),
            (
                "gcc main.c -x c -E",
                "[\n  {\n    \"arguments\": [\n      \"gcc\"\n    ],\n    \"directory\": \"/coder/build\",\n    \"file\": \"main.c\"\n  }\n]"
            ),
        ];

        for (i, (input, expected)) in test_cases.iter().enumerate() {
            let result = parser.parser_command(input);
            assert_eq!(result, Some(expected.to_string()), "Test case {} failed", i);
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
    fn test_absolute_path(){
        let reader = crate::reader::MockReader(); 

        let base_path = "/cc//rust";
        let parser: Parser = Parser::new(
            Box::new(reader), 
            Some(String::from(base_path))
        );

        let src_path = "../././tests/code//config.txt";
        let abs_path = parser.absolute_path(src_path);
        assert_eq!(abs_path, "/cc/tests/code/config.txt");
    }

    #[test]
    fn test_relative_path(){
        let src_path = "../././tests/code//config.txt";
        let base_path = "../tests/./../tests";

        assert_eq!(Parser::relative_path(src_path, base_path), "code/config.txt");
        // assert_eq!(Parser::relative_path("/", "/"), ".");
        assert_eq!(Parser::relative_path("/a/b/c", "/a/b/c"), ".");
        assert_eq!(Parser::relative_path("/a/b/c", "/a/b/d"), "../d");
        assert_eq!(Parser::relative_path("/a/b/c", "/a/c/d"), "../../c/d");
        assert_eq!(Parser::relative_path("/a/b/c", "/b/c/d"), "../../../b/c/d");
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

    macro_rules! find_target {
        ($s:expr, $($t:expr), *) => {{
            $($s.ends_with($t)) || *
        }};
    }
    
    #[test]
    fn test_find_target() {
        assert_eq!(find_target!("hello", "world!"), false);
        assert_eq!(find_target!("hello world", "world"), true);
        assert_eq!(find_target!("hello world", "world!"), false);
        assert_eq!(find_target!("hello world", "hello"), false);
        assert_eq!(find_target!("hello", "planet"), false);
        assert_eq!(find_target!("hello", "world", "planet"), false);
        assert_eq!(find_target!("hello", "world", "planet", "universe"), false);
        assert_eq!(find_target!("hello, world", "world", "planet", "universe", "multiverse"), true);   
     }
}
