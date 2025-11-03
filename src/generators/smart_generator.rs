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
