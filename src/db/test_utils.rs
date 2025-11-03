use sqlx::SqlitePool;

pub async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();

    sqlx::query(
        r#"CREATE TABLE datasets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                row_count INTEGER NOT NULL,
                column_count INTEGER NOT NULL,
                data_type TEXT NOT NULL,
                headers TEXT NOT NULL,
                column_types TEXT,
                has_sample_data BOOLEAN NOT NULL
            )"#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        r#"CREATE TABLE dataset_rows (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                dataset_id INTEGER NOT NULL,
                row_index INTEGER NOT NULL,
                row_data TEXT NOT NULL,
                FOREIGN KEY (dataset_id) REFERENCES datasets(id) ON DELETE CASCADE
            )"#,
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}
