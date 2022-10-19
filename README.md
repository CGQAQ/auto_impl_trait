# auto_impl_trait

```rust
#[auto_impl_trait("./src/rect_trait.rs", Rect)]
#[doc = "Test this will keep after expand"]
#[derive(Debug)]
struct Square {
    side: i32,
}
```

Will expand to

```rust
mod item;
mod area;
mod perimeter;
mod scale;
mod ____CGQAQ__SUPER_TRAIT____ {
    use std::ops::{Add, Sub, Mul, Div};

    pub trait Rect {
        type Item: Add + Sub + Mul + Div;
        fn area(&self) -> Self::Item;
        fn perimeter(&self) -> Self::Item;
        fn scale(&mut self, scale: Self::Item);
    }
}
use ____CGQAQ__SUPER_TRAIT____::Rect;
#[doc = "Test this will keep after expand"]
#[derive(Debug)]
struct Square {
    side: i32,
}
impl Rect for Square {
    type Item = crate::item::Item;
    fn area(&self) -> Self::Item { <dyn crate::area::Area>::area(self) }
    fn perimeter(&self) -> Self::Item { <dyn crate::perimeter::Perimeter>::perimeter(self) }
    fn scale(&mut self, scale: Self::Item) { <dyn crate::scale::Scale>::scale(self, scale) }
}
```