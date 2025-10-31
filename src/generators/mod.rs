mod flexible;

pub use flexible::FlexibleGenerator;

use rand::Rng;

pub trait DataGenerator {
    fn headers(&self) -> Vec<String>;
    fn generate_row(&self, index: usize, rng: &mut impl Rng) -> Vec<String>;
}
