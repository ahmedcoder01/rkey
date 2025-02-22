use std::collections::HashMap;

pub struct Storage {
    items: HashMap<String, String>
}

impl Storage {

    pub fn new() -> Self {
        Self {items: HashMap::new()}
    }

    pub fn set(&mut self, k: &str, v: &str) {
        self.items.insert(k.to_string(), v.to_string());
    }

    pub fn get(&self, k: &str) -> Option<&String> {
        self.items.get(k)
    }

    pub fn exists(&self, k: &str) -> bool {
        self.items.contains_key(k)
    }

    pub fn del(&mut self, k: &str) -> bool {
        self.items.remove(k).is_some()
    }
}