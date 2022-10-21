# auto_impl_trait

```rust
#[auto_impl_trait("./src/rect_trait.rs", Rect, "runtime")]
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
    pub mod runtime {
        use std::ops::{Add, Sub, Mul, Div};

        pub trait Rect {
            type Item: Add + Sub + Mul + Div;
            fn area(&self) -> Self::Item;
            fn perimeter(&self) -> Self::Item;
            fn scale(&mut self, scale: Self::Item);
        }
    }

    pub use runtime::Rect;
}
pub mod __Rect_area__ {
    pub type Item = crate::item::Item;

    pub trait __Rect_area__ { fn area(&self) -> Item; }
}
pub mod __Rect_perimeter__ {
    pub type Item = crate::item::Item;

    pub trait __Rect_perimeter__ { fn perimeter(&self) -> Item; }
}
pub mod __Rect_scale__ {
    pub type Item = crate::item::Item;

    pub trait __Rect_scale__ { fn scale(&mut self, scale: Item); }
}
use ____CGQAQ__SUPER_TRAIT____::Rect;
#[doc = "Test this will keep after expand"]
#[derive(Debug)]
struct Square {
    side: i32,
}
impl Rect for Square {
    type Item = crate::item::Item;
    fn area(&self) -> Self::Item { <dyn __Rect_area__::__Rect_area__>::area(self) }
    fn perimeter(&self) -> Self::Item { <dyn __Rect_perimeter__::__Rect_perimeter__>::perimeter(self) }
    fn scale(&mut self, scale: Self::Item) { <dyn __Rect_scale__::__Rect_scale__>::scale(self, scale) }
}
```