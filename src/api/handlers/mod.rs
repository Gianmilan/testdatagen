pub mod health;
pub mod upload;
pub mod generate;
pub mod datasets;

use serde::{Deserialize, Serialize};
use crate::csv_parser::CsvData;

// Re-export handler functions
pub use health::health_check;
pub use upload::upload_csv;
pub use generate::generate_placeholder;

// Shared response types
#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize, Deserialize)]
pub struct SuccessResponse {
    pub data: CsvData,
    pub message: String,
}
