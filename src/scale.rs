use crate::Square;

impl Square {
    pub fn scale(&mut self, scale: i32) {
        self.side *= scale;
    }
}