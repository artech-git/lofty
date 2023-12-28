use std::{sync::Arc, path::PathBuf, str::FromStr, borrow::Cow, panic::catch_unwind};

use dashmap::DashMap; 
use axum::{http::{Request, HeaderValue, HeaderMap, header::{*}, Response}, body::Body, response::IntoResponse, Extension};
use uuid::Uuid;
use futures::stream::StreamExt;
use tokio::io::AsyncWriteExt;
use crate::{errors::{OptionExt, ErrorStates, HeaderErrors}, authorization::extract_header_fields}; 

use crate::{file::{FileObject, UploadState}, utils::generate_file, FragmentError};


// Task handle for keeping track of all the spawned instance on the runtime
pub type JobHandle = Arc<DashMap<Uuid, FileObject>>; 

/*

*/
async fn streamer_writer(
    mut body: Body, 
    mut file: FileObject,
    mut handle: dashmap::mapref::one::RefMut<'_, Uuid, FileObject>,
) -> Result<(), FragmentError> {

    let mut stream = body.into_data_stream();

    let mut buf_size = 1_000_0000;  //TODO: allocated buffer_size

    let file_path = file.output_file_path(); 

    handle.set_state(UploadState::Progress(0));
    let mut buf_writer = generate_file( file_path, buf_size).await?; 
    
    let mut chunk_counter = 0; 
    let mut byte_counter = 0; 
    
    // println!("we entered the stream");
    
    handle.set_state(UploadState::Progress(0));
    while let Some(chunk) = stream.next().await { 
        let bytes = chunk?;
        // tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        byte_counter += buf_writer.write(&bytes).await?; 
        chunk_counter += 1;             

        // let p = (file.file_size / byte_counter) * 100;
        handle.set_state(UploadState::Progress(byte_counter));
    }
    // we acquired more bytes than nessecary
    if byte_counter > file.file_size { 
        handle.set_state(UploadState::Broken(byte_counter));
        return Err(HeaderErrors::FieldMismatch("Content-Length")); 
    }
    // we got less bytes than possible 
    if byte_counter < file.file_size { 
        handle.set_state(UploadState::Broken(byte_counter));
        return Err(HeaderErrors::FieldMismatch("Content-Length"));
    }
    
    drop(stream);
    
    let _ = buf_writer.shutdown().await?;
    handle.set_state(UploadState::Complete);
    Ok(())
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
    Extension(ext): Extension<JobHandle>,
    req: Request<Body>, 
) -> Result<impl IntoResponse, FragmentError> {

    let (parts, body) = req.into_parts(); 
    let headers = parts.headers; 

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

    // let mut ext_file_obj = guard_with_error( file_obj, |e| { });
    //return error if stream is already present
    let mut update_handle = ext
        .entry(*(file_obj.get_uuid()).to_owned())
        .or_insert(file_obj); 

    let _ = streamer_writer(body, file_obj, update_handle).await?;

    let mut response = { 

        let json = json::object!{ 
            uid: update_handle.get_uuid(), 
        };

        let mut resp = Response::builder()
            .status(200)
            .body(json)
            .expect("unable to create response"); 

        resp        
    };

    Ok(response)

}



//Resume the interrupted upload process
pub async fn resume_upload(
    Extension(ext): Extension<JobHandle>,
    req: Request<Body>
) -> Result<impl IntoResponse, FragmentError> {
    todo!()
}

// Handle for acquring the status of the In_progress, discarded or cancelled upload process 
pub async fn task_progress(
    mut ext: Extension<JobHandle>,
    mut req: Request<Body>, 
) -> Result<impl IntoResponse, FragmentError> 
{ 

    let (mut parts, body) = req.into_parts(); 

    let headers = parts.headers; 

    let headers_names = [
        // "FileName",
        "uuid", 
        "Content-Length",
    ];

    let extracted_headers = futures::future::join_all(headers_names
        .iter()
        .map(|field_name| {
            let field = HeaderName::from_str(field_name).expect("Invalid header name");
            extract_header_fields(&headers, &field)
        })).await;

    let uuid = { 
        let uid = extracted_headers[0]?;
        let str_uid = uid.to_str()?;
        uuid::Uuid::from_str(str_uid)?
    };

    let response = if let Some(val) = ext.get(&uuid) { 
        
        let body = json::object! { 
            status: val.get_state()
        };

        let resp = Response::builder()
            .status(200)
            .body(body)?;

        resp

    } else { 

        let body = "Key not found";

        let resp = Response::builder()
            .status(200)
            .body(body)?;

        resp
    };

    Ok(response)

}