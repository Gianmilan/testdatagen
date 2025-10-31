-- Add column_types and has_sample_data to datasets table
-- column_types: Optional JSON object mapping column names to data types
-- has_sample_data: Boolean flag indicating if sample rows are stored

ALTER TABLE datasets ADD COLUMN column_types TEXT DEFAULT NULL;
ALTER TABLE datasets ADD COLUMN has_sample_data BOOLEAN DEFAULT FALSE NOT NULL;
