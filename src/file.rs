use std::{path::PathBuf, usize};
use serde::{Serialize, Deserialize}; 
use tokio::sync::oneshot;
use uuid::Uuid;

// use crate::errors::BackendErrors; 


#[derive(Debug, Serialize, Deserialize )]
pub struct FileObject { 
    path: PathBuf, 
    state: UploadState, 
    file_size: usize, 
    name: String, 
    uuid: Uuid, 
    // hash: [u8; 256],
    // sender: oneshot::Sender<usize> 
}

impl FileObject { 

    pub fn new(path: impl Into<PathBuf>, size: usize, ) -> Self { 
        Self { 
            path: path.into(), 
            state: UploadState::UnInit, 
            file_size: size, 
            name: "".to_string(),
            uuid: Uuid::new_v4(), 
            // hash: [0_u8; 256]
        }
    }


    pub fn update_state(&mut self, state: UploadState) {
        self.state = state; 
    }


}



#[derive(Debug, Serialize, Deserialize)]
pub enum UploadState {
    UnInit, 
    Failed, 
    Broken(usize),
    Resume(usize), 
    Complete, 
}


#[derive(Debug)]
pub struct SharedFileState { 
    state: oneshot::Receiver<usize>,

}