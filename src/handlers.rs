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
            "reason": "can't be given", //TODO: change the message type presented here
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
    /*
        Request: 
            headers: 
                FileName
                Content_Length
                uuid: "xxxx-xxxx-xxxx-xxxx"
            Body: 
                Chunks, 128Mb
                Chunks, 128Mb
                Chunks, 128Mb
                Chunks, 128Mb
                Chunks, 128Mb

        Response:
            Status: 200 | 404 | 500
            headers: 
                ...
            Body:
                Json { 
                    Status: Complete|Broke|Failed, 
                }
     */
    let (parts, body) = req.into_parts(); 
    let headers = parts.headers; 

    let headers_names = [
        "FileName",
        "Content-Length",
        "uuid", 
    ];

    let mut extracted_headers = futures::future::join_all(headers_names
        .iter()
        .map(|field_name| {
            let field = HeaderName::from_str(field_name).expect("Invalid header name");
            extract_header_fields(&headers, field)
        })).await;
    
    //=========================================================
    let (uuid, file_size, file_name) = { 
        //TODO: FIX THIS unsafe unwrap
        let uuid =      extracted_headers.pop().unwrap()?;
        let file_size = extracted_headers.pop().unwrap()?;
        let file_name = extracted_headers.pop().unwrap()?; 

        let _ = init_upload_process::validate_headers(&file_name)?;
        let _ = init_upload_process::validate_headers(&file_size)?;
        let _ = init_upload_process::validate_headers(&uuid)?;

        // println!("file_size: {:?}", file_size);

        let file_size = file_size.to_str()
            .map_err(|e| HeaderErrors::HeaderUnwrapError(e))?
            .parse::<u64>()
            .unwrap();  

        let file_name = file_name.to_str()
            .map_err(|e| HeaderErrors::HeaderUnwrapError(e))?
            .to_string(); 

        
        let uuid = uuid.to_str()
            .map_err(|e| HeaderErrors::HeaderUnwrapError(e))?; 

        let uuid = uuid::Uuid::from_str(uuid)?;

        (uuid, file_size, file_name)
    };
    
    // let mut ext_file_obj = guard_with_error( file_obj, |e| { });
    // return error if stream is already present
    let mut update_handle_entry = match ext
        .entry(uuid) {
            dashmap::mapref::entry::Entry::Occupied(mut file_obj) => { 
                // Continue the update process
                file_obj
            },  
            dashmap::mapref::entry::Entry::Vacant(val) => {
                // The fileobj is missing from the uuid field
                return Err(HeaderErrors::InvalidField(Cow::Borrowed("uuid")).into());
            }
        };
        
    let update_handle = update_handle_entry.get_mut();        

    let _ = init_upload_process::streamer_writer(body, update_handle).await?;

    let response = { 
        let status = update_handle.get_state(); 
        let json = serde_json::json!({ 
            "status": status 
        });

        let json = serde_json::to_vec(&json).unwrap(); 
        let json = axum::body::Body::from(json);
        
        let resp = Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(json)?; 

        resp        
    };

    Ok(response)

}

