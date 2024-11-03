use std::collections::HashMap;

pub struct Storage {
    data: HashMap<String, String>,
    expires: HashMap<String, u64>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            expires: HashMap::new(),
        }
    }

    pub fn get(&mut self, key: String) -> Option<String> {
        if self.expires.contains_key(&key) && self.expires[&key] < std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() {
            self.data.remove(&key);
            return None;
        }
        self.data.get(&key).cloned()
    }

    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value.to_string());
    }

    pub fn set_expire(&mut self, key: String, expire: u64) {
        self.expires.insert(key, expire);
    }

    pub fn has(&self, key: String) -> bool {
        self.data.contains_key(&key)
    }

    pub fn del(&mut self, key: String) {
        self.data.remove(&key);
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}
