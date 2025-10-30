-- Add migration script here
CREATE TABLE datasets (
                          id INTEGER PRIMARY KEY AUTOINCREMENT,
                          name TEXT NOT NULL,
                          created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                          row_count INTEGER NOT NULL,
                          column_count INTEGER NOT NULL,
                          data_type TEXT NOT NULL,  -- 'generated' or 'uploaded'
                          headers TEXT NOT NULL      -- JSON array of header names
);

-- Store the actual data rows
CREATE TABLE dataset_rows (
                              id INTEGER PRIMARY KEY AUTOINCREMENT,
                              dataset_id INTEGER NOT NULL,
                              row_index INTEGER NOT NULL,
                              row_data TEXT NOT NULL,    -- JSON array of cell values
                              FOREIGN KEY (dataset_id) REFERENCES datasets(id) ON DELETE CASCADE
);

-- Index for faster queries
CREATE INDEX idx_dataset_rows_dataset_id ON dataset_rows(dataset_id);
