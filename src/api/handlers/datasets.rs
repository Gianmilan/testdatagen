use crate::csv_parser::CsvData;
use crate::db::models::{GenerateFromDatasetRequest, SaveDatasetRequest};
use crate::db::operations;
use crate::generators::{DataGenerator, SmartGenerator};
use actix_web::{HttpResponse, Responder, web};
use log::{error, info};
use sqlx::SqlitePool;

pub async fn list(pool: web::Data<SqlitePool>) -> impl Responder {
    info!("Listing all datasets");

    match operations::list_datasets(pool.get_ref()).await {
        Ok(datasets) => HttpResponse::Ok().json(datasets),
        Err(e) => {
            error!("Failed to list datasets: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to list datasets: {}", e)
            }))
        }
    }
}

pub async fn save(
    pool: web::Data<SqlitePool>,
    req: web::Json<SaveDatasetRequest>,
) -> impl Responder {
    info!("Saving dataset: {}", req.name);

    match operations::save_dataset(
        pool.get_ref(),
        &req.name,
        &req.headers,
        &req.data_type,
        req.column_types.as_ref(),
        req.sample_data.as_deref(),
    )
    .await
    {
        Ok(id) => {
            info!("Dataset saved with id: {}", id);
            HttpResponse::Ok().json(serde_json::json!({
                "id": id,
                "message": "Dataset saved successfully"
            }))
        }
        Err(e) => {
            error!("Failed to save dataset: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to save dataset: {}", e)
            }))
        }
    }
}

pub async fn get_one(pool: web::Data<SqlitePool>, path: web::Path<i64>) -> impl Responder {
    let id = path.into_inner();
    info!("Getting dataset with id: {}", id);

    match operations::get_datasets(pool.get_ref(), id).await {
        Ok(Some((dataset, csv_data))) => HttpResponse::Ok().json(serde_json::json!({
            "dataset": dataset,
            "data": csv_data
        })),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Dataset with id {} not found", id)
        })),
        Err(e) => {
            error!("Failed to get dataset: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to get dataset: {}", e)
            }))
        }
    }
}

pub async fn delete(pool: web::Data<SqlitePool>, path: web::Path<i64>) -> impl Responder {
    let id = path.into_inner();
    info!("Deleting dataset with id: {}", id);

    match operations::delete_dataset(pool.get_ref(), id).await {
        Ok(true) => {
            info!("Dataset {} deleted successfully", id);
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Dataset deleted successfully"
            }))
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Dataset with id {} not found", id)
        })),
        Err(e) => {
            error!("Failed to delete dataset: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to delete dataset: {}", e)
            }))
        }
    }
}

pub async fn generate_from_dataset(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
    req: web::Json<GenerateFromDatasetRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let row_count = req.row_count.unwrap_or(20);

    info!("Generating {} rows from dataset with id: {}", row_count, id);

    if row_count == 0 || row_count > 1000 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "row_count must be between 1 and 1000"
        }));
    }

    // Fetch dataset
    let dataset_result = operations::get_datasets(pool.get_ref(), id).await;

    let (dataset, _) = match dataset_result {
        Ok(Some(data)) => data,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Dataset with id {} not found", id)
            }));
        }
        Err(e) => {
            error!("Failed to fetch dataset: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to fetch dataset: {}", e)
            }));
        }
    };

    let headers: Vec<String> = match serde_json::from_str(&dataset.headers) {
        Ok(h) => h,
        Err(e) => {
            error!("Failed to parse headers: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to parse dataset headers"
            }));
        }
    };

    let generator = SmartGenerator::new(headers.clone());

    let mut rng = rand::rng();
    let rows: Vec<Vec<String>> = (1..=row_count)
        .map(|i| generator.generate_row(i, &mut rng))
        .collect();

    let csv_data = CsvData { headers, rows };

    info!(
        "Generated {} rows from dataset '{}'",
        row_count, dataset.name
    );

    HttpResponse::Ok().json(serde_json::json!({
        "data": csv_data,
        "message": format!("Generated {} rows from dataset '{}'", row_count, dataset.name)
    }))
}

