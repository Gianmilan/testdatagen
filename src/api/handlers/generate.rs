use crate::csv_parser::CsvData;
use crate::generators::{DataGenerator, FlexibleGenerator};
use actix_web::{HttpResponse, Responder, web};
use log::{debug, info};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct GenerateRequest {
    pub row_count: Option<usize>,
    pub save: Option<bool>,
    pub headers: Option<Vec<String>>,
}

fn generate_with_generator<G: DataGenerator>(generator: G, row_count: usize) -> CsvData {
    let mut rng = rand::rng();
    let headers = generator.headers();

    debug!("Generating {} rows using functional map", row_count);
    let rows: Vec<Vec<String>> = (1..=row_count)
        .map(|i| generator.generate_row(i, &mut rng))
        .collect();

    CsvData { headers, rows }
}

pub async fn generate_placeholder(req: web::Json<GenerateRequest>) -> impl Responder {
    let row_count = req.row_count.unwrap_or(20);

    info!("Generating {} placeholder rows", row_count);

    if row_count == 0 || row_count > 1000 {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "row_count must be between 1 and 1000"
        }));
    }

    // Use provided headers or default ones
    let headers = req.headers.clone().unwrap_or_else(|| {
        vec![
            "id".to_string(),
            "name".to_string(),
            "email".to_string(),
            "age".to_string(),
            "city".to_string(),
        ]
    });

    let generator = FlexibleGenerator::new(headers.clone());
    let csv_data = generate_with_generator(generator, row_count);

    info!("Generated {} rows successfully", row_count);

    HttpResponse::Ok().json(serde_json::json!({
        "data": csv_data,
        "message": format!("Generated {} rows successfully", row_count)
    }))
}
