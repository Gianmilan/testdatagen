use super::DataGenerator;
use rand::Rng;

pub struct SmartGenerator {
    headers: Vec<String>,
}

impl SmartGenerator {
    pub fn new(headers: Vec<String>) -> Self {
        Self { headers }
    }

    // TODO: ML-based detection(Ollama)
    fn detect_data_type(&self, header: &str) -> DataType {
        let header_lower = header.to_lowercase();

        if header_lower.contains("id") || header_lower == "id" {
            return DataType::Id;
        }

        if header_lower.contains("name") {
            return DataType::Name;
        }

        if header_lower.contains("email") || header_lower.contains("mail") {
            return DataType::Email;
        }

        if header_lower.contains("age") {
            return DataType::Age;
        }

        if header_lower.contains("city") {
            return DataType::City;
        }

        if header_lower.contains("country") {
            return DataType::Country;
        }

        if header_lower.contains("phone") || header_lower.contains("tel") {
            return DataType::Phone;
        }

        if header_lower.contains("date") {
            return DataType::Date;
        }

        if header_lower.contains("price")
            || header_lower.contains("cost")
            || header_lower.contains("amount")
            || header_lower.contains("salary")
        {
            return DataType::Money;
        }

        DataType::Text
    }

    fn generate_value(&self, data_type: &DataType, index: usize, rng: &mut impl Rng) -> String {
        match data_type {
            DataType::Id => index.to_string(),
            DataType::Name => generate_name(rng),
            DataType::Email => generate_email(rng),
            DataType::Age => rng.random_range(18..=80).to_string(),
            DataType::City => generate_city(rng),
            DataType::Country => generate_country(rng),
            DataType::Phone => generate_phone(rng),
            DataType::Date => generate_date(rng),
            DataType::Money => format!("{:.2}", rng.random_range(10.0..=10000.0)),
            DataType::Text => generate_random_text(rng),
        }
    }
}

impl DataGenerator for SmartGenerator {
    fn headers(&self) -> Vec<String> {
        self.headers.clone()
    }

