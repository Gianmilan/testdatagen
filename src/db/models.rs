use chrono::DateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Dataset {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<chrono::Utc>,
    pub row_count: i64,
    pub column_count: i64,
    pub data_type: String,
    pub headers: String,
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
    pub csv_data: crate::csv_parser::CsvData,
    pub data_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSetSummary {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<chrono::Utc>,
    pub row_count: i64,
    pub column_count: i64,
    pub data_type: String,
}
