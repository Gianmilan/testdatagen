use actix_multipart::Multipart;
use actix_web::HttpResponse;
use futures_util::stream::StreamExt;
use log::{debug, error, info, warn};
use std::time::Instant;

use super::{ErrorResponse, SuccessResponse};
use crate::csv_parser::parse_csv_from_bytes;

pub async fn upload_csv(mut payload: Multipart) -> HttpResponse {
    let start_time = Instant::now();
    info!("Received CSV upload request");

    let mut csv_bytes = Vec::new();

    debug!("Reading multipart payload");
    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(field) => field,
            Err(e) => {
                error!("Error reading multipart field: {}", e);
                return HttpResponse::BadRequest().json(ErrorResponse {
                    error: format!("Error reading multipart field: {}", e),
                });
            }
        };

        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(data) => data,
                Err(e) => {
                    error!("Error reading chunk: {}", e);
                    return HttpResponse::BadRequest().json(ErrorResponse {
                        error: format!("Error reading chunk: {}", e),
                    });
                }
            };
            csv_bytes.extend_from_slice(&data);
        }
    }

    if csv_bytes.is_empty() {
        warn!("No file data received in upload request");
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "No file data received".to_string(),
        });
    }

    debug!("Received {} bytes of CSV data", csv_bytes.len());

    match parse_csv_from_bytes(&csv_bytes) {
        Ok(csv_data) => {
            let row_count = csv_data.rows.len();
            let elapsed = start_time.elapsed();
            info!(
                "Successfully parsed CSV with {} rows in {:.2}ms",
                row_count,
                elapsed.as_secs_f64() * 1000.0
            );

            HttpResponse::Ok().json(SuccessResponse {
                data: csv_data,
                message: format!("Successfully parsed CSV with {} rows", row_count),
            })
        }
        Err(e) => {
            error!("Failed to parse CSV: {}", e);
            HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Failed to parse CSV: {}", e),
            })
        }
    }
}
