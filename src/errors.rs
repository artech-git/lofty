use std::{panic::Location, backtrace::Backtrace};

use axum::{http::{StatusCode, HeaderMap, Response, self, HeaderValue},response::IntoResponse, body::Body};
use thiserror::Error;

use http_error_derive::HttpError;

use std::convert::From; 


// pub trait BackendErrors: std::fmt::Debug + Send {  
//     fn report_to(&self) -> ErrorReport; 
// }

// impl BackendErrors for FragmentError<E> { 
//     fn report_to(&self) -> ErrorReport {
//         self.into_report()
//     }
// }

#[derive(Debug)]
pub struct ErrorReport { 
    reason: ErrorStates, 
    resp_code: StatusCode, 
    headers: HeaderMap
}

impl ErrorReport { 
    pub fn new(state: ErrorStates, header: HeaderMap) -> Self { 
        Self { 
            reason: state, 
            resp_code: StatusCode::BAD_REQUEST, 
            headers: header
        }
    }

    pub fn convert_to_response(self) -> Response<Body> { 
        
        // let mut json_content = axum::Json::from(self.reason.http_message().unwrap());
        let body = Body::from(self.reason.http_message().unwrap()); 

        let mut resp = Response::new(body);
        let mut headers = resp.headers_mut();
        
        headers.extend(self.headers); 
        *resp.status_mut() = self.resp_code; 
        
        resp
    }
}

impl IntoResponse for FragmentError { 
    fn into_response(self) -> Response<Body> { 
        self.into_report().convert_to_response()
    }
}


#[derive(Debug)]
pub struct FragmentError { 
    // location: Location<'a>, 
    // error: E, 
    error_state: ErrorStates, 
    method: String, 
    trace: Backtrace
}


impl<'a, E> std::convert::From<E> for FragmentError
    where E: Into<ErrorStates>
{
    #[track_caller]
    fn from(err: E) -> FragmentError {
        Self { 
            // location: Location::caller().to_owned(), 
            // error: err, 
            error_state: err.into(), 
            method: "".to_string(), //TODO function capture for the location
            trace: std::backtrace::Backtrace::capture()
        }
    }    
}


impl<'a> FragmentError { 
    pub fn into_report(self) -> ErrorReport { 
        
        let statuscode = StatusCode::from_u16(
            self.error_state
                    .http_code()
                    .unwrap_or(404)
                ).unwrap(); 

        let mut headers = vec![
            (http::header::CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap())
        ]; 

        ErrorReport { 
            reason: self.error_state, 
            resp_code: statuscode, 
            headers: HeaderMap::from_iter(headers)
        }
    }
}


#[derive(Debug, thiserror::Error, HttpError)]
pub enum ErrorStates {

    // // #[http(code = 500, message = "server went into undesired mode")]
    // #[error("Internal I/O Error")] 
    // IOError(#[from] std::io::Error), 

    #[http(code = 500, message = "server went into undesired mode")]
    #[error("Join Handle Error")]
    TaskError(#[from] tokio::task::JoinError),

    #[http(code = 500, message = "server went into undesired mode")]
    #[error("Tokio IO Error")]
    BufferError(#[from] tokio::io::Error),

    
    #[http(code = 500, message = "server went into undesired mode")]
    #[error("internal axum Error")]
    InternalError(#[from] axum::Error),

    
    // #[http(code = 500, message = "server went into undesired mode")]
    // #[error("internal socket Error")]
    // SocketError(#[from] ),


    
    // #[http(code = 500, message = "server went into undesired mode")]
    // #[error("internal axum Error")]
    // UndefinedError(Box<dyn std::error::Error>),

    

}
