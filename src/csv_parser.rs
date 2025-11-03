use csv::Reader;
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

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_parse_csv_from_bytes_basic() {
        let csv_bytes = b"id,name,age\n1,Alice,30\n2,Bob,25\n3,Charlie,35\n";
        let result = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(result.headers, vec!["id", "name", "age"]);
        assert_eq!(result.rows.len(), 3);
        assert_eq!(result.rows[0], vec!["1", "Alice", "30"]);
        assert_eq!(result.rows[1], vec!["2", "Bob", "25"]);
        assert_eq!(result.rows[2], vec!["3", "Charlie", "35"]);
    }

    #[test]
    fn test_parse_csv_from_bytes_headers_only() {
        let csv_bytes = b"id,name,email,created_at\n";
        let result = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(result.headers, vec!["id", "name", "email", "created_at"]);
        assert_eq!(result.rows.len(), 0);
    }

    #[test]
    fn test_parse_csv_from_bytes_single_column() {
        let csv_bytes = b"id\n1\n2\n3\n4\n5\n";
        let result = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(result.headers, vec!["id"]);
        assert_eq!(result.rows.len(), 5);
        assert_eq!(result.rows[0], vec!["1"]);
        assert_eq!(result.rows[4], vec!["5"]);
    }

    #[test]
    fn test_parse_csv_from_bytes_single_row() {
        let csv_bytes = b"name,value\ntest,42\n";
        let result = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(result.headers, vec!["name", "value"]);
        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.rows[0], vec!["test", "42"]);
    }

    #[test]
    fn test_parse_csv_from_bytes_empty() {
        let csv_bytes = b"";
        let result = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(result.headers.len(), 0);
        assert_eq!(result.rows.len(), 0);
    }

    #[test]
    fn test_parse_csv_from_bytes_quoted_commas() {
        let csv_bytes =
            b"id,name,address\n1,John,\"123 Main St, Apt 4\"\n2,Jane,\"456 Oak Ave, Suite 10\"\n";
        let result = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(result.headers, vec!["id", "name", "address"]);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][2], "123 Main St, Apt 4");
        assert_eq!(result.rows[1][2], "456 Oak Ave, Suite 10");
    }

    #[test]
    fn test_parse_csv_from_bytes_quoted_newlines() {
        let csv_bytes = b"id,description\n1,\"Line 1\nLine 2\"\n2,\"Single line\"\n";
        let result = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(result.headers, vec!["id", "description"]);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0][1], "Line 1\nLine 2");
        assert_eq!(result.rows[1][1], "Single line");
    }

    #[test]
    fn test_parse_csv_from_bytes_special_header_chars() {
        let csv_bytes = b"User ID,Full Name,Email@Address,Created_At\n1,Test User,test@example.com,2024-01-01\n";
        let result = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(
            result.headers,
            vec!["User ID", "Full Name", "Email@Address", "Created_At"]
        );
        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_parse_csv_from_bytes_whitespace_headers() {
        let csv_bytes = b" id , name , age \n1,John,20\n";
        let result = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(result.headers, vec![" id ", " name ", " age "]);
        assert_eq!(result.rows.len(), 1);
    }

    #[test]
    fn test_parse_csv_from_bytes_whitespace_values() {
        let csv_bytes = b"id,name,city\n1, John Smith , Boston \n2, Jane Doe , New York \n";
        let result = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(result.rows[0][1], " John Smith ");
        assert_eq!(result.rows[0][2], " Boston ");
    }

    #[test]
    fn test_parse_csv_from_bytes_unicode() {
        let csv_bytes = "名前,年齢,都市\n太郎,25,東京\n花子,30,大阪\n".as_bytes();
        let result = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(result.headers, vec!["名前", "年齢", "都市"]);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0], vec!["太郎", "25", "東京"]);
        assert_eq!(result.rows[1], vec!["花子", "30", "大阪"]);
    }

    #[test]
    fn test_parse_csv_from_bytes_emojis() {
        let csv_bytes = "name,status,emoji\nHappy,😊,✅\nSad,😢,❌\n".as_bytes();
        let result = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(result.headers, vec!["name", "status", "emoji"]);
        assert_eq!(result.rows[0], vec!["Happy", "😊", "✅"]);
        assert_eq!(result.rows[1], vec!["Sad", "😢", "❌"]);
    }

    #[test]
    fn test_parse_csv_from_bytes_escaped_quotes() {
        let csv_bytes = b"id,message\n1,\"He said \"\"Hello\"\"\"\n2,\"She replied \"\"Hi\"\"\"\n";
        let result = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(result.headers, vec!["id", "message"]);
        assert_eq!(result.rows[0][1], "He said \"Hello\"");
        assert_eq!(result.rows[1][1], "She replied \"Hi\"");
    }

    #[test]
    fn test_parse_csv_from_bytes_empty_values() {
        let csv_bytes = b"id,name,email\n1,,alice@test.com\n2,Bob,\n3,,\n";
        let result = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(result.headers, vec!["id", "name", "email"]);
        assert_eq!(result.rows.len(), 3);
        assert_eq!(result.rows[0], vec!["1", "", "alice@test.com"]);
        assert_eq!(result.rows[1], vec!["2", "Bob", ""]);
        assert_eq!(result.rows[2], vec!["3", "", ""]);
    }

    #[test]
    fn test_parse_csv_from_bytes_many_columns() {
        let csv_bytes =
            b"col1,col2,col3,col4,col5,col6,col7,col8,col9,col10\na,b,c,d,e,f,g,h,i,j\n";
        let result = parse_csv_from_bytes(csv_bytes).unwrap();

        assert_eq!(result.headers.len(), 10);
        assert_eq!(result.rows[0].len(), 10);
        assert_eq!(result.rows[0][9], "j");
    }

    #[test]
    fn test_parse_csv_from_file_basic() {
        // Create a temporary test file
        let test_file = "/tmp/test_csv_parser_basic.csv";
        let mut file = File::create(test_file).unwrap();
        file.write_all(b"id,name,score\n1,Alice,95\n2,Bob,87\n")
            .unwrap();
        drop(file);

        let result = parse_csv_from_file(test_file).unwrap();

        assert_eq!(result.headers, vec!["id", "name", "score"]);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0], vec!["1", "Alice", "95"]);

        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_parse_csv_from_file_headers_only() {
        let test_file = "/tmp/test_csv_parser_headers_only.csv";
        let mut file = File::create(test_file).unwrap();
        file.write_all(b"column1,column2,column3\n").unwrap();
        drop(file);

        let result = parse_csv_from_file(test_file).unwrap();

        assert_eq!(result.headers, vec!["column1", "column2", "column3"]);
        assert_eq!(result.rows.len(), 0);

        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_parse_csv_from_file_not_found() {
        let result = parse_csv_from_file("/tmp/nonexistent_file_12345.csv");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_csv_from_file_empty() {
        let test_file = "/tmp/test_csv_parser_empty.csv";
        File::create(test_file).unwrap();

        let result = parse_csv_from_file(test_file).unwrap();

        assert_eq!(result.headers.len(), 0);
        assert_eq!(result.rows.len(), 0);

        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_parse_csv_from_file_unicode() {
        let test_file = "/tmp/test_csv_parser_unicode.csv";
        let mut file = File::create(test_file).unwrap();
        file.write_all("名前,メール\n太郎,taro@test.com\n".as_bytes())
            .unwrap();
        drop(file);

        let result = parse_csv_from_file(test_file).unwrap();

        assert_eq!(result.headers, vec!["名前", "メール"]);
        assert_eq!(result.rows[0], vec!["太郎", "taro@test.com"]);

        fs::remove_file(test_file).ok();
    }

    #[test]
    fn test_result_in_records_with_data() {
        let csv_bytes = b"h1,h2\nv1,v2\nv3,v4\n";
        let mut rdr = Reader::from_reader(&csv_bytes[..]);
        let _headers = rdr.headers().unwrap(); // Consume headers first

        let rows = result_in_records(&mut rdr).unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], vec!["v1", "v2"]);
        assert_eq!(rows[1], vec!["v3", "v4"]);
    }

    #[test]
    fn test_result_in_records_no_data() {
        let csv_bytes = b"h1,h2\n";
        let mut rdr = Reader::from_reader(&csv_bytes[..]);
        let _headers = rdr.headers().unwrap(); // Consume headers first

        let rows = result_in_records(&mut rdr).unwrap();

        assert_eq!(rows.len(), 0);
    }

    #[test]
    fn test_csv_data_serde() {
        let csv_data = CsvData {
            headers: vec!["id".to_string(), "name".to_string()],
            rows: vec![
                vec!["1".to_string(), "Alice".to_string()],
                vec!["2".to_string(), "Bob".to_string()],
            ],
        };

        let json = serde_json::to_string(&csv_data).unwrap();
        assert!(json.contains("headers"));
        assert!(json.contains("rows"));

        let deserialized: CsvData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.headers, csv_data.headers);
        assert_eq!(deserialized.rows, csv_data.rows);
    }
}
