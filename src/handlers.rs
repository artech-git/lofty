use std::{sync::Arc, str::FromStr, borrow::Cow, io::Read, collections::HashMap};

use axum_core::response::IntoResponse;
use bytes::Bytes;
use dashmap::DashMap; 
use axum::{http::{Request, HeaderValue, header::{*}, Response, request}, body::{Body, HttpBody}, Extension};
use serde_json::json;
use uuid::Uuid;
use futures::stream::StreamExt;
use tokio::io::AsyncWriteExt;
use crate::{errors::{OptionExt, HeaderErrors}, authorization::extract_header_fields}; 

use crate::{file::{FileObject, UploadState}, utils::generate_file, FragmentError};

use self::schedule_upload_process::BodyContent;


// Task handle for keeping track of all the spawned instance on the runtime
pub type JobHandle = Arc<DashMap<Uuid, FileObject>>; 

pub async fn schedule_upload_process(
    ext: Extension<JobHandle>,
    req: Request<HashMap<String, String>>, 
) -> Result<Response<axum::body::Body>, FragmentError> {
    /*
        Request: 
            headers: { 

            }
            body: {
                'fileHash': 'xxx-xxx-xxx', 
                'Length': 1445343, //Mb 
            }
    
        Response: 200 
            body: { status: Approved, uuid: xxxx, time_to_schedule: time}
        
        Response: 200
            body: { status: Denied, Reason: "out of disk space"}
     */

    let (parts, body) = req.into_parts(); 
    let headers = parts.headers; 

    let headers_names = [
        // "FileName",
        "Content-Length",
        "Content-Length",
    ];

    let mut extracted_headers = futures::future::join_all(headers_names
        .iter()
        .map(|field_name| {
            let field = HeaderName::from_str(field_name).expect("Invalid header name");
            extract_header_fields(&headers, field)
        })).await;


    // Check Disk space
    // Check server condition
    let allocated_disk_space = schedule_upload_process::get_allocated_disk_space(100_000); 
    let server_condition = schedule_upload_process::get_server_condition(); 

    let (allocated_disk_space, server_condition) = tokio::join!(allocated_disk_space, server_condition); 
    
    let mut resp = Response::builder(); 
    
    if allocated_disk_space.is_none() || server_condition.is_none() {

        let mut status = "Denied"; 

        let mut json_body = serde_json::json!({
            "status" : status, 
            "reason": "can't be given", 
        });

        let json_body = serde_json::to_vec(&json_body).unwrap(); 

        let mut body = Body::from(json_body);

        resp.body(body);

    }
    else { 

        let mut status = "Approved"; // TODO: Modified that

        let mut extracted_body: BodyContent = schedule_upload_process::process_body(body)?;

        // Init file uploader for 
        let mut uid = uuid::Uuid::new_v4(); 
        let mut file_size = extracted_body.1;
        let mut file_name = extracted_body.0; 
        let path = "./data"; // TODO: modify this part
        
        let file_obj = FileObject::new(path, file_size as usize, file_name.to_string(), Some(uid)); 

        // let mut ext_file_obj = guard_with_error( file_obj, |e| { });
        // return error if stream is already present
        let mut update_handle = ext
            .entry(*(file_obj.get_uuid()).to_owned())
            .or_insert(file_obj); 

        let mut json_body = serde_json::json!({
            "status" : status, 
            "uuid": uid.to_string(),
        });
        
        let mut json_body = serde_json::to_vec(&json_body).unwrap(); 
        
        let mut body = Body::from(json_body);
        
        resp.body(body);
        
    } 

    return Ok(resp); 
}

mod schedule_upload_process {
    use std::time::Duration;

    use crate::errors::BodyErrors;

    use super::*; 
    use sysinfo::Disks; 

    pub type BodyContent = (String, u64);

    pub fn process_body(mut contents: HashMap<String, String>) -> Result<BodyContent, BodyErrors<'static>> { 
        let fileHash = contents.remove("filehash"); 
        let length = contents.remove("length"); 
    
        if fileHash.is_none() || length.is_none() {
            return Err(BodyErrors::MissingField(Cow::Borrowed("FilHash or Length"))); 
        }

        let fileHash = fileHash.unwrap();
        let length = length.unwrap(); 

