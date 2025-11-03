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

    // Limit sample data to 100 rows for storage efficiency
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

    // Only save rows if sample_data is provided
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
    // Check if dataset exists
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

    // Limit sample data to 100 rows for storage efficiency
    let limited_sample_data =
        sample_data.map(|data| if data.len() > 100 { &data[..100] } else { data });

    let row_count = limited_sample_data.map(|d| d.len()).unwrap_or(0) as i64;
    let column_count = headers.len() as i64;
    let has_sample_data = limited_sample_data.is_some();

    // Update dataset metadata
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

    // Delete existing rows
    sqlx::query!("DELETE FROM dataset_rows WHERE dataset_id = ?", id)
        .execute(&mut *tx)
        .await?;

    // Insert new rows if sample_data is provided
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
    // Fetch the original dataset
    let original = get_datasets(pool, id).await?;

    let (dataset, csv_data) = match original {
        Some(data) => data,
        None => return Ok(None),
    };

    // Parse headers and column types
    let headers: Vec<String> = serde_json::from_str(&dataset.headers)?;
    let column_types: Option<std::collections::HashMap<String, String>> = match dataset.column_types
    {
        Some(ref types_json) => Some(serde_json::from_str(types_json)?),
        None => None,
    };

    // Generate new name if not provided
    let default_name = format!("{} (Copy)", dataset.name);
    let name = new_name.unwrap_or(&default_name);

    // Prepare sample data if original has it
    let sample_data = if dataset.has_sample_data && !csv_data.rows.is_empty() {
        Some(csv_data.rows.as_slice())
    } else {
        None
    };

    // Save as new dataset
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