    fn generate_row(&self, index: usize, rng: &mut impl Rng) -> Vec<String> {
        self.headers
            .iter()
            .map(|header| {
                let data_type = self.detect_data_type(header);
                self.generate_value(&data_type, index, rng)
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
enum DataType {
    Id,
    Name,
    Email,
    Age,
    City,
    Country,
    Phone,
    Date,
    Money,
    Text, // Fallback for unknown types
}

fn generate_name(rng: &mut impl Rng) -> String {
    const FIRST_NAMES: &[&str] = &[
        "James",
        "Mary",
        "John",
        "Patricia",
        "Robert",
        "Jennifer",
        "Michael",
        "Linda",
        "William",
        "Elizabeth",
        "David",
        "Barbara",
        "Richard",
        "Susan",
        "Joseph",
        "Jessica",
        "Thomas",
        "Sarah",
        "Charles",
        "Karen",
        "Emma",
        "Oliver",
        "Sophia",
        "Liam",
    ];
    const LAST_NAMES: &[&str] = &[
        "Smith",
        "Johnson",
        "Williams",
        "Brown",
        "Jones",
        "Garcia",
        "Miller",
        "Davis",
        "Rodriguez",
        "Martinez",
        "Hernandez",
        "Lopez",
        "Gonzalez",
        "Wilson",
        "Anderson",
        "Thomas",
        "Taylor",
        "Moore",
        "Jackson",
        "Martin",
        "Lee",
        "Thompson",
        "White",
        "Harris",
    ];

    let first = FIRST_NAMES[rng.random_range(0..FIRST_NAMES.len())];
    let last = LAST_NAMES[rng.random_range(0..LAST_NAMES.len())];
    format!("{} {}", first, last)
}

fn generate_email(rng: &mut impl Rng) -> String {
    const DOMAINS: &[&str] = &[
        "gmail.com",
        "yahoo.com",
        "outlook.com",
        "example.com",
        "test.com",
    ];

    let username: String = (0..8)
        .map(|_| {
            const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
            CHARSET[rng.random_range(0..CHARSET.len())] as char
        })
        .collect();

    let domain = DOMAINS[rng.random_range(0..DOMAINS.len())];
    format!("{}@{}", username, domain)
}

fn generate_city(rng: &mut impl Rng) -> String {
    const CITIES: &[&str] = &[
        "New York",
        "Los Angeles",
        "Chicago",
        "Houston",
        "Phoenix",
        "Philadelphia",
        "San Antonio",
        "San Diego",
        "Dallas",
        "San Jose",
        "Austin",
        "Jacksonville",
        "London",
        "Paris",
        "Tokyo",
        "Berlin",
        "Madrid",
        "Rome",
        "Sydney",
        "Toronto",
    ];

    CITIES[rng.random_range(0..CITIES.len())].to_string()
}

fn generate_country(rng: &mut impl Rng) -> String {
    const COUNTRIES: &[&str] = &[
        "United States",
        "Canada",
        "United Kingdom",
        "Germany",
        "France",
        "Japan",
        "Australia",
        "Spain",
        "Italy",
        "Brazil",
        "Mexico",
        "Netherlands",
    ];

    COUNTRIES[rng.random_range(0..COUNTRIES.len())].to_string()
}

fn generate_phone(rng: &mut impl Rng) -> String {
    format!(
        "+1-{:03}-{:03}-{:04}",
        rng.random_range(200..=999),
        rng.random_range(200..=999),
        rng.random_range(1000..=9999)
    )
}

fn generate_date(rng: &mut impl Rng) -> String {
    let year = rng.random_range(2020..=2025);
    let month = rng.random_range(1..=12);
    let day = rng.random_range(1..=28); // Simple approach to avoid invalid dates
    format!("{:04}-{:02}-{:02}", year, month, day)
}

fn generate_random_text(rng: &mut impl Rng) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let length = rng.random_range(5..=12);

    (0..length)
        .map(|_| CHARSET[rng.random_range(0..CHARSET.len())] as char)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_detect_data_type_id() {
        let generator = SmartGenerator::new(vec![]);
        assert!(matches!(generator.detect_data_type("id"), DataType::Id));
        assert!(matches!(generator.detect_data_type("ID"), DataType::Id));
        assert!(matches!(generator.detect_data_type("user_id"), DataType::Id));
        assert!(matches!(generator.detect_data_type("product_id"), DataType::Id));
    }

    #[test]
    fn test_detect_data_type_name() {
        let generator = SmartGenerator::new(vec![]);
        assert!(matches!(generator.detect_data_type("name"), DataType::Name));
        assert!(matches!(generator.detect_data_type("Name"), DataType::Name));
        assert!(matches!(generator.detect_data_type("first_name"), DataType::Name));
        assert!(matches!(generator.detect_data_type("last_name"), DataType::Name));
        assert!(matches!(generator.detect_data_type("username"), DataType::Name));
    }

    #[test]
    fn test_detect_data_type_email() {
        let generator = SmartGenerator::new(vec![]);
        assert!(matches!(generator.detect_data_type("email"), DataType::Email));
        assert!(matches!(generator.detect_data_type("Email"), DataType::Email));
        assert!(matches!(generator.detect_data_type("mail"), DataType::Email));
        assert!(matches!(generator.detect_data_type("user_email"), DataType::Email));
    }

    #[test]
    fn test_detect_data_type_age() {
        let generator = SmartGenerator::new(vec![]);
        assert!(matches!(generator.detect_data_type("age"), DataType::Age));
        assert!(matches!(generator.detect_data_type("Age"), DataType::Age));
        assert!(matches!(generator.detect_data_type("user_age"), DataType::Age));
    }

    #[test]
    fn test_detect_data_type_city() {
        let generator = SmartGenerator::new(vec![]);
        assert!(matches!(generator.detect_data_type("city"), DataType::City));
        assert!(matches!(generator.detect_data_type("City"), DataType::City));
        assert!(matches!(generator.detect_data_type("home_city"), DataType::City));
    }

    #[test]
    fn test_detect_data_type_country() {
        let generator = SmartGenerator::new(vec![]);
        assert!(matches!(
            generator.detect_data_type("country"),
            DataType::Country
        ));
        assert!(matches!(
            generator.detect_data_type("Country"),
            DataType::Country
        ));
        assert!(matches!(
            generator.detect_data_type("home_country"),
            DataType::Country
        ));
    }

    #[test]
    fn test_detect_data_type_phone() {
        let generator = SmartGenerator::new(vec![]);
        assert!(matches!(generator.detect_data_type("phone"), DataType::Phone));
        assert!(matches!(generator.detect_data_type("Phone"), DataType::Phone));
        assert!(matches!(generator.detect_data_type("tel"), DataType::Phone));
        assert!(matches!(
            generator.detect_data_type("telephone"),
            DataType::Phone
        ));
    }

    #[test]
    fn test_detect_data_type_date() {
        let generator = SmartGenerator::new(vec![]);
        assert!(matches!(generator.detect_data_type("date"), DataType::Date));
        assert!(matches!(generator.detect_data_type("Date"), DataType::Date));
        assert!(matches!(
            generator.detect_data_type("birth_date"),
            DataType::Date
        ));
        assert!(matches!(
            generator.detect_data_type("created_date"),
            DataType::Date
        ));
    }

    #[test]
    fn test_detect_data_type_money() {
        let generator = SmartGenerator::new(vec![]);
        assert!(matches!(generator.detect_data_type("price"), DataType::Money));
        assert!(matches!(generator.detect_data_type("cost"), DataType::Money));
        assert!(matches!(
            generator.detect_data_type("amount"),
            DataType::Money
        ));
        assert!(matches!(
            generator.detect_data_type("salary"),
            DataType::Money
        ));
        assert!(matches!(
            generator.detect_data_type("total_price"),
            DataType::Money
        ));
    }

    #[test]
    fn test_detect_data_type_text_fallback() {
        let generator = SmartGenerator::new(vec![]);
        assert!(matches!(
            generator.detect_data_type("description"),
            DataType::Text
        ));
        assert!(matches!(
            generator.detect_data_type("random"),
            DataType::Text
        ));
        assert!(matches!(generator.detect_data_type("xyz"), DataType::Text));
    }

    #[test]
    fn test_generate_name() {
        let mut rng = StdRng::seed_from_u64(42);
        let name = generate_name(&mut rng);
        assert!(name.contains(' '));
        assert!(name.len() > 3);
        let parts: Vec<&str> = name.split(' ').collect();
        assert_eq!(parts.len(), 2);
    }

    #[test]
    fn test_generate_email() {
        let mut rng = StdRng::seed_from_u64(42);
        let email = generate_email(&mut rng);
        assert!(email.contains('@'));
        assert!(email.contains('.'));
        let parts: Vec<&str> = email.split('@').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].len(), 8);
    }

    #[test]
    fn test_generate_city() {
        let mut rng = StdRng::seed_from_u64(42);
        let city = generate_city(&mut rng);
        assert!(!city.is_empty());
        assert!(city.chars().next().unwrap().is_uppercase());
    }

    #[test]
    fn test_generate_country() {
        let mut rng = StdRng::seed_from_u64(42);
        let country = generate_country(&mut rng);
        assert!(!country.is_empty());
        assert!(country.chars().next().unwrap().is_uppercase());
    }

    #[test]
    fn test_generate_phone() {
        let mut rng = StdRng::seed_from_u64(42);
        let phone = generate_phone(&mut rng);
        assert!(phone.starts_with("+1-"));
        assert!(phone.matches('-').count() == 3);
        let parts: Vec<&str> = phone.split('-').collect();
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[1].len(), 3);
        assert_eq!(parts[2].len(), 3);
        assert_eq!(parts[3].len(), 4);
    }

