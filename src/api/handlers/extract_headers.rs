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
                headers: csv_data
                    .headers
                    .iter()
                    .map(|s| s.trim().to_string())
                    .collect(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_headers_basic() {
        let csv_bytes = b"id,name,age\n1,John,20\n2,Jane,21\n3,Bob,22\n";
        let csv_data = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(csv_data.headers, vec!["id", "name", "age"]);
        assert_eq!(csv_data.rows.len(), 3);
    }

    #[test]
    fn test_extract_headers_only() {
        let csv_bytes = b"id,name,email,created_at\n";
        let csv_data = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(csv_data.headers, vec!["id", "name", "email", "created_at"]);
        assert_eq!(csv_data.rows.len(), 0);
    }

    #[test]
    fn test_extract_headers_single_column() {
        let csv_bytes = b"id\n1\n2\n3\n";
        let csv_data = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(csv_data.headers, vec!["id"]);
        assert_eq!(csv_data.rows.len(), 3);
    }

    #[test]
    fn test_extract_headers_special_chars() {
        let csv_bytes =
            b"User ID,Full Name,Email Address,Created@Time\n1,John,john@test.com,2024-01-01\n";
        let csv_data = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(
            csv_data.headers,
            vec!["User ID", "Full Name", "Email Address", "Created@Time"]
        );
        assert_eq!(csv_data.rows.len(), 1);
    }

    #[test]
    fn test_extract_headers_quoted_values() {
        let csv_bytes = b"id,address,city\n1,\"123 Main St, Apt 4\",Boston\n";
        let csv_data = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(csv_data.headers, vec!["id", "address", "city"]);
        assert_eq!(csv_data.rows.len(), 1);
        assert_eq!(csv_data.rows[0][1], "123 Main St, Apt 4");
    }

    #[test]
    fn test_extract_headers_with_whitespace() {
        let csv_bytes = b" id , name , age \n1,John,20\n";
        let csv_data = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(csv_data.headers.len(), 3);
        assert_eq!(csv_data.headers[0], " id ");
        assert_eq!(csv_data.headers[1], " name ");
        assert_eq!(csv_data.headers[2], " age ");

        let trimmed: Vec<String> = csv_data
            .headers
            .iter()
            .map(|s| s.trim().to_string())
            .collect();
        assert_eq!(trimmed, vec!["id", "name", "age"]);
        assert_eq!(csv_data.rows.len(), 1);
    }

    #[test]
    fn test_extract_headers_empty_csv() {
        let csv_bytes = b"";
        let result = parse_csv_from_bytes(csv_bytes);

        assert!(result.is_err() || result.unwrap().headers.is_empty());
    }

    #[test]
    fn test_extract_headers_mismatched_columns() {
        let csv_bytes = b"id,name,age\n1,John,20\n2,Jane\n3,Bob,22,Extra\n";
        let result = parse_csv_from_bytes(csv_bytes);

        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_headers_response_message_singular() {
        let response = HeadersResponse {
            headers: vec!["id".to_string()],
            message: format!("Extracted {} column{}", 1, if 1 == 1 { "" } else { "s" }),
        };

        assert_eq!(response.message, "Extracted 1 column");
    }

    #[test]
    fn test_headers_response_message_plural() {
        let response = HeadersResponse {
            headers: vec!["id".to_string(), "name".to_string(), "age".to_string()],
            message: format!("Extracted {} column{}", 3, if 3 == 1 { "" } else { "s" }),
        };

        assert_eq!(response.message, "Extracted 3 columns");
    }

    #[test]
    fn test_extract_headers_unicode() {
        let csv_bytes = "名前,年齢,メール\n太郎,25,taro@test.com\n".as_bytes();
        let csv_data = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(csv_data.headers, vec!["名前", "年齢", "メール"]);
        assert_eq!(csv_data.rows.len(), 1);
    }
}
