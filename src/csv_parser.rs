use csv::{Reader};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
pub struct CsvData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub fn parse_csv_from_bytes(data: &[u8]) -> Result<CsvData, Box<dyn Error>> {
    let mut rdr = Reader::from_reader(data);

    let headers = rdr.headers()?.iter().map(|s| s.to_string()).collect();
    let rows = result_in_records::<&[u8]>(&mut rdr)?;

    Ok(CsvData { headers, rows })
}

pub fn parse_csv_from_file(path: &str) -> Result<CsvData, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = Reader::from_reader(file);

    let headers = rdr.headers()?.iter().map(|s| s.to_string()).collect();
    let rows = result_in_records::<File>(&mut rdr)?;

    Ok(CsvData { headers, rows })
}

pub fn result_in_records<R: Read>(rdr: &mut Reader<R>) -> Result<Vec<Vec<String>>, Box<dyn Error>> {
    let mut rows = Vec::new();

    for result in rdr.records() {
        let record = result?;
        let row: Vec<String> = record.iter().map(|s| s.to_string()).collect();
        rows.push(row);
    }

    if rows.is_empty() {
        Err("No rows found".into())
    } else {
        Ok(rows)
    }
}