pub async fn update(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
    req: web::Json<SaveDatasetRequest>,
) -> impl Responder {
    let id = path.into_inner();
    info!("Updating dataset with id: {}", id);

    match operations::update_dataset(
        pool.get_ref(),
        id,
        &req.name,
        &req.headers,
        &req.data_type,
        req.column_types.as_ref(),
        req.sample_data.as_deref(),
    )
    .await
    {
        Ok(true) => {
            info!("Dataset {} updated successfully", id);
            HttpResponse::Ok().json(serde_json::json!({
                "id": id,
                "message": "Dataset updated successfully"
            }))
        }
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Dataset with id {} not found", id)
        })),
        Err(e) => {
            error!("Failed to update dataset: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to update dataset: {}", e)
            }))
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DuplicateDatasetRequest {
    pub name: Option<String>,
}

pub async fn duplicate(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
    req: web::Json<DuplicateDatasetRequest>,
) -> impl Responder {
    let id = path.into_inner();
    info!("Duplicating dataset with id: {}", id);

    let new_name = req.name.as_deref();

    match operations::duplicate_dataset(pool.get_ref(), id, new_name).await {
        Ok(Some(new_id)) => {
            info!("Dataset {} duplicated successfully as {}", id, new_id);
            HttpResponse::Ok().json(serde_json::json!({
                "id": new_id,
                "message": "Dataset duplicated successfully"
            }))
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Dataset with id {} not found", id)
        })),
        Err(e) => {
            error!("Failed to duplicate dataset: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to duplicate dataset: {}", e)
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_utils;
    use actix_web::dev::{Service, ServiceResponse};
    use actix_web::{App, Error, test};
    use std::collections::HashMap;

    async fn init_test_service_with_routes(
        pool: SqlitePool,
        route_config: impl FnOnce(&mut web::ServiceConfig),
    ) -> impl Service<actix_http::Request, Response = ServiceResponse, Error = Error> {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .configure(route_config),
        )
        .await;

        app
    }

    #[actix_web::test]
    async fn test_list_empty() {
        let pool = test_utils::setup_test_db().await;
        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route("/datasets", web::get().to(list));
        })
        .await;

        let req = test::TestRequest::get().uri("/datasets").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body.is_array());
        assert_eq!(body.as_array().unwrap().len(), 0);
    }

    #[actix_web::test]
    async fn test_list_with_datasets() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["col1".to_string()];

        operations::save_dataset(&pool, "dataset1", &headers, "custom", None, None)
            .await
            .unwrap();
        operations::save_dataset(&pool, "dataset2", &headers, "uploaded", None, None)
            .await
            .unwrap();

        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route("/datasets", web::get().to(list));
        })
        .await;

        let req = test::TestRequest::get().uri("/datasets").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body.as_array().unwrap().len(), 2);
    }

    #[actix_web::test]
    async fn test_save_basic() {
        let pool = test_utils::setup_test_db().await;
        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route("/datasets", web::post().to(save));
        })
        .await;

        let payload = SaveDatasetRequest {
            name: "test_dataset".to_string(),
            headers: vec!["id".to_string(), "name".to_string()],
            data_type: "custom".to_string(),
            column_types: None,
            sample_data: None,
        };

        let req = test::TestRequest::post()
            .uri("/datasets")
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["id"].as_i64().unwrap() > 0);
        assert!(body["message"].as_str().unwrap().contains("successfully"));
    }

    #[actix_web::test]
    async fn test_save_with_sample_data() {
        let pool = test_utils::setup_test_db().await;
        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route("/datasets", web::post().to(save));
        })
        .await;

        let payload = SaveDatasetRequest {
            name: "test_dataset".to_string(),
            headers: vec!["id".to_string(), "name".to_string()],
            data_type: "uploaded".to_string(),
            column_types: None,
            sample_data: Some(vec![
                vec!["1".to_string(), "Alice".to_string()],
                vec!["2".to_string(), "Bob".to_string()],
            ]),
        };

        let req = test::TestRequest::post()
            .uri("/datasets")
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["id"].as_i64().unwrap() > 0);
    }

    #[actix_web::test]
    async fn test_save_with_column_types() {
        let pool = test_utils::setup_test_db().await;
        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route("/datasets", web::post().to(save));
        })
        .await;

        let mut column_types = HashMap::new();
        column_types.insert("id".to_string(), "number".to_string());
        column_types.insert("email".to_string(), "email".to_string());

        let payload = SaveDatasetRequest {
            name: "test_dataset".to_string(),
            headers: vec!["id".to_string(), "email".to_string()],
            data_type: "custom".to_string(),
            column_types: Some(column_types),
            sample_data: None,
        };

        let req = test::TestRequest::post()
            .uri("/datasets")
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_get_one_success() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["col1".to_string(), "col2".to_string()];
        let sample_data = vec![vec!["a".to_string(), "b".to_string()]];

        let dataset_id =
            operations::save_dataset(&pool, "test", &headers, "custom", None, Some(&sample_data))
                .await
                .unwrap();

        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route("/datasets/{id}", web::get().to(get_one));
        })
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/datasets/{}", dataset_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["dataset"]["name"].as_str().unwrap(), "test");
        assert_eq!(body["data"]["headers"].as_array().unwrap().len(), 2);
        assert_eq!(body["data"]["rows"].as_array().unwrap().len(), 1);
    }

    #[actix_web::test]
    async fn test_get_one_not_found() {
        let pool = test_utils::setup_test_db().await;
        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route("/datasets/{id}", web::get().to(get_one));
        })
        .await;

        let req = test::TestRequest::get().uri("/datasets/999").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["error"].as_str().unwrap().contains("not found"));
    }

    #[actix_web::test]
    async fn test_delete_success() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["col1".to_string()];

        let dataset_id =
            operations::save_dataset(&pool, "to_delete", &headers, "custom", None, None)
                .await
                .unwrap();

        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route("/datasets/{id}", web::delete().to(delete));
        })
        .await;

        let req = test::TestRequest::delete()
            .uri(&format!("/datasets/{}", dataset_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["message"].as_str().unwrap().contains("successfully"));
    }

    #[actix_web::test]
    async fn test_delete_not_found() {
        let pool = test_utils::setup_test_db().await;
        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route("/datasets/{id}", web::delete().to(delete));
        })
        .await;

        let req = test::TestRequest::delete()
            .uri("/datasets/999")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn test_generate_from_dataset_success() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["id".to_string(), "name".to_string(), "email".to_string()];

        let dataset_id = operations::save_dataset(&pool, "test", &headers, "custom", None, None)
            .await
            .unwrap();

        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route(
                "/datasets/{id}/generate",
                web::post().to(generate_from_dataset),
            );
        })
        .await;

        let payload = GenerateFromDatasetRequest {
            row_count: Some(10),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/datasets/{}/generate", dataset_id))
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["data"]["rows"].as_array().unwrap().len(), 10);
        assert!(body["message"].as_str().unwrap().contains("10 rows"));
    }

    #[actix_web::test]
    async fn test_generate_from_dataset_default_row_count() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["id".to_string()];

        let dataset_id = operations::save_dataset(&pool, "test", &headers, "custom", None, None)
            .await
            .unwrap();

        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route(
                "/datasets/{id}/generate",
                web::post().to(generate_from_dataset),
            );
        })
        .await;

        let payload = GenerateFromDatasetRequest { row_count: None };

        let req = test::TestRequest::post()
            .uri(&format!("/datasets/{}/generate", dataset_id))
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["data"]["rows"].as_array().unwrap().len(), 20);
    }

    #[actix_web::test]
    async fn test_generate_from_dataset_invalid_row_count_zero() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["id".to_string()];

        let dataset_id = operations::save_dataset(&pool, "test", &headers, "custom", None, None)
            .await
            .unwrap();

        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route(
                "/datasets/{id}/generate",
                web::post().to(generate_from_dataset),
            );
        })
        .await;

        let payload = GenerateFromDatasetRequest { row_count: Some(0) };

        let req = test::TestRequest::post()
            .uri(&format!("/datasets/{}/generate", dataset_id))
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(
            body["error"]
                .as_str()
                .unwrap()
                .contains("between 1 and 1000")
        );
    }

    #[actix_web::test]
    async fn test_generate_from_dataset_invalid_row_count_too_large() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["id".to_string()];

        let dataset_id = operations::save_dataset(&pool, "test", &headers, "custom", None, None)
            .await
            .unwrap();

        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route(
                "/datasets/{id}/generate",
                web::post().to(generate_from_dataset),
            );
        })
        .await;

        let payload = GenerateFromDatasetRequest {
            row_count: Some(1001),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/datasets/{}/generate", dataset_id))
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_generate_from_dataset_not_found() {
        let pool = test_utils::setup_test_db().await;
        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route(
                "/datasets/{id}/generate",
                web::post().to(generate_from_dataset),
            );
        })
        .await;

        let payload = GenerateFromDatasetRequest {
            row_count: Some(10),
        };

        let req = test::TestRequest::post()
            .uri("/datasets/999/generate")
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn test_update_success() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["col1".to_string()];

        let dataset_id =
            operations::save_dataset(&pool, "original", &headers, "custom", None, None)
                .await
                .unwrap();

        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route("/datasets/{id}", web::put().to(update));
        })
        .await;

        let payload = SaveDatasetRequest {
            name: "updated".to_string(),
            headers: vec!["col1".to_string(), "col2".to_string()],
            data_type: "uploaded".to_string(),
            column_types: None,
            sample_data: None,
        };

        let req = test::TestRequest::put()
            .uri(&format!("/datasets/{}", dataset_id))
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["message"].as_str().unwrap().contains("successfully"));
    }

    #[actix_web::test]
    async fn test_update_not_found() {
        let pool = test_utils::setup_test_db().await;
        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route("/datasets/{id}", web::put().to(update));
        })
        .await;

        let payload = SaveDatasetRequest {
            name: "updated".to_string(),
            headers: vec!["col1".to_string()],
            data_type: "custom".to_string(),
            column_types: None,
            sample_data: None,
        };

        let req = test::TestRequest::put()
            .uri("/datasets/999")
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn test_duplicate_success() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["id".to_string(), "name".to_string()];

        let original_id =
            operations::save_dataset(&pool, "original", &headers, "custom", None, None)
                .await
                .unwrap();

        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route("/datasets/{id}/duplicate", web::post().to(duplicate));
        })
        .await;

        let payload = DuplicateDatasetRequest { name: None };

        let req = test::TestRequest::post()
            .uri(&format!("/datasets/{}/duplicate", original_id))
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        let duplicate_id = body["id"].as_i64().unwrap();
        assert_ne!(duplicate_id, original_id);
        assert!(body["message"].as_str().unwrap().contains("successfully"));
    }

    #[actix_web::test]
    async fn test_duplicate_with_custom_name() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["col1".to_string()];

        let original_id =
            operations::save_dataset(&pool, "original", &headers, "custom", None, None)
                .await
                .unwrap();

        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route("/datasets/{id}/duplicate", web::post().to(duplicate));
        })
        .await;

        let payload = DuplicateDatasetRequest {
            name: Some("custom_copy".to_string()),
        };

        let req = test::TestRequest::post()
            .uri(&format!("/datasets/{}/duplicate", original_id))
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_duplicate_not_found() {
        let pool = test_utils::setup_test_db().await;
        let app = init_test_service_with_routes(pool, |cfg| {
            cfg.route("/datasets/{id}/duplicate", web::post().to(duplicate));
        })
        .await;

        let payload = DuplicateDatasetRequest { name: None };

        let req = test::TestRequest::post()
            .uri("/datasets/999/duplicate")
            .set_json(&payload)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn test_duplicate_dataset_request_deserialization() {
        let json_with_name = r#"{"name": "my_copy"}"#;
        let req: DuplicateDatasetRequest = serde_json::from_str(json_with_name).unwrap();
        assert_eq!(req.name, Some("my_copy".to_string()));

        let json_without_name = r#"{}"#;
        let req: DuplicateDatasetRequest = serde_json::from_str(json_without_name).unwrap();
        assert_eq!(req.name, None);
    }
}
