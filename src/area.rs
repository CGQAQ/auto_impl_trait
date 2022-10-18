use crate::Square;

pub trait Area {
    fn area(&self) -> i32;
}

impl Area for Square {
    fn area(&self) -> i32 {
        self.side * self.side
    }
}
