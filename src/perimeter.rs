use crate::{Square};
use crate::__Rect_perimeter__::__Rect_perimeter__;

impl __Rect_perimeter__ for Square {
    fn perimeter(&self) -> i32 {
        self.side * 4
    }
}
