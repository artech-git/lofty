use std::{sync::Arc, time::Duration};

// use errors::BackendErrors;
use dashmap::DashMap;
use errors::FragmentError;
use handlers::JobHandle; 

// extern crate scopeguard;


mod file; 
mod errors;
mod utils; 
mod handlers;
mod authorization;
mod config;

async fn tokio_main() -> Result<(), FragmentError> { 

    let addr = [0_u8; 4].into(); 
    let port = 2053; 

    let handle: JobHandle = Arc::new(DashMap::new());

    let router = utils::create_router(handle).await?;
    let _app = utils::start_server((addr, port), router).await?;
    
    Ok(())
}

fn main() {
    let mut runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .global_queue_interval(40)
        .build()
        .unwrap();

    let tokio_main_process = tokio_main(); 
    
    runtime.block_on(tokio_main_process);
}