use crate::{Square};
use crate::__Rect_scale__::{__Rect_scale__, Item};

impl __Rect_scale__ for Square {
    fn scale(&mut self, scale: Item) {
        self.side *= scale;
    }
}