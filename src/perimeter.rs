use crate::Square;

impl Square {
    pub fn perimeter(&self) -> i32 {
        self.side * 4
    }
}
