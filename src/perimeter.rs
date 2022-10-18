use crate::Square;

pub trait Perimeter {
    fn perimeter(&self) -> i32;
}

impl Perimeter for Square {
    fn perimeter(&self) -> i32 {
        self.side * 4
    }
}
