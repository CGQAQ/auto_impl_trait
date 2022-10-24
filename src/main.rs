use tonic::Request;
use auto_impl_trait::auto_impl;

#[auto_impl(image)]
pub struct Square {
    side: i32,
}

fn main() {
    let square = Square { side: 5 };
    square.list_images(Request::new(v1::ListImagesRequest::default()));
}
