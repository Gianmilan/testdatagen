pub mod datasets;
pub mod extract_headers;
pub mod generate;
pub mod health;
pub mod upload;

use crate::csv_parser::CsvData;
use serde::{Deserialize, Serialize};

pub use extract_headers::extract_headers;
pub use generate::generate_placeholder;
pub use health::health_check;
pub use upload::upload_csv;

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