mod init_upload_process { 
    use super::*;
    pub async fn streamer_writer(
        mut body: Body, 
        handle: &mut FileObject,
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
//todo: apply the logic for removal of the FileObj from the JobHandle extension if error happens
pub async fn resume_upload(
    Extension(ext): Extension<JobHandle>,
    req: Request<Body>
) -> Result<Response<String>, FragmentError> {
    /*
            Request: 
                headers: 
                    uuid: "xxxx-xxxx-xxxx-xxxx" // uuid to start this upload process from
                    Content-Length: "..."   //give content-length of file
                    Content-Pointer: "..."  //some position in the file
                    
                Body: 
                    remaining_bytes: 128Mb,
                    remaining_bytes: 128Mb,
                    remaining_bytes: 128Mb,
                    remaining_bytes: 128Mb,
     */
    let (parts, body) = req.into_parts(); 
    let headers = parts.headers; 

    let headers_names = [
        "uuid",
        "Content-Length",
        "Content-Pointer"
    ];

    let mut extracted_headers = futures::future::join_all(headers_names
        .iter()
        .map(|field_name| {
            let field = HeaderName::from_str(field_name).expect("Invalid header name");
            extract_header_fields(&headers, field)
        })).await;

    
    let (uuid, content_length, content_pointer) = { 
        
        let uuid = extracted_headers.pop().unwrap()?; 
        let content_length = extracted_headers.pop().unwrap()?; 
        let content_pointer = extracted_headers.pop().unwrap()?; 
        
        let _ = init_upload_process::validate_headers(&uuid)?;
        let _ = init_upload_process::validate_headers(&content_length)?;
        let _ = init_upload_process::validate_headers(&content_pointer)?;

        // println!("file_size: {:?}", file_size);

        let content_pointer = content_pointer.to_str()
            .map_err(|e| HeaderErrors::HeaderUnwrapError(e))?
            .parse::<u64>()
            .map_err(|e| HeaderErrors::InvalidField(Cow::Borrowed("FileSize")))?;  

        let content_length = content_length.to_str()
            .map_err(|e| HeaderErrors::HeaderUnwrapError(e))?
            .parse::<u64>()
            .map_err(|e| HeaderErrors::InvalidField(Cow::Borrowed("FileSize")))?; 

        
        let uuid = uuid.to_str()
            .map_err(|e| HeaderErrors::HeaderUnwrapError(e))?; 

        let uuid = uuid::Uuid::from_str(uuid)?;

        (uuid, content_length, content_pointer)
    };

    let mut update_handle_entry = match ext
        .entry(uuid) {
            dashmap::mapref::entry::Entry::Occupied(mut file_obj) => { 
                // Continue the update process
                file_obj
            },  
            dashmap::mapref::entry::Entry::Vacant(val) => {
                // The fileobj is missing from the uuid field
                return Err(HeaderErrors::InvalidField(Cow::Borrowed("uuid")).into());
            }
    };

    let update_handle = update_handle_entry.get_mut(); 

    //verify the logical validity of the content passed
    resume_upload::validate_header_entries(update_handle, content_length, content_pointer)?;

    //resume writing to file from the poitner onwards




    todo!()
}

mod resume_upload { 
    use std::path::Path;

    use tokio::{io::{BufWriter, AsyncSeekExt}, fs::File};

    use super::*; 

    pub fn validate_header_entries(
        file_obj: &mut FileObject, 
        content_length: u64, 
        content_pointer: u64
    ) -> Result<(), FragmentError> { 
        
        if (content_pointer as usize) >= file_obj.file_size { 
            return Err(HeaderErrors::InvalidField(Cow::Borrowed(("content_pointer"))).into());
        }
        
        if (content_length as usize) <= 0  {
            return Err(HeaderErrors::InvalidField(Cow::Borrowed(("Content-Length"))).into());
        }
        
        if (content_pointer as usize) <= 0  {
            return Err(HeaderErrors::InvalidField(Cow::Borrowed(("Content-Pointer"))).into());
        }

        if (content_pointer as usize) <= 0  {
            return Err(HeaderErrors::InvalidField(Cow::Borrowed(("Content-Pointer"))).into());
        }

        Ok(())
    }

    pub async fn streamer_writer(
        mut body: Body, 
        content_pointer: u64,
        handle: &mut FileObject
    ) -> Result<(), FragmentError> {
        let mut stream = body.into_data_stream(); 

        let previous_file_path = handle.output_file_path(); 

        handle.set_state(UploadState::Resume((content_pointer) as usize));

        let size = 10_000_000; //todo: modify this part to be fetched from 

        let mut buf_writer = open_file(content_pointer, previous_file_path, size)
            .await
            .map_err(|e| {
                handle.set_state(UploadState::Failed); 
                e
            })?; 

        let mut chunk_counter = 0; 
        let mut byte_counter = 0; 
    
        while let Some(chunk) = stream.next().await { 
            
            let bytes = chunk.map_err(|e| { 
                handle.set_state(UploadState::Broken(((content_pointer as usize) + (byte_counter as usize))));
                e
            })?; 

            byte_counter += buf_writer.write(&bytes).await.map_err(|e| { 
                handle.set_state(UploadState::Broken(((content_pointer as usize) + (byte_counter as usize))));
                e
            })?;

            chunk_counter += 1; 
            handle.set_state(UploadState::Progress(byte_counter as usize));
        }

        //TODO: ensure & test the logical validity of the post checks performed

        // we acquired more bytes than nessecary
        if (byte_counter + content_pointer as usize) > handle.file_size { 
            handle.set_state(UploadState::Broken(byte_counter));
            return Err(HeaderErrors::FieldMismatch(Cow::Borrowed("Content-Length")).into()); 
        }
        // we got less bytes than possible 
        if (byte_counter + content_pointer as usize) < handle.file_size { 
            handle.set_state(UploadState::Broken(byte_counter));
            return Err(HeaderErrors::FieldMismatch(Cow::Borrowed("Content-Length")).into());
        }

        drop(stream); 

        let _ = buf_writer.shutdown().await.map_err(|e| {
            handle.set_state(UploadState::Failed);
            e
        })?; 

        handle.set_state(UploadState::Complete);
        
        Ok(())

    }

    async fn open_file(pointer: u64, path: impl AsRef<Path>, size: usize) -> Result<BufWriter<File>, FragmentError> {
        let mut file = File::open(path).await?;
        file.seek(std::io::SeekFrom::Start(pointer)).await; 
        let buffered_file = BufWriter::with_capacity(size, file); 
        
        Ok(buffered_file)
    }

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


