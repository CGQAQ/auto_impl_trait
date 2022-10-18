use std::ops::{ Add, Sub, Mul, Div };

trait Rect {
    type Item: Add + Sub + Mul + Div;

    fn area(&self) -> Item;
    fn perimeter(&self) -> Item;

    fn scale(&mut self, scale: Item);
}
