use reader::Reader;
use tokio_stream::StreamExt;
pub mod reader;
pub mod parser;
pub mod writer;


pub async fn run(reader: Box<dyn Reader>, parser: parser::Parser, mut writer: writer::Writer){
    let mut vec_handle = Vec::new(); 

    let mut reader = reader; // Make reader mutable
    let reader_handle = tokio::spawn(async move{
        while let Some(line) = reader.read_line().await {
            if !reader.readable(){
                break;
            }
            reader.notify(line).await;
        } 
    }); 
    vec_handle.push(reader_handle);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(10);
  
    let parser_handle = tokio::spawn(async move{
        let stream = parser.into_stream();
        tokio::pin!(stream);
        while let Some(line) = stream.next().await{
            let _ = tx.send(line).await;
        }
    });
    vec_handle.push(parser_handle);

    let writer_handle = tokio::spawn(async move{
        writer.write("[\n").await;
        loop {
            if let Some(line) = rx.recv().await{
                writer.write(&line[2..line.len() - 2]).await;
            }
            if rx.is_closed() && rx.is_empty() {
                break;
            }
        }
        writer.write("\n]").await;
        writer.flush().await;
    });
    vec_handle.push(writer_handle);
    
    for handle in vec_handle{
        handle.await.unwrap();
    }

}