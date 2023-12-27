use std::{path::PathBuf, usize, borrow::Cow};
use serde::{Serialize, Deserialize}; 
use tokio::sync::oneshot;
use uuid::Uuid;

// use crate::errors::BackendErrors; 


#[derive(Debug, Serialize, Deserialize )]
pub struct FileObject { 
    pub path: PathBuf, 
    state: UploadState, 
    file_size: usize, 
    name: String, 
    uuid: Uuid, 
    // hash: [u8; 256],
}

impl FileObject { 

    pub fn new(path: impl Into<PathBuf>, size: usize, name: impl ToString) -> Self { 
        Self { 
            path: path.into(),  
            state: UploadState::UnInit, 
            file_size: size, 
            name: name.to_string(),
            uuid: Uuid::new_v4(), 
            // hash: [0_u8; 256]
        }
    }

    #[inline(always)]
    pub fn set_state(&mut self, state: UploadState) {
        self.state = state; 
    }

    #[inline(always)]
    pub fn get_uuid(&self) -> Cow<'_, Uuid> { 
        Cow::Borrowed(&(self.uuid))
    }
    
    pub fn output_file_path(&self) -> PathBuf {
        self.path.join(self.name)
    }

}



#[derive(Debug, Serialize, Deserialize)]
pub enum UploadState {
    UnInit, 
    Broken(usize),
    Resume(usize), 
    Complete, 
    Failed, 
}


#[derive(Debug)]
pub struct SharedFileState { 
    state: oneshot::Receiver<usize>,

}