mod user;

pub use user::UserGenerator;

use rand::Rng;

pub trait DataGenerator {
    fn headers(&self) -> Vec<String>;
    fn generate_row(&self, index: usize, rng: &mut impl Rng) -> Vec<String>;
}