        let cond1 = fileHash.chars().any(|c| !c.is_ascii_alphanumeric()); 
        let cond2 = length.chars().any(|c| !c.is_ascii_digit()); 

        if cond1 || cond2 { 
            return Err(BodyErrors::MissingValueField(Cow::Borrowed("FilHash or Length"))); 
        }

        let parsed_length = match length.parse::<u64>() { 
            Ok(val) => { 
                val
            },
            Err(e) => { 
                return Err(BodyErrors::InvalidValues(Cow::Borrowed("Length"))); 
            }
        };

        Ok((fileHash, parsed_length))
    }


    pub async fn get_allocated_disk_space(allowed_threshold: u64) -> Option<()> { 
        /*
            Obtain the available space on the disk for writing purpose.
         */

         let mut disks = Disks::new_with_refreshed_list();
         for disk in disks { 
            if disk.name() == "M" { //TODO: modify the storage behaviour
                if disk.available_space() > allowed_threshold {
                    return Some(());
                }
                return None;
            }
         }
        
        return None; 
        // return Err()
    }

    pub async fn get_allocated_memory_space() -> Option<()> {
        /*
            check for allocated memory space within the application 
         */

        const CONTENTION_PERCENT: u8 = 95; // 95 percentage 

        let mut memory = sysinfo::MemoryRefreshKind::new().without_swap(); 

        let mut system = System::new();

        // We don't want to update all memories information.
        system.refresh_memory_specifics(MemoryRefreshKind::new().without_swap());

        let usage_eq = ((system.total_memory() - system.free_memory()) / system.total_memory()) as u8; 

        if usage_eq  >= CONTENTION_PERCENT { 
            return Some(());
        }

        return None; 
    }

    pub async fn get_server_condition() -> Option<()> {
        /*
            get the internal condition of server, such as Contention or hyper fragmentation etc.. 
        */

        let mut networks = sysinfo::Networks::new();

        let instant1 = tokio::time::Instant::now(); 

        let mut summed_avg = 0;
        let mut counted_loops = 0;  

        loop { 

            if tokio::time::Instant::now() - instant1 > Duration::from_millis(500) {
                break; 
            }

            for net in networks.iter() { 
                summed_avg += net.1.errors_on_received();    
            }

            counted_loops += 1; 
        }


        todo!()
    }
}

pub async fn init_upload_process(
    ext: Extension<JobHandle>,
    req: Request<Body>, 
) -> Result<Response<axum::body::Body>, FragmentError> {

    let (parts, body) = req.into_parts(); 
    let headers = parts.headers; 

    let headers_names = [
        "FileName",
        "Content-Length",
    ];

    let mut extracted_headers = futures::future::join_all(headers_names
        .iter()
        .map(|field_name| {
            let field = HeaderName::from_str(field_name).expect("Invalid header name");
            extract_header_fields(&headers, field)
        })).await;
    
    //=========================================================
    let (path, file_size, file_name) = { 
        //TODO: FIX THIS unsafe unwrap
        let file_size = extracted_headers.pop().unwrap()?;
        let file_name = extracted_headers.pop().unwrap()?; 

        let _ = init_upload_process::validate_headers(&file_name)?;
        let _ = init_upload_process::validate_headers(&file_size)?;

        let path = "./data"; // TODO: change path to const. 

        // println!("file_size: {:?}", file_size);

        let file_size = file_size.to_str()
            .map_err(|e| HeaderErrors::HeaderUnwrapError(e))?
            .parse::<u64>()
            .unwrap();  

        let file_name = file_name.to_str()
            .map_err(|e| HeaderErrors::HeaderUnwrapError(e))?
            .to_string(); 

        (path, file_size, file_name)
    };
    
    let file_obj = FileObject::new(path, file_size as usize, file_name); 

    // let mut ext_file_obj = guard_with_error( file_obj, |e| { });
    // return error if stream is already present
    let mut update_handle = ext
        .entry(*(file_obj.get_uuid()).to_owned())
        .or_insert(file_obj); 

    let _ = init_upload_process::streamer_writer(body, &mut update_handle).await?;

    let response = { 

        let uid = update_handle.get_uuid();
        
        let json = serde_json::json!({ 
            "uid": uid
        });

        let json = serde_json::to_vec(&json).unwrap(); 
        let json = axum::body::Body::from(json);
        
        let resp = Response::builder()
            .status(200)
            .body(json)?; 

        resp        
    };

    Ok(response)

}

