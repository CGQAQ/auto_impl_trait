use crate::{Square};

impl Square {
    pub fn area(&self) -> i32 {
        self.side * self.side
    }
}
