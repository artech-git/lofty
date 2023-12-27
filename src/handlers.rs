use std::{sync::Arc, path::PathBuf, str::FromStr, borrow::Cow};

use dashmap::DashMap; 
use axum::{http::{Request, HeaderValue, HeaderMap, header::{*}}, body::Body, response::IntoResponse, Extension};
use uuid::Uuid;
use futures::stream::StreamExt;
use tokio::io::AsyncWriteExt;
use crate::{errors::{OptionExt, ErrorStates, HeaderErrors}, authorization::extract_header_fields}; 

use crate::{file::FileObject, utils::generate_file, FragmentError};


// Task handle for keeping track of all the spawned instance on the runtime
pub type JobHandle<E> = Arc< DashMap<Uuid, Result<FileObject, E>>>; 

/*

*/
async fn streamer_uploader(
    mut body: Body, 
    mut file: FileObject
) -> Result<(), FragmentError> {

    let streamer = body.into_data_stream();

    let mut buf_size = 1_000_0000;  //REMOVE: allocated buffer_size


    let file_path = file.

    let mut buf_writer = generate_file( streamer, buf_size).await?; 

    let mut _chunk_counter = 0; 
    let mut _byte_counter = 0; 

    println!("we entered the stream");

    while let Some(chunk) = stream.next().await { 
        let bytes = chunk?;
        
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        _byte_counter += buf_writer.write(&bytes).await?; 
        _chunk_counter += 1;             
    }
    
    drop(stream);
    
    let _ = buf_writer.shutdown().await?;
}


//validating the header data of the 
fn validate_headers(header: HeaderValue) -> Result<(), HeaderErrors<'static>> { 

    if header.is_empty() {
        return Err(HeaderErrors::HeaderFieldMissing(Cow::Borrowed("FileName")));
    }

    let str_header = header.to_str()?; 
    
    if str_header.chars().all(|e| !e.is_ascii() || !e.is_alphanumeric() || e.is_ascii_hexdigit()) {
        return Err(HeaderErrors::InvalidField(Cow::Borrowed(str_header)));
    }

    Ok(())
}




pub async fn init_upload_process(
    Extension(ext): Extension<JobHandle<FragmentError>>,
    req: Request<Body>, 
) -> Result<impl IntoResponse, FragmentError> {

    let (parts, body) = req.into_parts(); 
    let headers = parts.headers; 

    //=========================================================    
    let headers_names = [
        "FileName",
        "Content-Length",
    ];
    
    let extracted_headers = futures::future::join_all(headers_names
        .iter()
        .map(|field_name| {
            let field = HeaderName::from_str(field_name).expect("Invalid header name");
            extract_header_fields(&headers, &field)
        })).await;
    
    //=========================================================
    let (path, file_size, file_name) = { 
        let file_name = extracted_headers[0]?.to_owned(); 
        let file_size = extracted_headers[1]?.to_owned();

        let _ = validate_headers(file_name)?;
        let _ = validate_headers(file_size)?;

        let path = "./data"; // TODO: change path to const. 

        let file_size = file_size.to_str()
            .map_err(|e| HeaderErrors::HeaderUnwrapError(e))?
            .parse::<usize>()
            .unwrap();  

        let file_name = file_name.to_str()
            .map_err(|e| HeaderErrors::HeaderUnwrapError(e))?; 

        (path, file_size, file_name)
    };
        
    let mut file_obj = FileObject::new(path, file_size, file_name); 
    //=========================================================

    let _ = streamer_uploader(body, file_obj).await?;


        
    let _ = ext.entry(uuid).or_insert(handle); //return error if stream is already present
    
    // println!("task exited");

    // Ok(uuid.as_hyphenated().to_string())

    Ok()

}



//Resume the interrupted upload process
pub async fn resume_upload(
    Extension(ext): Extension<JobHandle<FragmentError>>,
    req: Request<Body>
) -> Result<impl IntoResponse, FragmentError> {

}

// Handle for acquring the status of the In_progress, discarded or cancelled upload process 
pub async fn task_progress(
    mut ext: Extension<JobHandle<FragmentError>>,
    mut req: Request<Body>, 
) -> Result<impl IntoResponse, FragmentError> 
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