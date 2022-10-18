pub struct Entity {
    pub data: String,
}

impl Entity {
    pub fn new(data: &'static str) -> Self {
        Entity {
            data: data.to_string(),
        }
    }
}
