use auto_impl_trait::auto_impl_trait;

#[auto_impl_trait("./src/rect_trait.rs", Rect, "runtime")]
#[doc = "Test this will keep after expand"]
#[derive(Debug)]
struct Square {
    side: i32,
}

fn main() {
    let mut s =  Square { side: 3 };

    println!("area: {}", s.area());
    println!("perimeter: {}", s.perimeter());
    s.scale(7);
    println!("scale: {}", s.side);
}
