pub struct Item {
    pub weight: u32,
}

impl Item {
    pub fn describe(&self) -> &str {
        "item"
    }

    pub fn kind(&self) -> &str {
        "generic"
    }
}
