use crate::csv_parser::CsvData;
use crate::generators::{DataGenerator, SmartGenerator};
use actix_web::{HttpResponse, Responder, web};
use log::{debug, info};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct GenerateRequest {
    pub row_count: Option<usize>,
    pub save: Option<bool>,
    pub headers: Option<Vec<String>>,
}

fn generate_with_generator<G: DataGenerator>(generator: G, row_count: usize) -> CsvData {
    let mut rng = rand::rng();
    let headers = generator.headers();

    debug!("Generating {} rows using functional map", row_count);
    let rows: Vec<Vec<String>> = (1..=row_count)
        .map(|i| generator.generate_row(i, &mut rng))
        .collect();

    CsvData { headers, rows }
}

pub async fn generate_placeholder(req: web::Json<GenerateRequest>) -> impl Responder {
    let row_count = req.row_count.unwrap_or(20);

    info!("Generating {} placeholder rows", row_count);

    if row_count == 0 || row_count > 1000 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "row_count must be between 1 and 1000"
        }));
    }

    let headers = req.headers.clone().unwrap_or_else(|| {
        vec![
            "id".to_string(),
            "name".to_string(),
            "email".to_string(),
            "age".to_string(),
            "city".to_string(),
        ]
    });

    let generator = SmartGenerator::new(headers.clone());
    let csv_data = generate_with_generator(generator, row_count);

    info!("Generated {} rows successfully", row_count);

    HttpResponse::Ok().json(serde_json::json!({
        "data": csv_data,
        "message": format!("Generated {} rows successfully", row_count)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test as actix_test, web};

    #[tokio::test]
    async fn test_generate_default() {
        let app = actix_test::init_service(
            App::new().route("/generate", web::post().to(generate_placeholder)),
        )
        .await;

        let req = actix_test::TestRequest::post()
            .uri("/generate")
            .set_json(&serde_json::json!({}))
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        assert!(body.get("data").is_some());
        assert!(body.get("message").is_some());

        let data = &body["data"];
        assert_eq!(data["headers"].as_array().unwrap().len(), 5);
        assert_eq!(data["rows"].as_array().unwrap().len(), 20);

        let headers = data["headers"].as_array().unwrap();
        assert_eq!(headers[0].as_str().unwrap(), "id");
        assert_eq!(headers[1].as_str().unwrap(), "name");
        assert_eq!(headers[2].as_str().unwrap(), "email");
        assert_eq!(headers[3].as_str().unwrap(), "age");
        assert_eq!(headers[4].as_str().unwrap(), "city");
    }

    #[tokio::test]
    async fn test_generate_custom_row_count() {
        let app = actix_test::init_service(
            App::new().route("/generate", web::post().to(generate_placeholder)),
        )
        .await;

        let req = actix_test::TestRequest::post()
            .uri("/generate")
            .set_json(&serde_json::json!({
                "row_count": 50
            }))
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];
        assert_eq!(data["rows"].as_array().unwrap().len(), 50);
        assert_eq!(
            body["message"].as_str().unwrap(),
            "Generated 50 rows successfully"
        );
    }

    #[tokio::test]
    async fn test_generate_custom_headers() {
        let app = actix_test::init_service(
            App::new().route("/generate", web::post().to(generate_placeholder)),
        )
        .await;

        let custom_headers = vec!["user_id", "username", "phone"];
        let req = actix_test::TestRequest::post()
            .uri("/generate")
            .set_json(&serde_json::json!({
                "headers": custom_headers,
                "row_count": 10
            }))
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];

        let headers = data["headers"].as_array().unwrap();
        assert_eq!(headers.len(), 3);
        assert_eq!(headers[0].as_str().unwrap(), "user_id");
        assert_eq!(headers[1].as_str().unwrap(), "username");
        assert_eq!(headers[2].as_str().unwrap(), "phone");

        assert_eq!(data["rows"].as_array().unwrap().len(), 10);
    }

    #[tokio::test]
    async fn test_generate_single_row() {
        let app = actix_test::init_service(
            App::new().route("/generate", web::post().to(generate_placeholder)),
        )
        .await;

        let req = actix_test::TestRequest::post()
            .uri("/generate")
            .set_json(&serde_json::json!({
                "row_count": 1
            }))
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];
        assert_eq!(data["rows"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_generate_max_rows() {
        let app = actix_test::init_service(
            App::new().route("/generate", web::post().to(generate_placeholder)),
        )
        .await;

        let req = actix_test::TestRequest::post()
            .uri("/generate")
            .set_json(&serde_json::json!({
                "row_count": 1000
            }))
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];
        assert_eq!(data["rows"].as_array().unwrap().len(), 1000);
    }

    #[tokio::test]
    async fn test_generate_zero_rows_invalid() {
        let app = actix_test::init_service(
            App::new().route("/generate", web::post().to(generate_placeholder)),
        )
        .await;

        let req = actix_test::TestRequest::post()
            .uri("/generate")
            .set_json(&serde_json::json!({
                "row_count": 0
            }))
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        assert_eq!(
            body["error"].as_str().unwrap(),
            "row_count must be between 1 and 1000"
        );
    }

    #[tokio::test]
    async fn test_generate_too_many_rows_invalid() {
        let app = actix_test::init_service(
            App::new().route("/generate", web::post().to(generate_placeholder)),
        )
        .await;

        let req = actix_test::TestRequest::post()
            .uri("/generate")
            .set_json(&serde_json::json!({
                "row_count": 1001
            }))
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        assert_eq!(
            body["error"].as_str().unwrap(),
            "row_count must be between 1 and 1000"
        );
    }

    #[tokio::test]
    async fn test_generate_extremely_large_row_count() {
        let app = actix_test::init_service(
            App::new().route("/generate", web::post().to(generate_placeholder)),
        )
        .await;

        let req = actix_test::TestRequest::post()
            .uri("/generate")
            .set_json(&serde_json::json!({
                "row_count": 99999
            }))
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[tokio::test]
    async fn test_generate_single_header() {
        let app = actix_test::init_service(
            App::new().route("/generate", web::post().to(generate_placeholder)),
        )
        .await;

        let req = actix_test::TestRequest::post()
            .uri("/generate")
            .set_json(&serde_json::json!({
                "headers": ["id"],
                "row_count": 5
            }))
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];

        let headers = data["headers"].as_array().unwrap();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].as_str().unwrap(), "id");

        let rows = data["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 5);
        for row in rows {
            assert_eq!(row.as_array().unwrap().len(), 1);
        }
    }

    #[tokio::test]
    async fn test_generate_many_headers() {
        let app = actix_test::init_service(
            App::new().route("/generate", web::post().to(generate_placeholder)),
        )
        .await;

        let headers: Vec<String> = (1..=20).map(|i| format!("col{}", i)).collect();
        let req = actix_test::TestRequest::post()
            .uri("/generate")
            .set_json(&serde_json::json!({
                "headers": headers,
                "row_count": 3
            }))
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];

        assert_eq!(data["headers"].as_array().unwrap().len(), 20);

        let rows = data["rows"].as_array().unwrap();
        for row in rows {
            assert_eq!(row.as_array().unwrap().len(), 20);
        }
    }

    #[test]
    fn test_generate_request_deserialization_full() {
        let json = r#"{
            "row_count": 25,
            "save": true,
            "headers": ["id", "name", "email"]
        }"#;

        let request: GenerateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.row_count, Some(25));
        assert_eq!(request.save, Some(true));
        assert_eq!(
            request.headers,
            Some(vec![
                "id".to_string(),
                "name".to_string(),
                "email".to_string()
            ])
        );
    }

    #[test]
    fn test_generate_request_deserialization_minimal() {
        let json = r#"{}"#;

        let request: GenerateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.row_count, None);
        assert_eq!(request.save, None);
        assert_eq!(request.headers, None);
    }

    #[test]
    fn test_generate_request_deserialization_partial() {
        let json = r#"{"row_count": 100}"#;

        let request: GenerateRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.row_count, Some(100));
        assert_eq!(request.save, None);
        assert_eq!(request.headers, None);
    }

    #[test]
    fn test_generate_with_generator_basic() {
        let headers = vec!["id".to_string(), "name".to_string()];
        let generator = SmartGenerator::new(headers.clone());

        let csv_data = generate_with_generator(generator, 10);

        assert_eq!(csv_data.headers, headers);
        assert_eq!(csv_data.rows.len(), 10);

        for row in &csv_data.rows {
            assert_eq!(row.len(), 2);
        }
    }

    #[test]
    fn test_generate_with_generator_single_row() {
        let headers = vec!["test".to_string()];
        let generator = SmartGenerator::new(headers.clone());

        let csv_data = generate_with_generator(generator, 1);

        assert_eq!(csv_data.headers.len(), 1);
        assert_eq!(csv_data.rows.len(), 1);
        assert_eq!(csv_data.rows[0].len(), 1);
    }

    #[test]
    fn test_generate_with_generator_many_rows() {
        let headers = vec!["col1".to_string(), "col2".to_string(), "col3".to_string()];
        let generator = SmartGenerator::new(headers.clone());

        let csv_data = generate_with_generator(generator, 500);

        assert_eq!(csv_data.headers, headers);
        assert_eq!(csv_data.rows.len(), 500);

        for row in &csv_data.rows {
            assert_eq!(row.len(), 3);
        }
    }

    #[tokio::test]
    async fn test_generate_response_structure() {
        let app = actix_test::init_service(
            App::new().route("/generate", web::post().to(generate_placeholder)),
        )
        .await;

        let req = actix_test::TestRequest::post()
            .uri("/generate")
            .set_json(&serde_json::json!({
                "row_count": 5
            }))
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        let body: serde_json::Value = actix_test::read_body_json(resp).await;

        assert!(body.is_object());
        assert!(body.get("data").is_some());
        assert!(body.get("message").is_some());

        let data = &body["data"];
        assert!(data.get("headers").is_some());
        assert!(data.get("rows").is_some());
        assert!(data["headers"].is_array());
        assert!(data["rows"].is_array());
    }

    #[tokio::test]
    async fn test_generate_empty_headers_array() {
        let app = actix_test::init_service(
            App::new().route("/generate", web::post().to(generate_placeholder)),
        )
        .await;

        let req = actix_test::TestRequest::post()
            .uri("/generate")
            .set_json(&serde_json::json!({
                "headers": [],
                "row_count": 5
            }))
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];

        assert_eq!(data["headers"].as_array().unwrap().len(), 0);
        let rows = data["rows"].as_array().unwrap();
        for row in rows {
            assert_eq!(row.as_array().unwrap().len(), 0);
        }
    }

    #[tokio::test]
    async fn test_generate_special_characters_in_headers() {
        let app = actix_test::init_service(
            App::new().route("/generate", web::post().to(generate_placeholder)),
        )
        .await;

        let special_headers = vec!["user-id", "first_name", "email@domain", "age (years)"];
        let req = actix_test::TestRequest::post()
            .uri("/generate")
            .set_json(&serde_json::json!({
                "headers": special_headers,
                "row_count": 3
            }))
            .to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let body: serde_json::Value = actix_test::read_body_json(resp).await;
        let data = &body["data"];

        let headers = data["headers"].as_array().unwrap();
        assert_eq!(headers[0].as_str().unwrap(), "user-id");
        assert_eq!(headers[1].as_str().unwrap(), "first_name");
        assert_eq!(headers[2].as_str().unwrap(), "email@domain");
        assert_eq!(headers[3].as_str().unwrap(), "age (years)");
    }
}
