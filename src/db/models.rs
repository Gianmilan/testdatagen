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

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveDatasetRequest {
    pub name: String,
    pub headers: Vec<String>,
    pub data_type: String,
    pub column_types: Option<HashMap<String, String>>,
    pub sample_data: Option<Vec<Vec<String>>>,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_dataset_request_deserialization() {
        let json = r#"{
            "name": "test_dataset",
            "headers": ["id", "name", "email"],
            "data_type": "uploaded",
            "column_types": {"id": "number", "name": "text", "email": "email"},
            "sample_data": [["1", "John", "john@test.com"]]
        }"#;
        let request: SaveDatasetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "test_dataset");
        assert_eq!(request.headers.len(), 3);
        assert_eq!(request.data_type, "uploaded");
        assert!(request.column_types.is_some());
        assert!(request.sample_data.is_some());
    }

    #[test]
    fn test_save_dataset_request_deserialization_minimal() {
        let json = r#"{
            "name": "minimal_dataset",
            "headers": ["col1", "col2"],
            "data_type": "custom"
        }"#;
        let request: SaveDatasetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.name, "minimal_dataset");
        assert_eq!(request.headers.len(), 2);
        assert_eq!(request.data_type, "custom");
        assert!(request.column_types.is_none());
        assert!(request.sample_data.is_none());
    }

    #[test]
    fn test_generate_from_dataset_request_deserialization() {
        let json = r#"{"row_count": 50}"#;
        let request: GenerateFromDatasetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.row_count, Some(50));
    }

    #[test]
    fn test_generate_from_dataset_request_deserialization_none() {
        let json = r#"{}"#;
        let request: GenerateFromDatasetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.row_count, None);
    }

    #[test]
    fn test_dataset_summary_serialization() {
        let summary = DataSetSummary {
            id: 1,
            name: "test".to_string(),
            created_at: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc),
            row_count: 10,
            column_count: 3,
            data_type: "uploaded".to_string(),
            has_sample_data: true,
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("\"row_count\":10"));
        assert!(json.contains("\"column_count\":3"));
        assert!(json.contains("\"has_sample_data\":true"));
    }

    #[test]
    fn test_dataset_summary_round_trip() {
        let summary = DataSetSummary {
            id: 42,
            name: "test_dataset".to_string(),
            created_at: DateTime::parse_from_rfc3339("2024-01-15T12:30:00Z")
                .unwrap()
                .with_timezone(&chrono::Utc),
            row_count: 100,
            column_count: 5,
            data_type: "generated".to_string(),
            has_sample_data: false,
        };
        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: DataSetSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, 42);
        assert_eq!(deserialized.name, "test_dataset");
        assert_eq!(deserialized.row_count, 100);
        assert_eq!(deserialized.column_count, 5);
        assert_eq!(deserialized.data_type, "generated");
        assert_eq!(deserialized.has_sample_data, false);
    }

    #[test]
    fn test_save_dataset_request_with_column_types() {
        let mut column_types = HashMap::new();
        column_types.insert("id".to_string(), "number".to_string());
        column_types.insert("name".to_string(), "text".to_string());
        let request = SaveDatasetRequest {
            name: "test".to_string(),
            headers: vec!["id".to_string(), "name".to_string()],
            data_type: "custom".to_string(),
            column_types: Some(column_types),
            sample_data: None,
        };
        assert!(request.column_types.is_some());
        assert_eq!(request.column_types.unwrap().len(), 2);
    }

    #[test]
    fn test_save_dataset_request_with_sample_data() {
        let sample_data = vec![
            vec!["1".to_string(), "Alice".to_string()],
            vec!["2".to_string(), "Bob".to_string()],
        ];
        let request = SaveDatasetRequest {
            name: "test".to_string(),
            headers: vec!["id".to_string(), "name".to_string()],
            data_type: "uploaded".to_string(),
            column_types: None,
            sample_data: Some(sample_data),
        };
        assert!(request.sample_data.is_some());
        assert_eq!(request.sample_data.unwrap().len(), 2);
    }

    #[test]
    fn test_dataset_row_debug() {
        let row = DatasetRow {
            id: 1,
            dataset_id: 10,
            row_index: 0,
            row_data: "test,data".to_string(),
        };
        let debug_str = format!("{:?}", row);
        assert!(debug_str.contains("DatasetRow"));
        assert!(debug_str.contains("id: 1"));
        assert!(debug_str.contains("dataset_id: 10"));
    }
}