mod init_upload_process { 
    use super::*;
    pub async fn streamer_writer(
        mut body: Body, 
        handle: &mut dashmap::mapref::one::RefMut<'_, Uuid, FileObject>,
    ) -> Result<(), FragmentError> {
    
        let mut stream = body.into_data_stream();
    
        let mut buf_size = 1_000_0000;  //TODO: allocated buffer_size
    
        let file_path = handle.output_file_path(); 
    
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
        if byte_counter > handle.file_size { 
            handle.set_state(UploadState::Broken(byte_counter));
            return Err(HeaderErrors::FieldMismatch(Cow::Borrowed("Content-Length")).into()); 
        }
        // we got less bytes than possible 
        if byte_counter < handle.file_size { 
            handle.set_state(UploadState::Broken(byte_counter));
            return Err(HeaderErrors::FieldMismatch(Cow::Borrowed("Content-Length")).into());
        }
        
        drop(stream);
        
        let _ = buf_writer.shutdown().await?;
        handle.set_state(UploadState::Complete);
        Ok(())
    }
    
    
    //validating the header data of the 
    pub fn validate_headers(header: &HeaderValue) -> Result<(), HeaderErrors<'static>> { 
    
        if header.is_empty() {
            return Err(HeaderErrors::HeaderFieldMissing(Cow::Borrowed("FileName")));
        }
    
        let str_header = header.to_str()?; 
        
        // if str_header.chars().all(|e| !e.is_ascii() ||*/ !e.is_alphanumeric() || e.is_ascii_hexdigit()) {
        //     return Err(HeaderErrors::InvalidField(Cow::Owned(str_header.to_string())));
        // }
    
        Ok(())
    }
    
}

//Resume the interrupted upload process
pub async fn resume_upload(
    Extension(ext): Extension<JobHandle>,
    req: Request<Body>
) -> Result<Response<String>, FragmentError> {

    let (parts, body) = req.into_parts(); 
    let headers = parts.headers; 

    let headers_names = [
        "uuid",
        "Content-Length",
        ""
    ];

    let mut extracted_headers = futures::future::join_all(headers_names
        .iter()
        .map(|field_name| {
            let field = HeaderName::from_str(field_name).expect("Invalid header name");
            extract_header_fields(&headers, field)
        })).await;

    


    Ok(Response::new("String".to_string()))
}

// Handle for acquring the status of the In_progress, discarded or cancelled upload process 
pub async fn task_progress(
    mut ext: Extension<JobHandle>,
    mut req: Request<Body>, 
) -> Result<Response<axum::body::Body>, FragmentError> { 

    let (mut parts, body) = req.into_parts(); 

    let headers = parts.headers; 

    let headers_names = [
        // "FileName",
        "uuid", 
        // "Content-Length",
    ];

    let mut extracted_headers = futures::future::join_all(headers_names
        .iter()
        .map(|field_name| {
            let field = HeaderName::from_str(field_name).expect("Invalid header name");
            extract_header_fields(&headers, field)
        })).await;

    let uuid = { 
        let uid = extracted_headers.pop().unwrap()?;
        let str_uid = uid.to_str()?;
        uuid::Uuid::from_str(str_uid)?
    };

    let response = if let Some(val) = ext.get(&uuid) {
        
        let uid = val.get_state(); 

        let body = serde_json::json!({ 
            "status": uid,
        });

        let body = serde_json::to_vec(&body).unwrap();
        // let body = axum::Json(body);
        let body = axum::body::Body::from(body);
        
        let resp = Response::builder()
            .status(200)
            .body(body)?;
    
        resp
    
    } else { 

        let body = serde_json::json!({
                "status": UploadState::UnInit
            });
        
        let body = serde_json::to_vec(&body).unwrap();
        // let body = axum::Json(body);
        // let ff = body.as;
        let body = axum::body::Body::from(body);

        let resp = Response::builder()
            .status(200)
            .body(body)?;

        resp
    };

    Ok(response)

}


