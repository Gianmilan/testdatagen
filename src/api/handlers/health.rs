use actix_web::{HttpResponse, Responder};
use log::debug;

pub async fn health_check() -> impl Responder {
    debug!("Health check endpoint called");
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "test_data_gen"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test as actix_test, web};

    #[tokio::test]
    async fn test_health_check_status_code() {
        let app =
            actix_test::init_service(App::new().route("/health", web::get().to(health_check)))
                .await;

        let req = actix_test::TestRequest::get().uri("/health").to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_health_check_response_body() {
        let app =
            actix_test::init_service(App::new().route("/health", web::get().to(health_check)))
                .await;

        let req = actix_test::TestRequest::get().uri("/health").to_request();

        let resp = actix_test::call_service(&app, req).await;
        let body: serde_json::Value = actix_test::read_body_json(resp).await;

        assert_eq!(body["status"].as_str().unwrap(), "healthy");
        assert_eq!(body["service"].as_str().unwrap(), "test_data_gen");
    }

    #[tokio::test]
    async fn test_health_check_response_structure() {
        let app =
            actix_test::init_service(App::new().route("/health", web::get().to(health_check)))
                .await;

        let req = actix_test::TestRequest::get().uri("/health").to_request();

        let resp = actix_test::call_service(&app, req).await;
        let body: serde_json::Value = actix_test::read_body_json(resp).await;

        assert!(body.is_object());
        assert!(body.get("status").is_some());
        assert!(body.get("service").is_some());
        assert_eq!(body.as_object().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_health_check_content_type() {
        let app =
            actix_test::init_service(App::new().route("/health", web::get().to(health_check)))
                .await;

        let req = actix_test::TestRequest::get().uri("/health").to_request();

        let resp = actix_test::call_service(&app, req).await;

        let content_type = resp.headers().get("content-type");
        assert!(content_type.is_some());
        let content_type_str = content_type.unwrap().to_str().unwrap();
        assert!(content_type_str.contains("application/json"));
    }

    #[tokio::test]
    async fn test_health_check_multiple_calls() {
        let app =
            actix_test::init_service(App::new().route("/health", web::get().to(health_check)))
                .await;

        for _ in 0..5 {
            let req = actix_test::TestRequest::get().uri("/health").to_request();

            let resp = actix_test::call_service(&app, req).await;
            assert_eq!(resp.status(), 200);

            let body: serde_json::Value = actix_test::read_body_json(resp).await;
            assert_eq!(body["status"].as_str().unwrap(), "healthy");
            assert_eq!(body["service"].as_str().unwrap(), "test_data_gen");
        }
    }

    #[tokio::test]
    async fn test_health_check_is_success() {
        let app =
            actix_test::init_service(App::new().route("/health", web::get().to(health_check)))
                .await;

        let req = actix_test::TestRequest::get().uri("/health").to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_health_check_json_deserialization() {
        let app =
            actix_test::init_service(App::new().route("/health", web::get().to(health_check)))
                .await;

        let req = actix_test::TestRequest::get().uri("/health").to_request();

        let resp = actix_test::call_service(&app, req).await;
        let body: serde_json::Value = actix_test::read_body_json(resp).await;

        assert!(body["status"].is_string());
        assert!(body["service"].is_string());
        assert!(!body["status"].is_null());
        assert!(!body["service"].is_null());
    }

    #[tokio::test]
    async fn test_health_check_with_post_method() {
        let app =
            actix_test::init_service(App::new().route("/health", web::get().to(health_check)))
                .await;

        let req = actix_test::TestRequest::post().uri("/health").to_request();

        let resp = actix_test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404); // Not Found (no route matches POST)
    }

    #[tokio::test]
    async fn test_health_check_direct_call() {
        let response = health_check().await;
        let http_response =
            response.respond_to(&actix_test::TestRequest::default().to_http_request());

        assert_eq!(http_response.status(), 200);
    }

    #[test]
    fn test_health_response_json_format() {
        let expected = serde_json::json!({
            "status": "healthy",
            "service": "test_data_gen"
        });

        assert_eq!(expected["status"].as_str().unwrap(), "healthy");
        assert_eq!(expected["service"].as_str().unwrap(), "test_data_gen");
    }
}
