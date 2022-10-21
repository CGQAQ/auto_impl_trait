pub mod runtime {
    use std::ops::{ Add, Sub, Mul, Div };

    pub trait Rect {
        type Item: Add + Sub + Mul + Div;

        fn area(&self) -> Self::Item;
        fn perimeter(&self) -> Self::Item;

        fn scale(&mut self, scale: Self::Item);

        // fn async_test() {}
    }
}
