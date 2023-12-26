use std::{sync::Arc, path::PathBuf, str::FromStr};

use dashmap::DashMap; 
use axum::{http::{Request, HeaderValue}, body::Body, response::IntoResponse, Extension};
use tokio::task::JoinHandle;
use uuid::{Uuid, uuid};
use futures::stream::StreamExt;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::{file::FileObject, utils::generate_file, FragmentError};


// Task handle for keeping track of all the spawned instance on the runtime
pub type JobHandle<E> = Arc<DashMap<Uuid, JoinHandle<Result<FileObject, E>>>>; 


pub async fn init_upload_process(
    mut ext: Extension<JobHandle<FragmentError>>,
    mut req: Request<Body>, 
) -> Result<impl IntoResponse, FragmentError> {

    let (header, body) = req.into_parts(); 

    let uuid = uuid::Uuid::new_v4();
    let mut path = PathBuf::from("./data/");

    //******************************** */
    let file_name = header.headers.get("FileName").unwrap().to_str().unwrap().to_string(); 
    // let file_size = header.headers
    //     .get("FileSize")
    //     .unwrap()
    //     .as_ref()
    //     .parse::<usize>()
    //     .unwrap(); 
    
    let file_size = 1000000; 

    let mut file_obj = FileObject::new(path.clone(), file_size); 

    println!("task called");

    // let handle = tokio::task::spawn(async move {
        let mut stream = body.into_data_stream(); 
        //=========================================================
        // let mut file_name = PathBuf::from("xxxxx-yyyyy.txt");
        let mut buf_size = 1_000_0000;  //allocated buffer_size
        //=========================================================
        let mut buf_writer = generate_file(path.join(PathBuf::from_str(file_name.as_str()).unwrap()), buf_size).await?; 

        let mut _chunk_counter = 0; 
        let mut _byte_counter = 0; 

        println!("we entered the stream");

        while let Some(chunk) = stream.next().await{ 
            let bytes = chunk?;
            // println!("chunk accepted: {} len: {}", _chunk_counter, bytes.len()); 
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
            _byte_counter += buf_writer.write(&bytes).await?; 
            _chunk_counter += 1;             
        }

        println!("stream closed: bytes rem");

        // drop(stream);
        
        let _ = buf_writer.shutdown().await?;
        
    //     println!("stream closed");
    //     Ok(file_obj)
    // });

    // let _ = ext.entry(uuid).or_insert(handle); //return error if stream is already present
    
    println!("task exited");

    Ok(uuid.as_hyphenated().to_string())

}


pub async fn task_progress(
    mut ext: Extension<JobHandle<FragmentError>>,
    mut req: Request<Body>, 
) -> impl IntoResponse 
    // where E: Into<FragmentError>
{ 

    let (mut headers, body) = req.into_parts(); 

    //******************************************** */
    let def = HeaderValue::from_str("aaaa-bbbb-cccc-dddd").unwrap(); 
    let uuid = headers.headers
        .get("uuid")
        .unwrap_or(&def); 

    let key = uuid::Uuid::from_str(uuid.to_str().unwrap()).unwrap(); 

    if let Some(val) = ext.0.get(&key) {
        println!("found the key {key:?}");
        // Ok(())
        return "found the key";
    }
    println!("key not found");

    "not found"

}