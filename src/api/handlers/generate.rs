use actix_web::{HttpResponse, web};
use log::{info, debug};
use serde::{Deserialize, Serialize};
use std::time::Instant;

use super::SuccessResponse;
use crate::csv_parser::CsvData;
use crate::generators::{DataGenerator, UserGenerator};

#[derive(Deserialize, Serialize)]
pub struct GenerateRequest {
    pub row_count: Option<usize>,
}

pub async fn generate_placeholder(payload: web::Json<GenerateRequest>) -> HttpResponse {
    let start_time = Instant::now();

    let row_count = payload.row_count.unwrap_or(20).min(1000);
    info!("Received request to generate {} rows", row_count);

    debug!("Initializing UserGenerator");
    let generator = UserGenerator;
    let mut rng = rand::rng();

    let headers = generator.headers();

    debug!("Generating {} rows using functional map", row_count);
    let rows: Vec<Vec<String>> = (1..=row_count)
        .map(|i| generator.generate_row(i, &mut rng))
        .collect();

    let csv_data = CsvData { headers, rows };

    let elapsed = start_time.elapsed();
    info!("Successfully generated {} rows in {:.2}ms", row_count, elapsed.as_secs_f64() * 1000.0);

    HttpResponse::Ok().json(SuccessResponse {
        data: csv_data,
        message: format!(
            "Generated {} sample row{}",
            row_count,
            if row_count == 1 { "" } else { "s" }
        ),
    })
}
