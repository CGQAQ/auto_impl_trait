use xxx::auto_impl_trait;

#[auto_impl_trait("./src/rect_trait.rs")]
struct Square {
    side: i32,
}

fn main() {
    let s = Square { side: 3 };

    println!("area: {}", s.area());
    println!("perimeter: {}", s.perimeter());
}
