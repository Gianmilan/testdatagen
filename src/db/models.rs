use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Dataset {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<chrono::Utc>,
    pub row_count: i64,
    pub column_count: i64,
    pub data_type: String,
    pub headers: String,
    pub column_types: Option<String>,
    pub has_sample_data: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DatasetRow {
    pub id: i64,
    pub dataset_id: i64,
    pub row_index: i64,
    pub row_data: String,
}

#[derive(Debug, Deserialize)]
pub struct SaveDatasetRequest {
    pub name: String,
    pub headers: Vec<String>,
    pub data_type: String,
    pub column_types: Option<HashMap<String, String>>,
    pub sample_data: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Deserialize)]
pub struct GenerateFromDatasetRequest {
    pub row_count: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSetSummary {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<chrono::Utc>,
    pub row_count: i64,
    pub column_count: i64,
    pub data_type: String,
    pub has_sample_data: bool,
}
