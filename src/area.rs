use crate::{Square};
use crate::__Rect_area__::{__Rect_area__, Item};

impl __Rect_area__ for Square {
    fn area(&self) -> Item {
        self.side * self.side
    }
}
