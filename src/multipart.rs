use crate::api::handlers::ErrorResponse;
use actix_multipart::Multipart;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use futures_util::StreamExt;
use log::{debug, error, warn};

#[derive(Debug)]
pub enum MultipartError {
    Nodata,
    ReadError(String),
}

impl std::fmt::Display for MultipartError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultipartError::Nodata => write!(f, "No file data received"),
            MultipartError::ReadError(msg) => write!(f, "Error reading file: {}", msg),
        }
    }
}

impl ResponseError for MultipartError {
    fn status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: self.to_string(),
        })
    }
}

pub async fn parse_multipart(mut payload: Multipart) -> Result<Vec<u8>, MultipartError> {
    let mut csv_bytes = Vec::new();

    debug!("Reading multipart payload");

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(field) => field,
            Err(e) => {
                error!("Error reading multipart field: {}", e);
                return Err(MultipartError::ReadError(e.to_string()));
            }
        };

        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(data) => data,
                Err(e) => {
                    error!("Error reading chunk: {}", e);
                    return Err(MultipartError::ReadError(e.to_string()));
                }
            };
            csv_bytes.extend_from_slice(&data);
        }
    }

    if csv_bytes.is_empty() {
        warn!("No file data received in upload request");
        return Err(MultipartError::Nodata);
    }

    debug!("Received {} bytes of CSV data", csv_bytes.len());

    Ok(csv_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multipart_error_nodata_display() {
        let error = MultipartError::Nodata;
        assert_eq!(error.to_string(), "No file data received");
    }

    #[test]
    fn test_multipart_error_read_error_display() {
        let error = MultipartError::ReadError("test error".to_string());
        assert_eq!(error.to_string(), "Error reading file: test error");
    }

    #[test]
    fn test_multipart_error_nodata_status_code() {
        let error = MultipartError::Nodata;
        assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_multipart_error_read_error_status_code() {
        let error = MultipartError::ReadError("something went wrong".to_string());
        assert_eq!(error.status_code(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_multipart_error_nodata_response() {
        let error = MultipartError::Nodata;
        let response = error.error_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_multipart_error_read_error_response() {
        let error = MultipartError::ReadError("file corrupted".to_string());
        let response = error.error_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_multipart_error_debug() {
        let error = MultipartError::Nodata;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Nodata"));
    }

    #[test]
    fn test_multipart_error_read_error_debug() {
        let error = MultipartError::ReadError("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ReadError"));
        assert!(debug_str.contains("test"));
    }
}
