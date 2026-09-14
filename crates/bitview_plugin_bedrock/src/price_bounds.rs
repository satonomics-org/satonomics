use bitview_traversable::Traversable;
use brk_types::Cents;

#[derive(Clone, Copy, Traversable)]
pub struct PriceBounds<T> {
    /// Lowest occupied price bucket.
    pub min: T,
    /// Highest occupied price bucket.
    pub max: T,
}

impl Default for PriceBounds<Cents> {
    fn default() -> Self {
        Self {
            min: Cents::NAN,
            max: Cents::NAN,
        }
    }
}

impl PriceBounds<Cents> {
    pub fn include(&mut self, price: Cents) {
        if self.min.is_nan() || price < self.min {
            self.min = price;
        }
        if self.max.is_nan() || price > self.max {
            self.max = price;
        }
    }
}
