use std::collections::HashMap;

#[derive(Debug)]
pub struct Key {
    start: u32,
    end: u32,
}

#[derive(Debug, Default)]
pub struct Interner {
    map: HashMap<String, Key>,
    buf: String,
}

impl Interner {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&mut self, string: String) {
        self.map.entry(string).or_insert_with(|| {
//            let key = 
        })
    }
}
