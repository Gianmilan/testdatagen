use crate::csv_parser::CsvData;
use crate::db::models::{DataSetSummary, Dataset, DatasetRow};
use sqlx::SqlitePool;
use sqlx::error::BoxDynError;

pub async fn save_dataset(
    pool: &SqlitePool,
    name: &str,
    headers: &[String],
    data_type: &str,
    column_types: Option<&std::collections::HashMap<String, String>>,
    sample_data: Option<&[Vec<String>]>,
) -> Result<i64, BoxDynError> {
    let mut tx = pool.begin().await?;

    let headers_json = serde_json::to_string(headers)?;
    let column_types_json = match column_types {
        Some(types) => Some(serde_json::to_string(types)?),
        None => None,
    };

    let limited_sample_data =
        sample_data.map(|data| if data.len() > 100 { &data[..100] } else { data });

    let row_count = limited_sample_data.map(|d| d.len()).unwrap_or(0) as i64;
    let column_count = headers.len() as i64;
    let has_sample_data = limited_sample_data.is_some();

    let result = sqlx::query!(
        r#"INSERT INTO datasets (name, row_count, column_count, data_type, headers, column_types, has_sample_data)
    VALUES (?, ?, ?, ?, ?, ?, ?)"#,
        name,
        row_count,
        column_count,
        data_type,
        headers_json,
        column_types_json,
        has_sample_data
    )
    .execute(&mut *tx)
    .await?;

    let dataset_id = result.last_insert_rowid();

    if let Some(rows) = limited_sample_data {
        for (idx, row) in rows.iter().enumerate() {
            let row_json = serde_json::to_string(row)?;
            let row_index = idx as i64;

            sqlx::query!(
                r#"INSERT INTO dataset_rows (dataset_id, row_index, row_data)
                VALUES (?, ?, ?)"#,
                dataset_id,
                row_index,
                row_json
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    Ok(dataset_id)
}

pub async fn get_datasets(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<(Dataset, CsvData)>, BoxDynError> {
    let dataset = sqlx::query_as!(
        Dataset,
        r#"SELECT id as "id!", name as "name!", created_at as "created_at!: _", row_count as "row_count!", column_count as "column_count!", data_type as "data_type!", headers as "headers!", column_types as "column_types?", has_sample_data as "has_sample_data!" FROM datasets WHERE id = ?"#,
        id
    )
    .fetch_optional(pool)
    .await?;

    let dataset = match dataset {
        Some(d) => d,
        None => return Ok(None),
    };

    let rows = sqlx::query_as!(
        DatasetRow,
        r#"SELECT id as "id!", dataset_id as "dataset_id!", row_index as "row_index!", row_data as "row_data!" FROM dataset_rows WHERE dataset_id = ? ORDER BY row_index"#,
        id
    )
    .fetch_all(pool)
    .await?;

    let headers: Vec<String> = serde_json::from_str(&dataset.headers)?;
    let csv_rows: Result<Vec<Vec<String>>, _> = rows
        .iter()
        .map(|r| serde_json::from_str(&r.row_data))
        .collect();

    let csv_data = CsvData {
        headers,
        rows: csv_rows?,
    };

    Ok(Some((dataset, csv_data)))
}

pub async fn list_datasets(pool: &SqlitePool) -> Result<Vec<DataSetSummary>, BoxDynError> {
    let datasets = sqlx::query_as!(
        DataSetSummary,
        r#"SELECT id as "id!", name as "name!", created_at as "created_at!: _", row_count as "row_count!", column_count as "column_count!", data_type as "data_type!", has_sample_data as "has_sample_data!" FROM datasets ORDER BY created_at DESC"#
    )
    .fetch_all(pool)
    .await?;

    Ok(datasets)
}

pub async fn delete_dataset(pool: &SqlitePool, id: i64) -> Result<bool, BoxDynError> {
    let result = sqlx::query!("DELETE FROM datasets WHERE id = ?", id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn update_dataset(
    pool: &SqlitePool,
    id: i64,
    name: &str,
    headers: &[String],
    data_type: &str,
    column_types: Option<&std::collections::HashMap<String, String>>,
    sample_data: Option<&[Vec<String>]>,
) -> Result<bool, BoxDynError> {
    let existing = sqlx::query!("SELECT id FROM datasets WHERE id = ?", id)
        .fetch_optional(pool)
        .await?;

    if existing.is_none() {
        return Ok(false);
    }

    let mut tx = pool.begin().await?;

    let headers_json = serde_json::to_string(headers)?;
    let column_types_json = match column_types {
        Some(types) => Some(serde_json::to_string(types)?),
        None => None,
    };

    let limited_sample_data =
        sample_data.map(|data| if data.len() > 100 { &data[..100] } else { data });

    let row_count = limited_sample_data.map(|d| d.len()).unwrap_or(0) as i64;
    let column_count = headers.len() as i64;
    let has_sample_data = limited_sample_data.is_some();

    sqlx::query!(
        r#"UPDATE datasets
           SET name = ?, row_count = ?, column_count = ?, data_type = ?,
               headers = ?, column_types = ?, has_sample_data = ?
           WHERE id = ?"#,
        name,
        row_count,
        column_count,
        data_type,
        headers_json,
        column_types_json,
        has_sample_data,
        id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!("DELETE FROM dataset_rows WHERE dataset_id = ?", id)
        .execute(&mut *tx)
        .await?;

    if let Some(rows) = limited_sample_data {
        for (idx, row) in rows.iter().enumerate() {
            let row_json = serde_json::to_string(row)?;
            let row_index = idx as i64;

            sqlx::query!(
                r#"INSERT INTO dataset_rows (dataset_id, row_index, row_data)
                VALUES (?, ?, ?)"#,
                id,
                row_index,
                row_json
            )
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    Ok(true)
}

pub async fn duplicate_dataset(
    pool: &SqlitePool,
    id: i64,
    new_name: Option<&str>,
) -> Result<Option<i64>, BoxDynError> {
    let original = get_datasets(pool, id).await?;

    let (dataset, csv_data) = match original {
        Some(data) => data,
        None => return Ok(None),
    };

    let headers: Vec<String> = serde_json::from_str(&dataset.headers)?;
    let column_types: Option<std::collections::HashMap<String, String>> = match dataset.column_types
    {
        Some(ref types_json) => Some(serde_json::from_str(types_json)?),
        None => None,
    };

    let default_name = format!("{} (Copy)", dataset.name);
    let name = new_name.unwrap_or(&default_name);

    let sample_data = if dataset.has_sample_data && !csv_data.rows.is_empty() {
        Some(csv_data.rows.as_slice())
    } else {
        None
    };

    let new_id = save_dataset(
        pool,
        name,
        &headers,
        &dataset.data_type,
        column_types.as_ref(),
        sample_data,
    )
    .await?;

    Ok(Some(new_id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_utils;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_save_dataset_basic() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["id".to_string(), "name".to_string()];

        let dataset_id = save_dataset(&pool, "test_dataset", &headers, "custom", None, None)
            .await
            .unwrap();

        assert!(dataset_id > 0);
    }

    #[tokio::test]
    async fn test_save_dataset_with_sample_data() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["id".to_string(), "name".to_string()];
        let sample_data = vec![
            vec!["1".to_string(), "Alice".to_string()],
            vec!["2".to_string(), "Bob".to_string()],
        ];

        let dataset_id = save_dataset(
            &pool,
            "test_dataset",
            &headers,
            "uploaded",
            None,
            Some(&sample_data),
        )
        .await
        .unwrap();

        assert!(dataset_id > 0);

        let result = get_datasets(&pool, dataset_id).await.unwrap();
        assert!(result.is_some());
        let (dataset, csv_data) = result.unwrap();
        assert_eq!(dataset.row_count, 2);
        assert_eq!(csv_data.rows.len(), 2);
    }

    #[tokio::test]
    async fn test_save_dataset_with_column_types() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["id".to_string(), "email".to_string()];
        let mut column_types = HashMap::new();
        column_types.insert("id".to_string(), "number".to_string());
        column_types.insert("email".to_string(), "email".to_string());

        let dataset_id = save_dataset(
            &pool,
            "test_dataset",
            &headers,
            "custom",
            Some(&column_types),
            None,
        )
        .await
        .unwrap();

        assert!(dataset_id > 0);

        let result = get_datasets(&pool, dataset_id).await.unwrap();
        let (dataset, _) = result.unwrap();
        assert!(dataset.column_types.is_some());
    }

    #[tokio::test]
    async fn test_save_dataset_limits_sample_data_to_100_rows() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["id".to_string()];
        let sample_data: Vec<Vec<String>> = (0..150).map(|i| vec![i.to_string()]).collect();

        let dataset_id = save_dataset(
            &pool,
            "large_dataset",
            &headers,
            "uploaded",
            None,
            Some(&sample_data),
        )
        .await
        .unwrap();

        let result = get_datasets(&pool, dataset_id).await.unwrap();
        let (dataset, csv_data) = result.unwrap();
        assert_eq!(dataset.row_count, 100);
        assert_eq!(csv_data.rows.len(), 100);
    }

    #[tokio::test]
    async fn test_get_datasets_not_found() {
        let pool = test_utils::setup_test_db().await;

        let result = get_datasets(&pool, 999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_datasets_with_rows() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["col1".to_string(), "col2".to_string()];
        let sample_data = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["c".to_string(), "d".to_string()],
        ];

        let dataset_id = save_dataset(&pool, "test", &headers, "custom", None, Some(&sample_data))
            .await
            .unwrap();

        let result = get_datasets(&pool, dataset_id).await.unwrap();
        assert!(result.is_some());
        let (dataset, csv_data) = result.unwrap();
        assert_eq!(dataset.name, "test");
        assert_eq!(csv_data.headers, headers);
        assert_eq!(csv_data.rows.len(), 2);
        assert_eq!(csv_data.rows[0], vec!["a", "b"]);
    }

    #[tokio::test]
    async fn test_list_datasets_empty() {
        let pool = test_utils::setup_test_db().await;

        let datasets = list_datasets(&pool).await.unwrap();
        assert_eq!(datasets.len(), 0);
    }

    #[tokio::test]
    async fn test_list_datasets_multiple() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["col1".to_string()];

        save_dataset(&pool, "dataset1", &headers, "custom", None, None)
            .await
            .unwrap();
        save_dataset(&pool, "dataset2", &headers, "uploaded", None, None)
            .await
            .unwrap();
        save_dataset(&pool, "dataset3", &headers, "generated", None, None)
            .await
            .unwrap();

        let datasets = list_datasets(&pool).await.unwrap();
        assert_eq!(datasets.len(), 3);
        let names: Vec<String> = datasets.iter().map(|d| d.name.clone()).collect();
        assert!(names.contains(&"dataset1".to_string()));
        assert!(names.contains(&"dataset2".to_string()));
        assert!(names.contains(&"dataset3".to_string()));
    }

    #[tokio::test]
    async fn test_delete_dataset_success() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["col1".to_string()];

        let dataset_id = save_dataset(&pool, "to_delete", &headers, "custom", None, None)
            .await
            .unwrap();

        let deleted = delete_dataset(&pool, dataset_id).await.unwrap();
        assert!(deleted);

        let result = get_datasets(&pool, dataset_id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_dataset_not_found() {
        let pool = test_utils::setup_test_db().await;

        let deleted = delete_dataset(&pool, 999).await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_update_dataset_basic() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["col1".to_string()];

        let dataset_id = save_dataset(&pool, "original", &headers, "custom", None, None)
            .await
            .unwrap();

        let new_headers = vec!["col1".to_string(), "col2".to_string()];
        let updated = update_dataset(
            &pool,
            dataset_id,
            "updated",
            &new_headers,
            "uploaded",
            None,
            None,
        )
        .await
        .unwrap();

        assert!(updated);

        let result = get_datasets(&pool, dataset_id).await.unwrap();
        let (dataset, csv_data) = result.unwrap();
        assert_eq!(dataset.name, "updated");
        assert_eq!(dataset.data_type, "uploaded");
        assert_eq!(csv_data.headers.len(), 2);
    }

    #[tokio::test]
    async fn test_update_dataset_not_found() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["col1".to_string()];

        let updated = update_dataset(&pool, 999, "name", &headers, "custom", None, None)
            .await
            .unwrap();

        assert!(!updated);
    }

    #[tokio::test]
    async fn test_update_dataset_with_sample_data() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["id".to_string()];
        let original_data = vec![vec!["1".to_string()]];

        let dataset_id = save_dataset(
            &pool,
            "test",
            &headers,
            "custom",
            None,
            Some(&original_data),
        )
        .await
        .unwrap();

        let new_data = vec![
            vec!["10".to_string()],
            vec!["20".to_string()],
            vec!["30".to_string()],
        ];

        update_dataset(
            &pool,
            dataset_id,
            "updated",
            &headers,
            "custom",
            None,
            Some(&new_data),
        )
        .await
        .unwrap();

        let result = get_datasets(&pool, dataset_id).await.unwrap();
        let (dataset, csv_data) = result.unwrap();
        assert_eq!(dataset.row_count, 3);
        assert_eq!(csv_data.rows.len(), 3);
        assert_eq!(csv_data.rows[2][0], "30");
    }

    #[tokio::test]
    async fn test_duplicate_dataset_basic() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["id".to_string(), "name".to_string()];
        let sample_data = vec![vec!["1".to_string(), "Test".to_string()]];

        let original_id = save_dataset(
            &pool,
            "original",
            &headers,
            "custom",
            None,
            Some(&sample_data),
        )
        .await
        .unwrap();

        let duplicate_id = duplicate_dataset(&pool, original_id, None)
            .await
            .unwrap()
            .unwrap();

        assert_ne!(original_id, duplicate_id);

        let result = get_datasets(&pool, duplicate_id).await.unwrap();
        let (dataset, csv_data) = result.unwrap();
        assert_eq!(dataset.name, "original (Copy)");
        assert_eq!(csv_data.headers, headers);
        assert_eq!(csv_data.rows, sample_data);
    }

    #[tokio::test]
    async fn test_duplicate_dataset_custom_name() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["col1".to_string()];

        let original_id = save_dataset(&pool, "original", &headers, "custom", None, None)
            .await
            .unwrap();

        let duplicate_id = duplicate_dataset(&pool, original_id, Some("custom_copy"))
            .await
            .unwrap()
            .unwrap();

        let result = get_datasets(&pool, duplicate_id).await.unwrap();
        let (dataset, _) = result.unwrap();
        assert_eq!(dataset.name, "custom_copy");
    }

    #[tokio::test]
    async fn test_duplicate_dataset_not_found() {
        let pool = test_utils::setup_test_db().await;

        let result = duplicate_dataset(&pool, 999, None).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_duplicate_dataset_with_column_types() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["id".to_string(), "email".to_string()];
        let mut column_types = HashMap::new();
        column_types.insert("id".to_string(), "number".to_string());
        column_types.insert("email".to_string(), "email".to_string());

        let original_id = save_dataset(
            &pool,
            "original",
            &headers,
            "custom",
            Some(&column_types),
            None,
        )
        .await
        .unwrap();

        let duplicate_id = duplicate_dataset(&pool, original_id, None)
            .await
            .unwrap()
            .unwrap();

        let result = get_datasets(&pool, duplicate_id).await.unwrap();
        let (dataset, _) = result.unwrap();
        assert!(dataset.column_types.is_some());
        let dup_types: HashMap<String, String> =
            serde_json::from_str(&dataset.column_types.unwrap()).unwrap();
        assert_eq!(dup_types.len(), 2);
        assert_eq!(dup_types.get("id"), Some(&"number".to_string()));
    }

    #[tokio::test]
    async fn test_dataset_cascade_delete() {
        let pool = test_utils::setup_test_db().await;
        let headers = vec!["col1".to_string()];
        let sample_data = vec![
            vec!["a".to_string()],
            vec!["b".to_string()],
            vec!["c".to_string()],
        ];

        let dataset_id = save_dataset(&pool, "test", &headers, "custom", None, Some(&sample_data))
            .await
            .unwrap();

        delete_dataset(&pool, dataset_id).await.unwrap();

        let rows = sqlx::query!(
            "SELECT COUNT(*) as count FROM dataset_rows WHERE dataset_id = ?",
            dataset_id
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(rows.count, 0);
    }
}
