use std::{net::Ipv4Addr, path::Path};

use axum::routing::{get, post};
use axum::{Extension, Router};
use tokio::fs::File;
use tokio::io::BufWriter;  

use crate::FragmentError;
use crate::handlers::{JobHandle, init_upload_process, task_progress};


pub async fn create_router( 
    ext: JobHandle<FragmentError>
) -> Result<Router, FragmentError> { 
    
    let router = Router::new()
        .route("/upload_file", get(init_upload_process))
        .route("/status", get(task_progress))
        .layer(Extension(ext)); 

    Ok(router)
}

pub async fn start_server(addr: (Ipv4Addr, u16), app: Router) -> Result<(), FragmentError>  {
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?; 
    Ok(())
}

pub async fn generate_file(path: impl AsRef<Path>, size: usize) -> Result<BufWriter<File>, FragmentError> { 
    let file =  File::create(path).await?;
    let buffered_file = BufWriter::with_capacity(size, file); 
    Ok(buffered_file)
}