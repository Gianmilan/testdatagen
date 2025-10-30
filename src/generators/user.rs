use rand::prelude::IndexedRandom;
use crate::generators::DataGenerator;
use rand::Rng;

pub struct UserGenerator;

impl DataGenerator for UserGenerator {
    fn headers(&self) -> Vec<String> {
        vec!["ID".into(), "Name".into(), "Email".into(), "Age".into(), "City".into()]
    }

    fn generate_row(&self, index: usize, rng: &mut impl Rng) -> Vec<String> {
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

        vec![
            index.to_string(),
            names.choose(rng).unwrap().to_string(),
            format!("user{}@example.com", index),
            rng.random_range(20..70).to_string(),
            cities.choose(rng).unwrap().to_string(),
        ]
    }
}
