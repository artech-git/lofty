use std::borrow::Cow;

use axum::http::{HeaderMap, HeaderValue, header::*};

use crate::errors::{FragmentError, HeaderErrors};


pub async fn ensure_headers(mut headers: &HeaderMap) -> Result<(), FragmentError> { 
    
    let header = { 
        let fields = [
            ACCESS_CONTROL_ALLOW_ORIGIN, 
            CONTENT_LENGTH, 
            ACCEPT_ENCODING];

        for assumed_header in fields.iter() { 
            if let Some(val) = headers.get(assumed_header){
                continue; 
            } 
        }
    };


    Ok(())

}


// return the value of the associated field in the headermap 
pub async fn extract_header_fields(headers: &HeaderMap, header_field: HeaderName) -> Result<HeaderValue, FragmentError> { 

    if let Some(val) = headers.get(&header_field) { 
        return Ok(val.to_owned());
    }
    else {

        let err_str = Cow::Owned(header_field.as_str().to_string());  

        return Err(HeaderErrors::HeaderFieldMissing(err_str).into());
    }
}