use actix_multipart::Multipart;
use actix_web::HttpResponse;
use log::{error, info, warn};
use serde::Serialize;
use std::time::Instant;

use super::ErrorResponse;
use crate::csv_parser::parse_csv_from_bytes;
use crate::multipart::{MultipartError, parse_multipart};

#[derive(Serialize)]
pub struct HeadersResponse {
    pub headers: Vec<String>,
    pub message: String,
}

pub async fn extract_headers(payload: Multipart) -> Result<HttpResponse, MultipartError> {
    let start_time = Instant::now();
    info!("Received CSV header extraction request");

    let csv_bytes = parse_multipart(payload).await?;

    if csv_bytes.is_empty() {
        warn!("Received empty CSV file");
        return Ok(HttpResponse::BadRequest().json(ErrorResponse {
            error: "Uploaded CSV file is empty".to_string(),
        }));
    }

    match parse_csv_from_bytes(&csv_bytes) {
        Ok(csv_data) => {
            let header_count = csv_data.headers.len();
            let elapsed = start_time.elapsed();

            info!(
                "Successfully parsed CSV with {} headers in {:.2}ms",
                header_count,
                elapsed.as_secs_f64() * 1000.0
            );

            Ok(HttpResponse::Ok().json(HeadersResponse {
                headers: csv_data.headers,
                message: format!(
                    "Extracted {} column{}",
                    header_count,
                    if header_count == 1 { "" } else { "s" }
                ),
            }))
        }
        Err(e) => {
            error!("Failed to parse CSV headers: {}", e);
            Ok(HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Failed to parse CSV: {}", e),
            }))
        }
    }
}
