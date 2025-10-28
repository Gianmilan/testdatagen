use actix_multipart::Multipart;
use actix_web::HttpResponse;
use futures_util::stream::StreamExt;

use crate::csv_parser::parse_csv_from_bytes;
use super::{ErrorResponse, SuccessResponse};

pub async fn upload_csv(mut payload: Multipart) -> HttpResponse {
    let mut csv_bytes = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field = match item {
            Ok(field) => field,
            Err(e) => {
                return HttpResponse::BadRequest().json(ErrorResponse {
                    error: format!("Error reading multipart field: {}", e),
                });
            }
        };

        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(data) => data,
                Err(e) => {
                    return HttpResponse::BadRequest().json(ErrorResponse {
                        error: format!("Error reading chunk: {}", e),
                    });
                }
            };
            csv_bytes.extend_from_slice(&data);
        }
    }

    if csv_bytes.is_empty() {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "No file data received".to_string(),
        });
    }

    match parse_csv_from_bytes(&csv_bytes) {
        Ok(csv_data) => {
            let row_count = csv_data.rows.len();
            HttpResponse::Ok().json(SuccessResponse {
                data: csv_data,
                message: format!("Successfully parsed CSV with {} rows", row_count),
            })
        }
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse {
            error: format!("Failed to parse CSV: {}", e),
        }),
    }
}
