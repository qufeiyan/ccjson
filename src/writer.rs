use tokio::{fs::File, io::{AsyncWriteExt, BufWriter}};
use std::path::{Path, PathBuf};

pub struct Writer{
    path: PathBuf,
    buffer: BufWriter<File>,
    items: u32,
    count: u32,
}

impl Writer{
    pub async fn new(target_dir: Option<&str>, items: u32) -> Writer{
        let raw_path = match target_dir {
            Some(t) => {
                let res = Path::new(t);
                if res.is_dir(){
                    res
                }else {
                    Path::new("./")
                }
            },
            None => Path::new("./")
        };
        let path: PathBuf = raw_path.join("compile_commands.json");
        let buffer = BufWriter::new(File::create(&path).await.unwrap());
        Writer{
            path,
            buffer,
            items,
            count: 0,
        }
    }   

    pub async fn write(&mut self, str: &str){
        match str{
            "[\n" | "\n]" => { 
                self.buffer.write_all(str.as_bytes()).await.unwrap();
            }
            _ => {
                if self.count > 1{ 
                    self.buffer.write_all(b",\n").await.expect("Error: failed to write prefix."); 
                }
                self.buffer.write_all(str.as_bytes()).await.expect("Error: failed to write content."); 
            }
        };

        let is_flush = self.items == 1 || self.items == (self.count % self.items) + 1;
        if is_flush{
            self.flush().await;
        }else {
            self.count += 1;
        }
    }

    pub async fn flush(&mut self){
        self.buffer.flush().await.expect("Error: failed to flush.");
    }
    
    pub fn path(&self) -> &Path {
        &self.path
    }
}



#[tokio::test]
async fn test_writer_new(){
    let binding = "./tests".to_owned();
    let writer = Writer::new(Some(&binding), 1).await;

    assert_eq!(writer.path.to_str(), Some("./tests/compile_commands.json"));

    let _ = std::fs::remove_file(Path::new("./tests/compile_commands.json"));

    let writer = Writer::new(Some("./"), 1).await;

    assert_eq!(writer.path.to_str(), Some("./compile_commands.json"));

    let _ = std::fs::remove_file(Path::new("./compile_commands.json"));
}
