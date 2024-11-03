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
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if self.expires.contains_key(&key) && self.expires[&key] < now {
            self.data.remove(&key);
            return None;
        }
        self.data.get(&key).cloned()
    }

    pub fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value.to_string());
    }

    pub fn set_expire(&mut self, key: String, expire: i64) -> Result<(), String> {
        if expire < 0 {
            self.data.remove(&key);
            self.expires.remove(&key);
            return Ok(());
        } else {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            self.expires.insert(key, now + expire as u64);
        }
        Ok(())
    }

    pub fn get_ttl(&self, key: String) -> i64 {
        if !self.has(key.clone()) {
            return -2;
        }
        if let Some(expire) = self.expires.get(&key) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            return (*expire - now).try_into().unwrap();
        }
        -1
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
