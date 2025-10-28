use actix_web::{HttpResponse, web};
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::csv_parser::CsvData;
use super::SuccessResponse;

#[derive(Deserialize, Serialize)]
pub struct GenerateRequest {
    pub row_count: Option<usize>,
}

pub async fn generate_placeholder(payload: web::Json<GenerateRequest>) -> HttpResponse {
    let headers = vec![
        "ID".to_string(),
        "Name".to_string(),
        "Email".to_string(),
        "Age".to_string(),
        "City".to_string(),
    ];

    let names = vec!["Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry"];
    let cities = vec![
        "New York",
        "London",
        "Tokyo",
        "Paris",
        "Berlin",
        "Sydney",
        "Toronto",
    ];

    let mut rng = rand::rng();
    let mut rows = Vec::new();

    let row_count = payload.row_count.unwrap_or(20).min(1000);

    for i in 1..=row_count {
        rows.push(vec![
            i.to_string(),
            names.choose(&mut rng).unwrap().to_string(),
            format!("user{}@example.com", i),
            rng.random_range(20..70).to_string(),
            cities.choose(&mut rng).unwrap().to_string(),
        ]);
    }

    let csv_data = CsvData { headers, rows };

    HttpResponse::Ok().json(SuccessResponse {
        data: csv_data,
        message: format!("Generated {} sample row{}", row_count, if row_count == 1 { "" } else { "s" }),
    })
}
