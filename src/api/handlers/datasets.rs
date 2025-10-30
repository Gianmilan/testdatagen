use actix_web::{web, HttpResponse, Responder};
use sqlx::SqlitePool;
use crate::db::models::SaveDatasetRequest;
use crate::db::operations;
use log::{error, info};

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
        &req.csv_data,
        &req.data_type,
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
        Ok(Some((dataset, csv_data))) => {
            HttpResponse::Ok().json(serde_json::json!({
                "dataset": dataset,
                "data": csv_data
            }))
        }
        Ok(None) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Dataset with id {} not found", id)
            }))
        }
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
        Ok(false) => {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": format!("Dataset with id {} not found", id)
            }))
        }
        Err(e) => {
            error!("Failed to delete dataset: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to delete dataset: {}", e)
            }))
        }
    }
}
