use crate::Square;

pub trait Scale {
    fn scale(&mut self, scale: i32);
}

impl Scale for Square {
    fn scale(&mut self, scale: i32) {
       self.side *= scale;
    }
}