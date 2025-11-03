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

#[derive(Debug, serde::Deserialize)]
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
