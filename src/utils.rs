use std::{net::Ipv4Addr, path::Path};

use axum::routing::{get, post};
use axum::{Extension, Router};
use tokio::fs::File;
use tokio::io::BufWriter;  

use crate::FragmentError;
use crate::config::LoadConfig;
use crate::handlers::{JobHandle, init_upload_process, task_progress, resume_upload,};


pub async fn create_router( 
    ext: JobHandle,
) -> Result<Router, FragmentError> { 
    
    let serve_dir = tower_http::services::fs::ServeDir::new("./admin");

    // let server_router = Router::new()
    //     .route("/foo", get(|| async { "Hi from /foo" }))
    //     .nest_service("/assets", serve_dir.clone())
    //     .fallback_service(serve_dir);

    let mut router = Router::new()
        .route("/upload_file", get(init_upload_process))
        .route("/status", get(task_progress))
        .route("/resume_upload", get(resume_upload))
        .layer(Extension(ext));  

    let final_router = router.nest_service("/dashboard", serve_dir); 

    Ok(final_router)
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

// pub async fn load_config(path: impl AsRef<str>) -> Result<LoadConfig, FragmentError>  {
//     Ok()
// }
