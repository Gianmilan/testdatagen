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
