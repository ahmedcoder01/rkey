use std::collections::HashMap;

struct Storage {
    items: HashMap<String, String>
}

impl Storage {

    fn new() -> Self {
        Self {items: HashMap::new()}
    }

    fn set(&mut self, k: &str, v: String) {
        self.items.insert(k.to_string(), v);
    }

    fn get(&self, k: &str) -> Option<&String> {
        self.items.get(k)
    }

    fn exists(&self, k: &str) -> bool {
        self.items.contains_key(k)
    }

    fn del(&mut self, k: &str) -> bool {
        self.items.remove(k).is_some()
    }
}