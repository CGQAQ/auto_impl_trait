pub trait Perimeter {
    fn perimeter(&self) -> f64;
}

impl Perimeter for Square {
    fn perimeter(&self) -> f64 {
        self.side * 4.0
    }
}
