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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv_parser::CsvData;
    use actix_web::{App, test as actix_test, web};

    // Helper function to create multipart request with CSV data
    fn create_multipart_request(csv_content: &str) -> actix_test::TestRequest {
        let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
        let body = format!(
            "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.csv\"\r\nContent-Type: text/csv\r\n\r\n{}\r\n--{}--\r\n",
            boundary, csv_content, boundary
        );

        actix_test::TestRequest::post()
            .uri("/upload")
            .insert_header((
                "content-type",
                format!("multipart/form-data; boundary={}", boundary),
            ))
            .set_payload(body)
    }

    #[tokio::test]
    async fn test_upload_csv_basic() {
        let app =
            actix_test::init_service(App::new().route("/upload", web::post().to(upload_csv))).await;

        let csv_content = "id,name,email\n1,John,john@test.com\n2,Jane,jane@test.com";
        let req = create_multipart_request(csv_content).to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        assert!(body.get("data").is_some());
        assert!(body.get("message").is_some());

        let data = &body["data"];
        assert_eq!(data["headers"].as_array().unwrap().len(), 3);
        assert_eq!(data["rows"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_upload_csv_headers_only() {
        let app =
            actix_test::init_service(App::new().route("/upload", web::post().to(upload_csv))).await;

        let csv_content = "id,name,email";
        let req = create_multipart_request(csv_content).to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];
        assert_eq!(data["headers"].as_array().unwrap().len(), 3);
        assert_eq!(data["rows"].as_array().unwrap().len(), 0);
        assert_eq!(
            body["message"].as_str().unwrap(),
            "Successfully parsed CSV with 0 rows"
        );
    }

    #[tokio::test]
    async fn test_upload_csv_single_column() {
        let app =
            actix_test::init_service(App::new().route("/upload", web::post().to(upload_csv))).await;

        let csv_content = "id\n1\n2\n3";
        let req = create_multipart_request(csv_content).to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];
        assert_eq!(data["headers"].as_array().unwrap().len(), 1);
        assert_eq!(data["rows"].as_array().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_upload_csv_many_columns() {
        let app =
            actix_test::init_service(App::new().route("/upload", web::post().to(upload_csv))).await;

        let csv_content =
            "col1,col2,col3,col4,col5,col6,col7,col8,col9,col10\n1,2,3,4,5,6,7,8,9,10";
        let req = create_multipart_request(csv_content).to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];
        assert_eq!(data["headers"].as_array().unwrap().len(), 10);
        assert_eq!(data["rows"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_upload_csv_with_special_characters() {
        let app =
            actix_test::init_service(App::new().route("/upload", web::post().to(upload_csv))).await;

        let csv_content = "user-id,first_name,email@domain\n1,John,john@test.com";
        let req = create_multipart_request(csv_content).to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];
        let headers = data["headers"].as_array().unwrap();
        assert_eq!(headers[0].as_str().unwrap(), "user-id");
        assert_eq!(headers[1].as_str().unwrap(), "first_name");
        assert_eq!(headers[2].as_str().unwrap(), "email@domain");
    }

    #[tokio::test]
    async fn test_upload_csv_with_unicode() {
        let app =
            actix_test::init_service(App::new().route("/upload", web::post().to(upload_csv))).await;

        let csv_content = "名前,年齢,都市\n太郎,25,東京";
        let req = create_multipart_request(csv_content).to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];
        assert_eq!(data["headers"].as_array().unwrap().len(), 3);
        assert_eq!(data["rows"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_upload_csv_with_quoted_values() {
        let app =
            actix_test::init_service(App::new().route("/upload", web::post().to(upload_csv))).await;

        let csv_content = "name,description\n\"John Doe\",\"A person, with a comma\"";
        let req = create_multipart_request(csv_content).to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];
        assert_eq!(data["headers"].as_array().unwrap().len(), 2);
        assert_eq!(data["rows"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_upload_csv_empty_payload() {
        let app =
            actix_test::init_service(App::new().route("/upload", web::post().to(upload_csv))).await;

        let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
        let body = format!("--{}--\r\n", boundary);

        let req = actix_test::TestRequest::post()
            .uri("/upload")
            .insert_header((
                "content-type",
                format!("multipart/form-data; boundary={}", boundary),
            ))
            .set_payload(body)
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        assert!(
            body["error"]
                .as_str()
                .unwrap()
                .contains("Error reading multipart field")
        );
    }

    #[tokio::test]
    async fn test_upload_csv_response_structure() {
        let app =
            actix_test::init_service(App::new().route("/upload", web::post().to(upload_csv))).await;

        let csv_content = "id,name\n1,Test";
        let req = create_multipart_request(csv_content).to_request();

        let resp = actix_test::call_service(&app, req).await;
        let body: serde_json::Value = actix_test::read_body_json(resp).await;

        // Verify structure
        assert!(body.is_object());
        assert!(body.get("data").is_some());
        assert!(body.get("message").is_some());

        let data = &body["data"];
        assert!(data.get("headers").is_some());
        assert!(data.get("rows").is_some());
    }

    #[tokio::test]
    async fn test_upload_csv_message_format() {
        let app =
            actix_test::init_service(App::new().route("/upload", web::post().to(upload_csv))).await;

        let csv_content = "id,name\n1,Test\n2,Test2\n3,Test3";
        let req = create_multipart_request(csv_content).to_request();

        let resp = actix_test::call_service(&app, req).await;
        let body: serde_json::Value = actix_test::read_body_json(resp).await;

        assert_eq!(
            body["message"].as_str().unwrap(),
            "Successfully parsed CSV with 3 rows"
        );
    }

    #[tokio::test]
    async fn test_upload_csv_content_type() {
        let app =
            actix_test::init_service(App::new().route("/upload", web::post().to(upload_csv))).await;

        let csv_content = "id,name\n1,Test";
        let req = create_multipart_request(csv_content).to_request();

        let resp = actix_test::call_service(&app, req).await;

        let content_type = resp.headers().get("content-type");
        assert!(content_type.is_some());
        let content_type_str = content_type.unwrap().to_str().unwrap();
        assert!(content_type_str.contains("application/json"));
    }

    #[tokio::test]
    async fn test_upload_csv_large_file() {
        let app =
            actix_test::init_service(App::new().route("/upload", web::post().to(upload_csv))).await;

        // Create CSV with 100 rows
        let mut csv_content = String::from("id,name,email\n");
        for i in 1..=100 {
            csv_content.push_str(&format!("{},User{},user{}@test.com\n", i, i, i));
        }

        let req = create_multipart_request(&csv_content).to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];
        assert_eq!(data["rows"].as_array().unwrap().len(), 100);
        assert_eq!(
            body["message"].as_str().unwrap(),
            "Successfully parsed CSV with 100 rows"
        );
    }

    #[test]
    fn test_success_response_serialization() {
        let csv_data = CsvData {
            headers: vec!["id".to_string(), "name".to_string()],
            rows: vec![vec!["1".to_string(), "Test".to_string()]],
        };

        let response = SuccessResponse {
            data: csv_data,
            message: "Test message".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"message\":\"Test message\""));
        assert!(json.contains("\"data\""));
    }

    #[test]
    fn test_error_response_serialization() {
        let response = ErrorResponse {
            error: "Test error".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert_eq!(json, "{\"error\":\"Test error\"}");
    }

    #[tokio::test]
    async fn test_upload_csv_with_empty_values() {
        let app =
            actix_test::init_service(App::new().route("/upload", web::post().to(upload_csv))).await;

        let csv_content = "id,name,email\n1,,\n2,Jane,";
        let req = create_multipart_request(csv_content).to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];
        assert_eq!(data["rows"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_upload_csv_with_newlines_in_quoted_fields() {
        let app =
            actix_test::init_service(App::new().route("/upload", web::post().to(upload_csv))).await;

        let csv_content = "id,description\n1,\"Line 1\nLine 2\"\n2,\"Single line\"";
        let req = create_multipart_request(csv_content).to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];
        assert_eq!(data["rows"].as_array().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_upload_csv_row_count_zero() {
        let app =
            actix_test::init_service(App::new().route("/upload", web::post().to(upload_csv))).await;

        let csv_content = "id,name,email";
        let req = create_multipart_request(csv_content).to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        assert_eq!(
            body["message"].as_str().unwrap(),
            "Successfully parsed CSV with 0 rows"
        );
    }

    #[tokio::test]
    async fn test_upload_csv_row_count_one() {
        let app =
            actix_test::init_service(App::new().route("/upload", web::post().to(upload_csv))).await;

        let csv_content = "id,name\n1,Test";
        let req = create_multipart_request(csv_content).to_request();

        let resp = actix_test::call_service(&app, req).await;
        let body: serde_json::Value = actix_test::read_body_json(resp).await;

        assert_eq!(
            body["message"].as_str().unwrap(),
            "Successfully parsed CSV with 1 rows"
        );
    }
}
