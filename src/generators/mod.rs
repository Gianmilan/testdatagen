mod smart_generator;

pub use smart_generator::SmartGenerator;

use rand::Rng;

pub trait DataGenerator {
    fn headers(&self) -> Vec<String>;
    fn generate_row(&self, index: usize, rng: &mut impl Rng) -> Vec<String>;
}
