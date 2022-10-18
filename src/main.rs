use xxx::auto_impl_trait;

trait Rect {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
}

#[auto_impl_trait(Rect)]
struct Square {
    side: f64,
}

fn main() {}