    #[test]
    fn test_generate_date() {
        let mut rng = StdRng::seed_from_u64(42);
        let date = generate_date(&mut rng);
        assert!(date.matches('-').count() == 2);
        let parts: Vec<&str> = date.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].len(), 4);
        assert_eq!(parts[1].len(), 2);
        assert_eq!(parts[2].len(), 2);
        let year: i32 = parts[0].parse().unwrap();
        assert!(year >= 2020 && year <= 2025);
        let month: i32 = parts[1].parse().unwrap();
        assert!(month >= 1 && month <= 12);
        let day: i32 = parts[2].parse().unwrap();
        assert!(day >= 1 && day <= 28);
    }

    #[test]
    fn test_generate_random_text() {
        let mut rng = StdRng::seed_from_u64(42);
        let text = generate_random_text(&mut rng);
        assert!(text.len() >= 5 && text.len() <= 12);
        assert!(text.chars().all(|c| c.is_alphanumeric()));
    }

    #[test]
    fn test_generate_value_id() {
        let generator = SmartGenerator::new(vec![]);
        let mut rng = StdRng::seed_from_u64(42);
        let value = generator.generate_value(&DataType::Id, 5, &mut rng);
        assert_eq!(value, "5");
    }

    #[test]
    fn test_generate_value_age() {
        let generator = SmartGenerator::new(vec![]);
        let mut rng = StdRng::seed_from_u64(42);
        let value = generator.generate_value(&DataType::Age, 0, &mut rng);
        let age: i32 = value.parse().unwrap();
        assert!(age >= 18 && age <= 80);
    }

    #[test]
    fn test_generate_value_money() {
        let generator = SmartGenerator::new(vec![]);
        let mut rng = StdRng::seed_from_u64(42);
        let value = generator.generate_value(&DataType::Money, 0, &mut rng);
        assert!(value.contains('.'));
        let parts: Vec<&str> = value.split('.').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[1].len(), 2);
        let amount: f64 = value.parse().unwrap();
        assert!(amount >= 10.0 && amount <= 10000.0);
    }

    #[test]
    fn test_smart_generator_new() {
        let headers = vec!["id".to_string(), "name".to_string(), "email".to_string()];
        let generator = SmartGenerator::new(headers.clone());
        assert_eq!(generator.headers(), headers);
    }

    #[test]
    fn test_smart_generator_headers() {
        let headers = vec!["col1".to_string(), "col2".to_string()];
        let generator = SmartGenerator::new(headers.clone());
        assert_eq!(generator.headers(), headers);
    }

    #[test]
    fn test_smart_generator_generate_row() {
        let headers = vec![
            "id".to_string(),
            "name".to_string(),
            "email".to_string(),
            "age".to_string(),
        ];
        let generator = SmartGenerator::new(headers.clone());
        let mut rng = StdRng::seed_from_u64(42);
        let row = generator.generate_row(1, &mut rng);
        assert_eq!(row.len(), 4);
        assert_eq!(row[0], "1");
        assert!(row[1].contains(' '));
        assert!(row[2].contains('@'));
        let age: i32 = row[3].parse().unwrap();
        assert!(age >= 18 && age <= 80);
    }

    #[test]
    fn test_smart_generator_generate_multiple_rows() {
        let headers = vec!["id".to_string(), "name".to_string()];
        let generator = SmartGenerator::new(headers);
        let mut rng = StdRng::seed_from_u64(42);
        let row1 = generator.generate_row(1, &mut rng);
        let row2 = generator.generate_row(2, &mut rng);
        assert_eq!(row1[0], "1");
        assert_eq!(row2[0], "2");
        assert_ne!(row1[1], row2[1]);
    }

    #[test]
    fn test_smart_generator_all_data_types() {
        let headers = vec![
            "id".to_string(),
            "name".to_string(),
            "email".to_string(),
            "age".to_string(),
            "city".to_string(),
            "country".to_string(),
            "phone".to_string(),
            "date".to_string(),
            "price".to_string(),
            "description".to_string(),
        ];
        let generator = SmartGenerator::new(headers);
        let mut rng = StdRng::seed_from_u64(42);
        let row = generator.generate_row(10, &mut rng);
        assert_eq!(row.len(), 10);
        assert_eq!(row[0], "10");
        assert!(row[1].contains(' '));
        assert!(row[2].contains('@'));
        let age: i32 = row[3].parse().unwrap();
        assert!(age >= 18 && age <= 80);
        assert!(!row[4].is_empty());
        assert!(!row[5].is_empty());
        assert!(row[6].starts_with("+1-"));
        assert!(row[7].matches('-').count() == 2);
        assert!(row[8].contains('.'));
        assert!(!row[9].is_empty());
    }

    #[test]
    fn test_deterministic_with_seed() {
        let headers = vec!["name".to_string(), "email".to_string()];
        let generator = SmartGenerator::new(headers);
        let mut rng1 = StdRng::seed_from_u64(123);
        let mut rng2 = StdRng::seed_from_u64(123);
        let row1 = generator.generate_row(1, &mut rng1);
        let row2 = generator.generate_row(1, &mut rng2);
        assert_eq!(row1, row2);
    }

    #[test]
    fn test_non_deterministic_different_seeds() {
        let headers = vec!["name".to_string()];
        let generator = SmartGenerator::new(headers);
        let mut rng1 = StdRng::seed_from_u64(123);
        let mut rng2 = StdRng::seed_from_u64(456);
        let row1 = generator.generate_row(1, &mut rng1);
        let row2 = generator.generate_row(1, &mut rng2);
        assert_ne!(row1, row2);
    }
}
