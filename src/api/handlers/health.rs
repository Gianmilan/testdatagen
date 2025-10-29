use actix_web::{HttpResponse, Responder};
use log::debug;

pub async fn health_check() -> impl Responder {
    debug!("Health check endpoint called");
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "test_data_gen"
    }))
}
