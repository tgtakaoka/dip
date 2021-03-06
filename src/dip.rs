use std::collections::HashMap;

pub struct Dip {
    pub name: String,
    pub pins: HashMap<u8, String>,
}

impl Dip {
    pub fn new(name: &str) -> Dip {
        Dip {
            name: String::from(name),
            pins: HashMap::new(),
        }
    }

    pub fn add(&mut self, pin: u8, name: &str) {
        self.pins.insert(pin, String::from(name));
    }
}
